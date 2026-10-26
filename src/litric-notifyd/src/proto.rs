//! The wire format, and the one place a notification is defined.
//!
//! # Shape
//!
//! Requests are one line. A request that carries a notification is followed by
//! a block of `key=value` lines terminated by `END`, which keeps titles and
//! bodies free of escaping: a notification title is a single line of text by
//! nature, so the line is the field.
//!
//! ```text
//!   POST                    -> OK 1
//!   app=com.litric.browser          id=3
//!   priority=high                   persisted=1
//!   ttl_ms=8000
//!   title=Download finished
//!   body=report.pdf is in Downloads
//!   END
//!
//!   TAKE                    -> OK 2
//!   NOTE 1                          NOTE 1
//!   app=…                           app=…
//!   priority=…                      priority=…
//!   title=…                         title=…
//!   body=…                          body=…
//!   created_ms=…                    created_ms=…
//!   expires_ms=…                    expires_ms=…
//!   END                             END
//!   …                               NOTE 2 …
//! ```
//!
//! A reply is `OK <count>` followed by exactly `count` lines, or `ERR <kind>
//! <message>`. The count is declared rather than implied by EOF because a client
//! that reads to EOF cannot tell a complete list from a truncated one — and a
//! truncated notification list looks exactly like "nothing arrived".
//!
//! # Why not JSON
//!
//! The workspace's runtime crates have no third-party dependencies (see the hard
//! rule at the top of `src/Cargo.toml`), so a JSON parser would be hand-written
//! here. A hand-written parser for a format with escapes is a parser with an
//! escape bug; a line-oriented key=value format has no escapes to get wrong, and
//! a person can read the queue file with `cat` when the desktop is broken, which
//! is the same argument `settings.conf` makes.

/// The longest line this protocol accepts, excluding the newline.
///
/// A title or body longer than this is refused rather than truncated: half a
/// sentence shown to a user as if it were the whole message is worse than a
/// sender that has to shorten its own text.
pub const MAX_LINE: usize = 4096;

/// The protocol version reported by `HELLO`.
pub const VERSION: u8 = 1;

/// The most `key=value` lines a posted block may have, so a client cannot grow
/// the parser's buffer by claiming a huge block.
pub const MAX_BLOCK_LINES: usize = 8;

/// How important a notification is.
///
/// The same four levels the shell draws, duplicated here on purpose: the wire
/// type must not depend on the shell's rendering crate. The one mapping between
/// them lives in the shell (`src/litric-shell/src/inbox.rs`) and carries a test
/// that walks every level in this ladder, so a level added here without a
/// counterpart there fails rather than silently becoming `normal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Priority {
    /// Background information.
    Low,
    /// The default.
    #[default]
    Normal,
    /// Needs attention soon.
    High,
    /// Something is wrong now. Never expires.
    Critical,
}

impl Priority {
    /// Every level, lowest first.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Low, Self::Normal, Self::High, Self::Critical]
    }

    /// The stable token used on the wire and in the queue file.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    /// Parse a token, or `None` for anything this build does not know.
    /// Case-insensitive, because a person typing `--priority HIGH` on a command
    /// line means high; the wire form this daemon writes is always lower case.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        Self::all()
            .iter()
            .copied()
            .find(|p| p.token().eq_ignore_ascii_case(raw))
    }
}

/// A notification that has been accepted by the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// Identity within this queue. Stable across a restart, because it is
    /// written with the note: an id that is reassigned is an id the shell would
    /// take twice.
    pub id: u32,
    /// The application that raised it, as a stable identifier rather than a
    /// display name (`com.litric.browser`, not "Browser").
    pub app: String,
    /// How important.
    pub priority: Priority,
    /// One line.
    pub title: String,
    /// More detail.
    pub body: String,
    /// When it was posted, in milliseconds since the epoch.
    pub created_ms: u64,
    /// When it stops being worth showing, in milliseconds since the epoch.
    /// `None` for a notification the sender wants kept (and always for
    /// [`Priority::Critical`]).
    pub expires_ms: Option<u64>,
}

