//! The line protocol `litric-sysd` speaks on its socket.
//!
//! Deliberately text, deliberately line-oriented, and deliberately boring. The
//! shell, the settings app and the selftest all talk to the daemon; a binary
//! protocol would mean none of them could be debugged with `socat` at three in
//! the morning, and the daemon is not on a path where the encoding is the
//! bottleneck.
//!
//! ```text
//!   -> ENV PLAN windows
//!   <- OK 2
//!   <- outcome=restoring
//!   <- latency=tens-of-seconds
//! ```
//!
//! A reply always starts with `OK <count>` or `ERR <kind> <detail>`, so a
//! client can tell success from failure before it has parsed anything else, and
//! knows exactly how many lines to read. Framing that depends on the client
//! recognising the end of a payload is framing that breaks the first time a
//! payload contains something that looks like a terminator.

use core::fmt;

/// The longest request line accepted, in bytes.
///
/// A client that sends more than this is malfunctioning or hostile; either way
/// the daemon must not grow a buffer on its say-so. The daemon is the thing
/// that has to survive.
pub const MAX_LINE: usize = 4096;

/// What the caller wants to do with storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessWord {
    /// Read a file.
    Read,
    /// Modify a file.
    Write,
    /// Enumerate a directory.
    List,
}

impl AccessWord {
    /// Parse `read`, `write` or `list`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "list" => Some(Self::List),
            _ => None,
        }
    }
}

/// A parsed request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Request {
    /// Liveness.
    Ping,
    /// Measured system facts.
    Facts,
    /// Every environment and its state.
    EnvList,
    /// What switching to an environment *would* do. Nothing changes.
    EnvPlan(String),
    /// Switch to an environment.
    EnvSwitch(String),
    /// What is in the trash.
    TrashList,
    /// What an automatic empty would remove. Nothing is destroyed.
    TrashPlanEmpty,
    /// The capability audit log.
    CapsAudit,
    /// Resolve a storage URI for an application.
    Resolve {
        /// The application asking. Supplied by the daemon's caller, never by
        /// the application itself.
        app: String,
        /// The URI.
        uri: String,
        /// What it wants to do.
        access: AccessWord,
    },
    /// Close the connection.
    Quit,
}

/// Why a request was rejected before it reached any subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProtoError {
    /// The line was empty.
    Empty,
    /// The line exceeded [`MAX_LINE`].
    TooLong {
        /// How long it was.
        len: usize,
    },
    /// No such verb.
    UnknownVerb {
        /// What was sent.
        verb: String,
    },
    /// Right verb, wrong number of arguments.
    BadArity {
        /// The verb.
        verb: &'static str,
        /// How many arguments it takes.
        expected: usize,
        /// How many arrived.
        got: usize,
    },
    /// An argument was not a value this verb accepts.
    BadArgument {
        /// Which one.
        name: &'static str,
        /// What was sent.
        value: String,
    },
}

impl ProtoError {
    /// A stable machine-readable token, for the `ERR <kind>` field.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::TooLong { .. } => "too-long",
            Self::UnknownVerb { .. } => "unknown-verb",
            Self::BadArity { .. } => "bad-arity",
            Self::BadArgument { .. } => "bad-argument",
        }
    }
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty request"),
            Self::TooLong { len } => write!(f, "{len} bytes exceeds the {MAX_LINE} byte limit"),
            Self::UnknownVerb { verb } => write!(f, "no such verb: {verb}"),
            Self::BadArity {
                verb,
                expected,
                got,
            } => write!(f, "{verb} takes {expected} arguments, got {got}"),
            Self::BadArgument { name, value } => write!(f, "{name} cannot be {value}"),
        }
    }
}

impl std::error::Error for ProtoError {}

impl Request {
    /// Parse one request line.
    ///
    /// # Errors
    /// [`ProtoError`] describing what was wrong with the line.
    pub fn parse(line: &str) -> Result<Self, ProtoError> {
        if line.len() > MAX_LINE {
            return Err(ProtoError::TooLong { len: line.len() });
        }
        let mut words = line.split_whitespace();
        let Some(verb) = words.next() else {
            return Err(ProtoError::Empty);
        };
        // Verbs are matched case-insensitively; arguments never are, because an
        // application id and a path are case-sensitive and silently folding
        // them would resolve one app's storage for another.
        let verb_lower = verb.to_ascii_lowercase();
        let args: Vec<&str> = words.collect();

        let arity = |verb: &'static str, want: usize| -> Result<(), ProtoError> {
            if args.len() == want {
                Ok(())
            } else {
                Err(ProtoError::BadArity {
                    verb,
                    expected: want,
                    got: args.len(),
                })
            }
        };
        let arg = |i: usize| -> &str { args.get(i).copied().unwrap_or_default() };