/// A notification a client is asking the queue to keep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    /// Who raised it.
    pub app: String,
    /// How important.
    pub priority: Priority,
    /// One line.
    pub title: String,
    /// More detail.
    pub body: String,
    /// How long the sender says it is worth keeping, in milliseconds. `None`
    /// or `Some(0)` means "until it is taken or dismissed".
    pub ttl_ms: Option<u64>,
}

impl Pending {
    /// The absolute expiry this request means at `now_ms`.
    ///
    /// Two rules, and both are the shell's rules rather than this daemon's
    /// invention: a critical notification never expires, and a request with no
    /// ttl does not either.
    #[must_use]
    pub fn expires_at(&self, now_ms: u64) -> Option<u64> {
        if self.priority == Priority::Critical {
            return None;
        }
        match self.ttl_ms {
            Some(0) | None => None,
            Some(ttl) => Some(now_ms.saturating_add(ttl)),
        }
    }
}

/// What a client asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Liveness and version.
    Hello,
    /// Take the queue and clear it. This is what the desktop calls: a
    /// notification is delivered once, not on every start.
    Take,
    /// Read the queue without removing anything.
    List,
    /// How many are queued.
    Count,
    /// Remove one by id.
    Dismiss(u32),
    /// Remove everything.
    Clear,
    /// Report what this daemon is.
    Info,
    /// Close the connection.
    Quit,
}

/// Why a request was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtoError {
    /// Nothing on the line.
    Empty,
    /// A verb this build does not implement.
    Unknown(String),
    /// The right verb, wrong arguments.
    Malformed(String),
    /// A line over [`MAX_LINE`].
    TooLong,
}

impl ProtoError {
    /// The short kind written after `ERR `.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Unknown(_) => "unknown",
            Self::Malformed(_) => "malformed",
            Self::TooLong => "too-long",
        }
    }
}

impl core::fmt::Display for ProtoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty request"),
            Self::Unknown(verb) => write!(f, "unknown request: {verb}"),
            Self::Malformed(why) => write!(f, "malformed request: {why}"),
            Self::TooLong => write!(f, "a line exceeded {MAX_LINE} bytes"),
        }
    }
}

/// Parse the one-line part of a request.
pub fn parse_request(line: &str) -> Result<Request, ProtoError> {
    let line = line.trim_end();
    if line.len() > MAX_LINE {
        return Err(ProtoError::TooLong);
    }
    let mut parts = line.split_whitespace();
    let Some(verb) = parts.next() else {
        return Err(ProtoError::Empty);
    };
    match verb.to_ascii_uppercase().as_str() {
        "HELLO" => Ok(Request::Hello),
        "TAKE" => Ok(Request::Take),
        "LIST" => Ok(Request::List),
        "COUNT" => Ok(Request::Count),
        "CLEAR" => Ok(Request::Clear),
        "INFO" => Ok(Request::Info),
        "QUIT" | "BYE" => Ok(Request::Quit),
        // `POST` is followed by a block, and the block is collected by the
        // caller: this parser deliberately does not read the rest of the
        // connection, so it stays a pure function.
        "POST" => Err(ProtoError::Malformed(
            "POST is followed by a block; the caller must collect it".to_owned(),
        )),
        "DISMISS" => {
            let raw = parts.next().ok_or_else(|| {
                ProtoError::Malformed("DISMISS needs a notification id".to_owned())
            })?;
            raw.parse::<u32>()
                .map(Request::Dismiss)
                .map_err(|_| ProtoError::Malformed(format!("{raw} is not an id")))
        }
        other => Err(ProtoError::Unknown(other.to_owned())),
    }
}

/// Build a [`Pending`] from the block that followed `POST`.
///
/// `lines` are the raw lines between `POST` and `END`, with the terminator
/// already removed. Unknown keys are ignored rather than refused: a newer sender
/// adding `icon=` must not be the reason a notification is dropped.
pub fn parse_post(lines: &[String]) -> Result<Pending, ProtoError> {
    if lines.len() > MAX_BLOCK_LINES {
        return Err(ProtoError::Malformed(format!(
            "a post may carry at most {MAX_BLOCK_LINES} fields"
        )));
    }
    let mut app: Option<String> = None;
    let mut priority = Priority::Normal;
    let mut title: Option<String> = None;
    let mut body = String::new();
    let mut ttl_ms: Option<u64> = None;

    for line in lines {
        let Some((key, value)) = line.split_once('=') else {
            return Err(ProtoError::Malformed(format!(
                "a block line must be key=value, not {line:?}"
            )));
        };
        let value = value.trim();
        if value.len() > MAX_LINE {
            return Err(ProtoError::TooLong);
        }
        match key.trim().to_ascii_lowercase().as_str() {
            "app" => app = Some(value.to_owned()),
            "priority" => {
                priority = Priority::parse(value)
                    .ok_or_else(|| ProtoError::Malformed(format!("unknown priority {value:?}")))?;
            }
            "title" => title = Some(value.to_owned()),
            "body" => body = value.to_owned(),
            "ttl_ms" => {
                // An unreadable ttl is refused rather than defaulted: a sender
                // that meant "five seconds" and asked for "5s" would otherwise
                // get a notification that never expires.
                let parsed = value.parse::<u64>().map_err(|_| {
                    ProtoError::Malformed(format!("ttl_ms {value:?} is not a number"))
                })?;
                ttl_ms = Some(parsed);
            }
            _ => {}
        }
    }

    let app = app.ok_or_else(|| ProtoError::Malformed("a post needs app=".to_owned()))?;
    let title = title.ok_or_else(|| ProtoError::Malformed("a post needs title=".to_owned()))?;
    if app.is_empty() {
        return Err(ProtoError::Malformed("app= is empty".to_owned()));
    }
    if title.is_empty() {
        return Err(ProtoError::Malformed("title= is empty".to_owned()));
    }
    // Control characters would let a sender forge framing: a title containing a
    // newline is a second line of protocol, not a longer title.
    for (field, text) in [("app", &app), ("title", &title), ("body", &body)] {
        if text.chars().any(|c| c.is_control()) {
            return Err(ProtoError::Malformed(format!(
                "{field}= contains a control character"
            )));
        }
    }
    Ok(Pending {
        app,
        priority,
        title,
        body,
        ttl_ms,
    })
}

/// A reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// The lines after the header, whose count the header declares.
    pub lines: Vec<String>,
}

impl Response {
    /// A reply carrying `lines`.
    #[must_use]
    pub fn ok(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Encode as `OK <n>` plus one line per entry, each newline-terminated.
    #[must_use]
    pub fn encode(&self) -> String {
        let mut out = format!("OK {}\n", self.lines.len());
        for line in &self.lines {
            out.push_str(line);
            out.push('\n');
        }
        out
    }
}

/// An encoded refusal.
#[must_use]
pub fn encode_error(kind: &str, message: &str) -> String {
    // The message is one line by construction; a newline would break framing.
    let message: String = message
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    format!("ERR {kind} {message}\n")
}

/// The lines a [`Note`] is written as, without the `NOTE <id>` header:
/// everything a reader needs to reconstruct it.
#[must_use]
pub fn note_lines(note: &Note) -> Vec<String> {
    vec![
        format!("app={}", note.app),
        format!("priority={}", note.priority.token()),
        format!("title={}", note.title),
        format!("body={}", note.body),
        format!("created_ms={}", note.created_ms),
        format!("expires_ms={}", note.expires_ms.unwrap_or(0)),
    ]
}

/// Rebuild a note from the lines [`note_lines`] wrote.
#[must_use]
pub fn note_from_lines(id: u32, lines: &[String]) -> Option<Note> {
    let mut app = String::new();
    let mut priority = Priority::Normal;
    let mut title = String::new();
    let mut body = String::new();
    let mut created_ms = 0u64;
    let mut expires_ms: Option<u64> = None;
    let mut saw_title = false;
    for line in lines {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "app" => app = value.to_owned(),
            "priority" => priority = Priority::parse(value)?,
            "title" => {
                title = value.to_owned();
                saw_title = true;
            }
            "body" => body = value.to_owned(),
            "created_ms" => created_ms = value.parse().ok()?,
            "expires_ms" => {
                let n: u64 = value.parse().ok()?;
                expires_ms = if n == 0 { None } else { Some(n) };
            }
            _ => {}
        }
    }
    if !saw_title || app.is_empty() {
        return None;
    }
    Some(Note {
        id,
        app,
        priority,
        title,
        body,
        created_ms,
        expires_ms,
    })
}