        match verb_lower.as_str() {
            "ping" => {
                arity("PING", 0)?;
                Ok(Self::Ping)
            }
            "facts" => {
                arity("FACTS", 0)?;
                Ok(Self::Facts)
            }
            "quit" => {
                arity("QUIT", 0)?;
                Ok(Self::Quit)
            }
            "env" => match arg(0).to_ascii_lowercase().as_str() {
                "list" => {
                    arity("ENV LIST", 1)?;
                    Ok(Self::EnvList)
                }
                "plan" => {
                    arity("ENV PLAN", 2)?;
                    Ok(Self::EnvPlan(arg(1).to_owned()))
                }
                "switch" => {
                    arity("ENV SWITCH", 2)?;
                    Ok(Self::EnvSwitch(arg(1).to_owned()))
                }
                other => Err(ProtoError::BadArgument {
                    name: "ENV subcommand",
                    value: other.to_owned(),
                }),
            },
            "trash" => match arg(0).to_ascii_lowercase().as_str() {
                "list" => {
                    arity("TRASH LIST", 1)?;
                    Ok(Self::TrashList)
                }
                "plan-empty" => {
                    arity("TRASH PLAN-EMPTY", 1)?;
                    Ok(Self::TrashPlanEmpty)
                }
                other => Err(ProtoError::BadArgument {
                    name: "TRASH subcommand",
                    value: other.to_owned(),
                }),
            },
            "caps" => match arg(0).to_ascii_lowercase().as_str() {
                "audit" => {
                    arity("CAPS AUDIT", 1)?;
                    Ok(Self::CapsAudit)
                }
                other => Err(ProtoError::BadArgument {
                    name: "CAPS subcommand",
                    value: other.to_owned(),
                }),
            },
            "resolve" => {
                arity("RESOLVE", 3)?;
                let access = AccessWord::parse(&arg(2).to_ascii_lowercase()).ok_or_else(|| {
                    ProtoError::BadArgument {
                        name: "access",
                        value: arg(2).to_owned(),
                    }
                })?;
                Ok(Self::Resolve {
                    app: arg(0).to_owned(),
                    uri: arg(1).to_owned(),
                    access,
                })
            }
            _ => Err(ProtoError::UnknownVerb {
                verb: verb.to_owned(),
            }),
        }
    }
}

/// A reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    ok: bool,
    kind: String,
    detail: String,
    lines: Vec<String>,
}

impl Response {
    /// A successful reply carrying zero or more payload lines.
    #[must_use]
    pub fn ok(lines: Vec<String>) -> Self {
        Self {
            ok: true,
            kind: String::new(),
            detail: String::new(),
            lines,
        }
    }

    /// A failure.
    #[must_use]
    pub fn err(kind: &str, detail: impl Into<String>) -> Self {
        Self {
            ok: false,
            kind: kind.to_owned(),
            detail: detail.into(),
            lines: Vec::new(),
        }
    }