/// Read the `NOTE <id>` header's id.
#[must_use]
pub fn note_header(line: &str) -> Option<u32> {
    let rest = line.strip_prefix("NOTE ")?;
    rest.trim().parse().ok()
}

/// Read a `key=value` line, for replies whose line count is one.
#[must_use]
pub fn field<'a>(lines: &'a [String], key: &str) -> Option<&'a str> {
    lines.iter().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix('='))
    })
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn block(pairs: &[(&str, &str)]) -> Vec<String> {
        pairs.iter().map(|(k, v)| format!("{k}={v}")).collect()
    }

    #[test]
    fn the_simple_verbs_parse() {
        for (line, expected) in [
            ("HELLO", Request::Hello),
            ("TAKE", Request::Take),
            ("LIST", Request::List),
            ("COUNT", Request::Count),
            ("CLEAR", Request::Clear),
            ("INFO", Request::Info),
            ("QUIT", Request::Quit),
            ("take", Request::Take),
            ("  TAKE  ", Request::Take),
        ] {
            assert_eq!(parse_request(line), Ok(expected), "{line}");
        }
    }

    #[test]
    fn a_dismiss_needs_a_number_that_is_a_number() {
        assert_eq!(parse_request("DISMISS 7"), Ok(Request::Dismiss(7)));
        assert!(matches!(
            parse_request("DISMISS"),
            Err(ProtoError::Malformed(_))
        ));
        assert!(matches!(
            parse_request("DISMISS seven"),
            Err(ProtoError::Malformed(_))
        ));
    }

    #[test]
    fn an_unknown_verb_is_refused_rather_than_guessed() {
        assert_eq!(
            parse_request("REBOOT"),
            Err(ProtoError::Unknown("REBOOT".to_owned()))
        );
        assert_eq!(parse_request("   "), Err(ProtoError::Empty));
    }

    #[test]
    fn post_is_not_a_one_line_request() {
        // The parser says so rather than pretending an empty post was accepted.
        assert!(matches!(
            parse_request("POST"),
            Err(ProtoError::Malformed(_))
        ));
    }

    #[test]
    fn a_post_block_becomes_a_pending_note() {
        let posted = parse_post(&block(&[
            ("app", "com.litric.browser"),
            ("priority", "high"),
            ("ttl_ms", "8000"),
            ("title", "Download finished"),
            ("body", "report.pdf is in Downloads"),
        ]))
        .expect("the block is valid");
        assert_eq!(posted.app, "com.litric.browser");
        assert_eq!(posted.priority, Priority::High);
        assert_eq!(posted.title, "Download finished");
        assert_eq!(posted.body, "report.pdf is in Downloads");
        assert_eq!(posted.ttl_ms, Some(8000));
    }

    #[test]
    fn a_post_needs_an_application_and_a_title() {
        assert!(matches!(
            parse_post(&block(&[("title", "x")])),
            Err(ProtoError::Malformed(_))
        ));
        assert!(matches!(
            parse_post(&block(&[("app", "a")])),
            Err(ProtoError::Malformed(_))
        ));
        assert!(matches!(
            parse_post(&block(&[("app", ""), ("title", "x")])),
            Err(ProtoError::Malformed(_))
        ));
        assert!(matches!(
            parse_post(&block(&[("app", "a"), ("title", "")])),
            Err(ProtoError::Malformed(_))
        ));
    }

    #[test]
    fn a_control_character_is_a_forged_line_and_is_refused() {
        // A title with a newline would be a second line of protocol. Refusing is
        // the only safe answer: escaping it would make this format need an
        // escape, which is the thing it was chosen to avoid.
        for bad in ["ok\nTAKE", "ok\rTAKE", "ok\u{0}"] {
            let posted = parse_post(&block(&[("app", "a"), ("title", bad)]));
            assert!(
                matches!(posted, Err(ProtoError::Malformed(_))),
                "{bad:?} was accepted"
            );
        }
    }

    #[test]
    fn an_unknown_field_is_ignored_not_refused() {
        // A newer sender adding a field must not lose the notification.
        let posted = parse_post(&block(&[
            ("app", "a"),
            ("title", "t"),
            ("icon", "document"),
            ("urgency", "5"),
        ]))
        .expect("unknown fields are tolerated");
        assert_eq!(posted.title, "t");
    }

    #[test]
    fn a_ttl_that_is_not_a_number_is_refused_rather_than_defaulted() {
        // "5s" meaning five seconds must not become "never expires".
        assert!(matches!(
            parse_post(&block(&[("app", "a"), ("title", "t"), ("ttl_ms", "5s")])),
            Err(ProtoError::Malformed(_))
        ));
    }

    #[test]
    fn a_block_line_without_an_equals_sign_is_refused() {
        assert!(matches!(
            parse_post(&["app=a".to_owned(), "title".to_owned()]),
            Err(ProtoError::Malformed(_))
        ));
    }

    #[test]
    fn an_oversized_block_is_refused_before_it_is_parsed() {
        let too_many: Vec<String> = (0..MAX_BLOCK_LINES + 1)
            .map(|i| format!("k{i}=v"))
            .collect();
        assert!(matches!(
            parse_post(&too_many),
            Err(ProtoError::Malformed(_))
        ));
    }

    #[test]
    fn critical_notifications_never_expire_and_neither_do_untimed_ones() {
        let mut posted = Pending {
            app: "a".to_owned(),
            priority: Priority::Critical,
            title: "t".to_owned(),
            body: String::new(),
            ttl_ms: Some(1_000),
        };
        assert_eq!(posted.expires_at(5_000), None);
        posted.priority = Priority::Normal;
        assert_eq!(posted.expires_at(5_000), Some(6_000));
        posted.ttl_ms = Some(0);
        assert_eq!(posted.expires_at(5_000), None);
        posted.ttl_ms = None;
        assert_eq!(posted.expires_at(5_000), None);
    }

    #[test]
    fn a_note_round_trips_through_its_lines() {
        let note = Note {
            id: 12,
            app: "com.litric.files".to_owned(),
            priority: Priority::Low,
            title: "Copy finished · ไฟล์ 3 ไฟล์".to_owned(),
            body: "ไว้ที่ Documents".to_owned(),
            created_ms: 1_700_000_000_000,
            expires_ms: Some(1_700_000_008_000),
        };
        let lines = note_lines(&note);
        assert_eq!(note_from_lines(12, &lines), Some(note));
        assert_eq!(note_header("NOTE 12"), Some(12));
        assert_eq!(note_header("NOTE twelve"), None);
        assert_eq!(note_header("FRAME 12"), None);
    }

    #[test]
    fn a_note_missing_its_title_is_not_reconstructed() {
        // Silence is worse than a refusal: a note with no title is a blank row
        // in the centre that the user cannot act on.
        let lines = vec!["app=a".to_owned(), "created_ms=1".to_owned()];
        assert_eq!(note_from_lines(1, &lines), None);
    }

    #[test]
    fn a_response_declares_its_own_length() {
        let encoded = Response::ok(vec!["a=1".to_owned(), "b=2".to_owned()]).encode();
        assert_eq!(encoded, "OK 2\na=1\nb=2\n");
        assert_eq!(Response::ok(Vec::new()).encode(), "OK 0\n");
    }

    #[test]
    fn an_error_message_cannot_break_framing() {
        let encoded = encode_error("malformed", "bad\nwarning: rest of reply");
        assert_eq!(encoded.lines().count(), 1, "{encoded:?}");
        assert!(encoded.starts_with("ERR malformed "));
    }

    #[test]
    fn every_priority_round_trips_through_its_token() {
        for priority in Priority::all() {
            assert_eq!(Priority::parse(priority.token()), Some(*priority));
        }
        assert_eq!(Priority::parse("urgent"), None);
        assert_eq!(Priority::parse(" HIGH "), Some(Priority::High));
    }
}