    /// Whether this reply reports success.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.ok
    }

    /// The payload lines.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// The wire form, newline-terminated.
    ///
    /// Payload lines have their newlines stripped: a value containing a newline
    /// would otherwise desynchronise the count in the header, and a client that
    /// trusts a count it can no longer rely on is a client that reads the next
    /// reply as part of this one.
    #[must_use]
    pub fn encode(&self) -> String {
        let mut out = String::new();
        if self.ok {
            out.push_str(&format!("OK {}\n", self.lines.len()));
            for l in &self.lines {
                out.push_str(&l.replace(['\n', '\r'], " "));
                out.push('\n');
            }
        } else {
            let detail = self.detail.replace(['\n', '\r'], " ");
            out.push_str(&format!("ERR {} {}\n", self.kind, detail));
        }
        out
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn simple_verbs_parse_and_are_case_insensitive() {
        assert_eq!(Request::parse("PING"), Ok(Request::Ping));
        assert_eq!(Request::parse("ping"), Ok(Request::Ping));
        assert_eq!(Request::parse("  Facts  "), Ok(Request::Facts));
        assert_eq!(Request::parse("QUIT"), Ok(Request::Quit));
    }

    #[test]
    fn subcommands_parse() {
        assert_eq!(Request::parse("ENV LIST"), Ok(Request::EnvList));
        assert_eq!(
            Request::parse("ENV PLAN windows"),
            Ok(Request::EnvPlan("windows".to_owned()))
        );
        assert_eq!(
            Request::parse("env switch litricos"),
            Ok(Request::EnvSwitch("litricos".to_owned()))
        );
        assert_eq!(
            Request::parse("TRASH PLAN-EMPTY"),
            Ok(Request::TrashPlanEmpty)
        );
        assert_eq!(Request::parse("CAPS AUDIT"), Ok(Request::CapsAudit));
    }

    #[test]
    fn resolve_parses_all_three_access_words() {
        for (w, want) in [
            ("read", AccessWord::Read),
            ("write", AccessWord::Write),
            ("list", AccessWord::List),
        ] {
            let r = Request::parse(&format!("RESOLVE com.litric.files user://docs {w}"));
            assert_eq!(
                r,
                Ok(Request::Resolve {
                    app: "com.litric.files".to_owned(),
                    uri: "user://docs".to_owned(),
                    access: want,
                })
            );
        }
    }

    #[test]
    fn arguments_are_not_case_folded() {
        // Folding an app id would resolve one application's private storage
        // for another whose id differs only in case.
        let Ok(Request::Resolve { app, uri, .. }) =
            Request::parse("RESOLVE Com.Litric.Files app://Notes/A.md read")
        else {
            panic!("expected a resolve request");
        };
        assert_eq!(app, "Com.Litric.Files");
        assert_eq!(uri, "app://Notes/A.md");
    }

    #[test]
    fn arity_is_enforced_in_both_directions() {
        assert!(matches!(
            Request::parse("PING extra"),
            Err(ProtoError::BadArity { verb: "PING", .. })
        ));
        assert!(matches!(
            Request::parse("ENV PLAN"),
            Err(ProtoError::BadArity {
                verb: "ENV PLAN",
                ..
            })
        ));
        assert!(matches!(
            Request::parse("RESOLVE a b"),
            Err(ProtoError::BadArity {
                verb: "RESOLVE",
                ..
            })
        ));
    }

    #[test]
    fn unknown_verbs_and_subcommands_are_named_in_the_error() {
        assert_eq!(
            Request::parse("SHUTDOWN"),
            Err(ProtoError::UnknownVerb {
                verb: "SHUTDOWN".to_owned()
            })
        );
        assert!(matches!(
            Request::parse("ENV DESTROY windows"),
            Err(ProtoError::BadArgument {
                name: "ENV subcommand",
                ..
            })
        ));
    }

    #[test]
    fn an_empty_line_is_an_error_not_a_ping() {
        assert_eq!(Request::parse(""), Err(ProtoError::Empty));
        assert_eq!(Request::parse("   \t "), Err(ProtoError::Empty));
    }

    #[test]
    fn an_oversized_line_is_refused_before_it_is_parsed() {
        let long = "PING ".to_owned() + &"a".repeat(MAX_LINE);
        assert!(matches!(
            Request::parse(&long),
            Err(ProtoError::TooLong { .. })
        ));
    }

    #[test]
    fn a_response_declares_its_line_count() {
        let r = Response::ok(vec!["a=1".to_owned(), "b=2".to_owned()]);
        assert_eq!(r.encode(), "OK 2\na=1\nb=2\n");
        assert_eq!(Response::ok(Vec::new()).encode(), "OK 0\n");
    }

    #[test]
    fn newlines_inside_a_payload_cannot_desynchronise_the_count() {
        let r = Response::ok(vec!["path=/tmp/a\nOK 99".to_owned()]);
        let wire = r.encode();
        assert_eq!(wire, "OK 1\npath=/tmp/a OK 99\n");
        assert_eq!(
            wire.lines().count(),
            2,
            "one header plus exactly the declared number of payload lines"
        );
    }

    #[test]
    fn an_error_reply_carries_a_stable_kind() {
        let e = Response::err("unknown-verb", "no such verb: SHUTDOWN");
        assert_eq!(e.encode(), "ERR unknown-verb no such verb: SHUTDOWN\n");
        assert!(!e.is_ok());
    }

    #[test]
    fn error_kinds_are_stable_tokens() {
        assert_eq!(ProtoError::Empty.kind(), "empty");
        assert_eq!(
            ProtoError::UnknownVerb {
                verb: "x".to_owned()
            }
            .kind(),
            "unknown-verb"
        );
        assert_eq!(ProtoError::TooLong { len: 9 }.kind(), "too-long");
    }
}
