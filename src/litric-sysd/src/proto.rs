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

use crate::action::PowerAction;
use crate::launch::MAX_ARGV;
use crate::providers::{self, AudioAction, AuthAction, BtAction, NetAction, SystemAction};
use crate::update::HealthSignal;
use litric_core::Version;

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
    /// The local client's active capabilities, and how the durable record was
    /// read.
    ///
    /// A question: it changes nothing, needs no capability, and is what makes a
    /// restart observable from outside the process (`store=loaded:<n>` after a
    /// clean start, `store=corrupt:…` when the record was refused).
    CapsList,
    /// Where the default sound sink's volume is, as the sound server reports it.
    ///
    /// A question, so it needs no capability and is allowed to the local client
    /// the way [`Request::Facts`] is: it changes nothing and reveals nothing
    /// that is not already on the screen behind the panel that asks.
    AudioLevel,
    /// Whether the Bluetooth radio is powered on, as BlueZ reports it.
    ///
    /// The same kind of question as [`Request::AudioLevel`], and here for the
    /// same reason: the radio's power is not in the sysfs facts, so a panel that
    /// showed anything but an answer from BlueZ would be showing what it last
    /// asked for. It changes nothing, so it needs no capability.
    BtStatus,
    /// Ask the machine for a power transition.
    ///
    /// `consent` is the UI's report that the user answered a prompt naming
    /// this action. Without it the broker refuses a destructive scope, so the
    /// protocol cannot be used to power the machine off by sending a line.
    Power {
        /// Which transition.
        action: PowerAction,
        /// Whether the user confirmed in the moment.
        consent: bool,
    },
    /// Ask a provider to change something: sound, the network, Bluetooth, or a
    /// credential check.
    ///
    /// `consent` is the same report as [`Request::Power`]'s and means the same
    /// thing, but only one of these actions needs it: `AuthSession` is a
    /// live-consent capability, so a verification without the `CONFIRM` word is
    /// refused by the broker. The three providers are reversible, and a volume
    /// slider that opened a dialog would be a worse interface for the same
    /// safety.
    Action {
        /// What to do.
        action: SystemAction,
        /// Whether the user answered a prompt naming this action.
        consent: bool,
    },
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
    /// Start a program on the user's behalf.
    ///
    /// The argument vector is already split: the daemon never passes it through
    /// a shell, so a value containing `;` or `$(…)` is one argument. Each
    /// element is hex on the wire because the protocol splits on whitespace and
    /// a desktop entry may hold a space in any argument.
    LaunchApp {
        /// The vector; the first element is the absolute program path.
        argv: Vec<String>,
    },
    /// Read the update generation state. A question: it changes nothing and
    /// needs no capability.
    UpdateStatus {
        /// The status schema the caller understands, if it said so. `None` is
        /// the current spelling; schema 1 and 2 clients are still served, but
        /// the system-written health fields are only returned for schema 3.
        schema: Option<u8>,
    },
    /// Stage a downloaded tree from the service's `incoming` directory.
    UpdateStage {
        /// The generation version being staged.
        version: Version,
        /// A relative path below `<root>/incoming`.
        source: String,
        /// Whether the caller confirmed the write in the moment.
        consent: bool,
    },
    /// Commit the staged candidate after the caller's health check.
    UpdateCommit {
        /// The caller's permission to ask for activation. The system-written
        /// health evidence, not this word, decides whether the commit happens.
        health: HealthSignal,
        /// Whether the caller confirmed the activation in the moment.
        consent: bool,
    },
    /// Return to the other committed generation.
    UpdateRollback {
        /// Whether the caller confirmed the switch in the moment.
        consent: bool,
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

        // The two words that appear as a second argument in several verbs. They
        // are matched case-insensitively, like verbs and unlike path arguments.
        let on_off = |word: &str| -> Result<bool, ProtoError> {
            match word.to_ascii_lowercase().as_str() {
                "on" => Ok(true),
                "off" => Ok(false),
                other => Err(ProtoError::BadArgument {
                    name: "on/off",
                    value: other.to_owned(),
                }),
            }
        };

        // A link takes the interface's own two words. This is a second closure
        // rather than a reuse of `on_off` because the first booted image proved
        // the difference: the in-guest self test sent `NET LINK lo up`, the
        // parser wanted `on`, and the reply was
        // `ERR bad-argument on/off cannot be up` — a real defect that only an
        // action asked for from inside the system could have found.
        let up_down = |word: &str| -> Result<bool, ProtoError> {
            match word.to_ascii_lowercase().as_str() {
                "up" => Ok(true),
                "down" => Ok(false),
                other => Err(ProtoError::BadArgument {
                    name: "up/down",
                    value: other.to_owned(),
                }),
            }
        };

        // Decode one hex field into text, refusing anything else without
        // repeating it. A secret that failed to decode is still a secret.
        let hex_text = |raw: &str, name: &'static str| -> Result<String, ProtoError> {
            let bytes = providers::unhex(raw).ok_or_else(|| ProtoError::BadArgument {
                name,
                value: "<not hex>".to_owned(),
            })?;
            String::from_utf8(bytes).map_err(|_| ProtoError::BadArgument {
                name,
                value: "<not utf-8>".to_owned(),
            })
        };

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
                "list" => {
                    arity("CAPS LIST", 1)?;
                    Ok(Self::CapsList)
                }
                other => Err(ProtoError::BadArgument {
                    name: "CAPS subcommand",
                    value: other.to_owned(),
                }),
            },
            "audio" => match arg(0).to_ascii_lowercase().as_str() {
                "level" => {
                    arity("AUDIO LEVEL", 1)?;
                    Ok(Self::AudioLevel)
                }
                "volume" => {
                    arity("AUDIO VOLUME", 2)?;
                    let percent = arg(1).parse::<u8>().map_err(|_| ProtoError::BadArgument {
                        name: "volume percent",
                        value: arg(1).to_owned(),
                    })?;
                    Ok(Self::Action {
                        action: SystemAction::Audio(AudioAction::Volume(percent)),
                        consent: false,
                    })
                }
                "mute" => {
                    arity("AUDIO MUTE", 2)?;
                    Ok(Self::Action {
                        action: SystemAction::Audio(AudioAction::Mute(on_off(arg(1))?)),
                        consent: false,
                    })
                }
                other => Err(ProtoError::BadArgument {
                    name: "AUDIO subcommand",
                    value: other.to_owned(),
                }),
            },
            "net" => match arg(0).to_ascii_lowercase().as_str() {
                "link" => {
                    arity("NET LINK", 3)?;
                    Ok(Self::Action {
                        action: SystemAction::Net(NetAction::SetLink {
                            iface: arg(1).to_owned(),
                            up: up_down(arg(2))?,
                        }),
                        consent: false,
                    })
                }
                "reconfigure" => {
                    arity("NET RECONFIGURE", 2)?;
                    Ok(Self::Action {
                        action: SystemAction::Net(NetAction::Reconfigure(arg(1).to_owned())),
                        consent: false,
                    })
                }
                "wifi" => match arg(1).to_ascii_lowercase().as_str() {
                    // A network name and a passphrase are hex on the wire: the
                    // protocol splits on whitespace, and a passphrase is a
                    // string a person typed. A value that is not hex is refused
                    // with `<not hex>` rather than echoed, so a malformed line
                    // cannot put a secret in an ERR reply.
                    "join" => {
                        arity("NET WIFI JOIN", 4)?;
                        Ok(Self::Action {
                            action: SystemAction::Net(NetAction::JoinWifi {
                                ssid: hex_text(arg(2), "network name")?,
                                passphrase: hex_text(arg(3), "passphrase")?,
                            }),
                            consent: false,
                        })
                    }
                    "forget" => {
                        arity("NET WIFI FORGET", 3)?;
                        Ok(Self::Action {
                            action: SystemAction::Net(NetAction::ForgetWifi(hex_text(
                                arg(2),
                                "network name",
                            )?)),
                            consent: false,
                        })
                    }
                    other => Err(ProtoError::BadArgument {
                        name: "NET WIFI subcommand",
                        value: other.to_owned(),
                    }),
                },
                other => Err(ProtoError::BadArgument {
                    name: "NET subcommand",
                    value: other.to_owned(),
                }),
            },
            "bt" | "bluetooth" => {
                let sub = arg(0).to_ascii_lowercase();
                let action = match sub.as_str() {
                    // A question rather than a change, like `AUDIO LEVEL`: the
                    // radio's power state exists only inside BlueZ, and this is
                    // how the interface reads it back instead of remembering
                    // what it asked for.
                    "status" => {
                        arity("BT STATUS", 1)?;
                        return Ok(Self::BtStatus);
                    }
                    "power" | "scan" => {
                        arity("BT POWER", 2)?;
                        let on = on_off(arg(1))?;
                        if sub == "power" {
                            BtAction::Power(on)
                        } else {
                            BtAction::Scan(on)
                        }
                    }
                    "pair" | "connect" | "forget" => {
                        arity("BT PAIR", 2)?;
                        let addr = arg(1).to_owned();
                        match sub.as_str() {
                            "pair" => BtAction::Pair(addr),
                            "connect" => BtAction::Connect(addr),
                            _ => BtAction::Forget(addr),
                        }
                    }
                    other => {
                        return Err(ProtoError::BadArgument {
                            name: "BT subcommand",
                            value: other.to_owned(),
                        })
                    }
                };
                Ok(Self::Action {
                    action: SystemAction::Bluetooth(action),
                    consent: false,
                })
            }
            "auth" => match arg(0).to_ascii_lowercase().as_str() {
                "verify" => {
                    // `AUTH VERIFY <user> <password-hex> [CONFIRM]`. The
                    // password is hex for the same reason a passphrase is, and
                    // it is never echoed: the reply for a bad line says the
                    // argument was not hex, not what it was.
                    if args.len() < 3 || args.len() > 4 {
                        return Err(ProtoError::BadArity {
                            verb: "AUTH VERIFY",
                            expected: 3,
                            got: args.len(),
                        });
                    }
                    let consent = match args.get(3) {
                        None => false,
                        Some(w) if w.eq_ignore_ascii_case("confirm") => true,
                        Some(other) => {
                            return Err(ProtoError::BadArgument {
                                name: "auth confirmation",
                                value: (*other).to_owned(),
                            })
                        }
                    };
                    Ok(Self::Action {
                        action: SystemAction::Auth(AuthAction::Verify {
                            user: arg(1).to_owned(),
                            password: hex_text(arg(2), "password")?,
                        }),
                        consent,
                    })
                }
                "create" => {
                    // Two spellings, one parser:
                    //   AUTH CREATE <user> <password-hex> [CONFIRM]
                    //   AUTH CREATE <user> <password-hex> <confirm-hex> [CONFIRM]
                    // The short form takes the confirmation to be the password
                    // itself; the long form lets the shared validator reject a
                    // mismatch before any helper is spawned. A fourth field that
                    // is the literal `CONFIRM` is the consent word, not a hash.
                    if args.len() < 3 || args.len() > 5 {
                        return Err(ProtoError::BadArity {
                            verb: "AUTH CREATE",
                            expected: 3,
                            got: args.len(),
                        });
                    }
                    let fourth = args.get(3).copied();
                    let fifth = args.get(4).copied();
                    let (confirm_hex, consent) = if args.len() == 3 {
                        (None, false)
                    } else if args.len() == 4 {
                        match fourth {
                            Some(w) if w.eq_ignore_ascii_case("confirm") => (None, true),
                            Some(raw) => (Some(raw), false),
                            None => (None, false),
                        }
                    } else {
                        match fifth {
                            Some(w) if w.eq_ignore_ascii_case("confirm") => (fourth, true),
                            Some(other) => {
                                return Err(ProtoError::BadArgument {
                                    name: "auth confirmation",
                                    value: other.to_owned(),
                                })
                            }
                            None => (None, false),
                        }
                    };
                    let password = hex_text(arg(2), "password")?;
                    let confirm = match confirm_hex {
                        Some(raw) => hex_text(raw, "password confirmation")?,
                        None => password.clone(),
                    };
                    Ok(Self::Action {
                        action: SystemAction::Auth(AuthAction::Create {
                            user: arg(1).to_owned(),
                            password,
                            confirm,
                        }),
                        consent,
                    })
                }
                "change" => {
                    // Two spellings, one parser:
                    //   AUTH CHANGE <user> <current-hex> <new-hex> [CONFIRM]
                    //   AUTH CHANGE <user> <current-hex> <new-hex> <confirm-hex>
                    //   [CONFIRM]
                    // The short form takes the confirmation to be the new
                    // password; the long form is checked for a mismatch. The
                    // current password is only ever compared by `unix_chkpwd`.
                    if args.len() < 4 || args.len() > 6 {
                        return Err(ProtoError::BadArity {
                            verb: "AUTH CHANGE",
                            expected: 4,
                            got: args.len(),
                        });
                    }
                    let fifth = args.get(4).copied();
                    let sixth = args.get(5).copied();
                    let (confirm_hex, consent) = if args.len() == 4 {
                        (None, false)
                    } else if args.len() == 5 {
                        match fifth {
                            Some(w) if w.eq_ignore_ascii_case("confirm") => (None, true),
                            Some(raw) => (Some(raw), false),
                            None => (None, false),
                        }
                    } else {
                        match sixth {
                            Some(w) if w.eq_ignore_ascii_case("confirm") => (fifth, true),
                            Some(other) => {
                                return Err(ProtoError::BadArgument {
                                    name: "auth confirmation",
                                    value: other.to_owned(),
                                })
                            }
                            None => (None, false),
                        }
                    };
                    let current = hex_text(arg(2), "current password")?;
                    let new_password = hex_text(arg(3), "new password")?;
                    let confirm = match confirm_hex {
                        Some(raw) => hex_text(raw, "password confirmation")?,
                        None => new_password.clone(),
                    };
                    Ok(Self::Action {
                        action: SystemAction::Auth(AuthAction::Change {
                            user: arg(1).to_owned(),
                            current,
                            new_password,
                            confirm,
                        }),
                        consent,
                    })
                }
                other => Err(ProtoError::BadArgument {
                    name: "AUTH subcommand",
                    value: other.to_owned(),
                }),
            },
            "launch" => {
                if args.is_empty() || args.len() > MAX_ARGV {
                    return Err(ProtoError::BadArity {
                        verb: "LAUNCH",
                        expected: 1,
                        got: args.len(),
                    });
                }
                let mut argv = Vec::with_capacity(args.len());
                for raw in args.iter().copied() {
                    let value = hex_text(raw, "argument")?;
                    if value.contains('\0') {
                        return Err(ProtoError::BadArgument {
                            name: "argument",
                            value: "<nul>".to_owned(),
                        });
                    }
                    argv.push(value);
                }
                Ok(Self::LaunchApp { argv })
            }
            "power" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(ProtoError::BadArity {
                        verb: "POWER",
                        expected: 1,
                        got: args.len(),
                    });
                }
                let action = PowerAction::parse(arg(0)).ok_or_else(|| ProtoError::BadArgument {
                    name: "power action",
                    value: arg(0).to_owned(),
                })?;
                // The confirmation word is optional and must be exactly
                // `confirm`. Anything else is a refusal rather than a
                // best-effort interpretation: a request that misspells the
                // word it needs to end the session must not be read as if it
                // had said it.
                let consent = match args.get(1) {
                    None => false,
                    Some(w) if w.eq_ignore_ascii_case("confirm") => true,
                    Some(other) => {
                        return Err(ProtoError::BadArgument {
                            name: "power confirmation",
                            value: (*other).to_owned(),
                        })
                    }
                };
                Ok(Self::Power { action, consent })
            }
            "update" => match arg(0).to_ascii_lowercase().as_str() {
                "status" => {
                    if args.len() > 2 {
                        return Err(ProtoError::BadArity {
                            verb: "UPDATE STATUS",
                            expected: 2,
                            got: args.len(),
                        });
                    }
                    let schema = match args.get(1) {
                        None => None,
                        Some(raw) => {
                            Some(raw.parse::<u8>().map_err(|_| ProtoError::BadArgument {
                                name: "update schema",
                                value: (*raw).to_owned(),
                            })?)
                        }
                    };
                    Ok(Self::UpdateStatus { schema })
                }
                "stage" => {
                    arity("UPDATE STAGE", 4)?;
                    if !arg(3).eq_ignore_ascii_case("confirm") {
                        return Err(ProtoError::BadArgument {
                            name: "update confirmation",
                            value: arg(3).to_owned(),
                        });
                    }
                    let version = Version::parse(arg(1)).map_err(|_| ProtoError::BadArgument {
                        name: "update version",
                        value: arg(1).to_owned(),
                    })?;
                    Ok(Self::UpdateStage {
                        version,
                        source: arg(2).to_owned(),
                        consent: true,
                    })
                }
                "commit" => {
                    arity("UPDATE COMMIT", 3)?;
                    if !arg(2).eq_ignore_ascii_case("confirm") {
                        return Err(ProtoError::BadArgument {
                            name: "update confirmation",
                            value: arg(2).to_owned(),
                        });
                    }
                    let health =
                        HealthSignal::parse(arg(1)).ok_or_else(|| ProtoError::BadArgument {
                            name: "health signal",
                            value: arg(1).to_owned(),
                        })?;
                    Ok(Self::UpdateCommit {
                        health,
                        consent: true,
                    })
                }
                "rollback" => {
                    arity("UPDATE ROLLBACK", 2)?;
                    if !arg(1).eq_ignore_ascii_case("confirm") {
                        return Err(ProtoError::BadArgument {
                            name: "update confirmation",
                            value: arg(1).to_owned(),
                        });
                    }
                    Ok(Self::UpdateRollback { consent: true })
                }
                other => Err(ProtoError::BadArgument {
                    name: "UPDATE subcommand",
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
        assert_eq!(Request::parse("CAPS LIST"), Ok(Request::CapsList));
        assert_eq!(
            Request::parse("CAPS LIST extra"),
            Err(ProtoError::BadArity {
                verb: "CAPS LIST",
                expected: 1,
                got: 2
            })
        );
    }

    // ---- the write half: provider actions ---------------------------------

    fn action(request: &str) -> SystemAction {
        match Request::parse(request) {
            Ok(Request::Action { action, .. }) => action,
            other => panic!("{request} did not parse to an action: {other:?}"),
        }
    }

    #[test]
    fn a_link_takes_up_and_down_and_not_on_and_off() {
        // The shape the in-guest self test uses, and the shape the parser must
        // accept. `on`/`off` is what this verb said before the first booted
        // image disagreed with it.
        assert_eq!(
            action("NET LINK lo up"),
            SystemAction::Net(NetAction::SetLink {
                iface: "lo".to_owned(),
                up: true
            })
        );
        assert_eq!(
            action("net link eth0 DOWN"),
            SystemAction::Net(NetAction::SetLink {
                iface: "eth0".to_owned(),
                up: false
            })
        );
        let refused = Request::parse("NET LINK eth0 on").expect_err("a link is not on or off");
        assert_eq!(refused.kind(), "bad-argument");
    }

    #[test]
    fn the_radio_takes_on_and_off() {
        assert_eq!(
            action("BT POWER ON"),
            SystemAction::Bluetooth(BtAction::Power(true))
        );
        // A question, not an action: `BT STATUS` reads the radio's power back
        // from BlueZ and must not be mistaken for `BT POWER` with an argument.
        assert_eq!(Request::parse("BT STATUS"), Ok(Request::BtStatus));
        assert_eq!(Request::parse("bt status"), Ok(Request::BtStatus));
        let refused = Request::parse("BT STATUS ON").expect_err("status takes no argument");
        assert_eq!(refused.kind(), "bad-arity");
        assert_eq!(
            action("bluetooth scan off"),
            SystemAction::Bluetooth(BtAction::Scan(false))
        );
        assert_eq!(
            action("BT CONNECT aa:bb:cc:dd:ee:ff"),
            SystemAction::Bluetooth(BtAction::Connect("aa:bb:cc:dd:ee:ff".to_owned()))
        );
        // An address that is not one is refused by the action, not by the
        // parser: the parser's job is the shape of the line, and the action's is
        // whether a value makes sense. Both are refusals before anything runs.
        assert!(Request::parse("BT PAIR --help").is_ok());
    }

    #[test]
    fn audio_parses_a_percentage_and_a_mute_word() {
        assert_eq!(
            action("AUDIO VOLUME 40"),
            SystemAction::Audio(AudioAction::Volume(40))
        );
        assert_eq!(
            action("audio mute on"),
            SystemAction::Audio(AudioAction::Mute(true))
        );
        assert!(Request::parse("AUDIO VOLUME loud").is_err());
        assert!(Request::parse("AUDIO VOLUME 500").is_err(), "not a u8");
    }

    #[test]
    fn a_wifi_join_decodes_hex_and_never_echoes_a_secret() {
        // "Home" and "correct horse" in hex.
        let request = "NET WIFI JOIN 486f6d65 636f727265637420686f727365";
        assert_eq!(
            action(request),
            SystemAction::Net(NetAction::JoinWifi {
                ssid: "Home".to_owned(),
                passphrase: "correct horse".to_owned()
            })
        );
        let refused =
            Request::parse("NET WIFI JOIN zz 636f727265637420686f727365").expect_err("not hex");
        let text = refused.to_string();
        assert!(text.contains("<not hex>"), "{text}");
        assert!(
            !text.contains("correct horse") && !text.contains("636f7272"),
            "a malformed line must not put a secret in the reply: {text}"
        );
    }

    #[test]
    fn a_parsed_request_never_renders_the_credential_it_carries() {
        // The line a client sends, with "Home" and "correct horse" in hex. A
        // `{:?}` of the request is how a log line is added by reflex; it must
        // not be the way a passphrase ends up in the journal.
        let request =
            Request::parse("NET WIFI JOIN 486f6d65 636f727265637420686f727365").expect("a join");
        let rendered = format!("{request:?}");
        assert!(
            !rendered.contains("correct horse"),
            "Debug leaked the passphrase: {rendered}"
        );
        assert!(
            !rendered.contains("636f727265637420686f727365"),
            "Debug leaked the hex encoding: {rendered}"
        );
        assert!(
            rendered.contains("Home"),
            "the network name is not a secret and stays readable: {rendered}"
        );

        let verify =
            Request::parse("AUTH VERIFY alice 77726f6e67 CONFIRM").expect("a verification");
        let rendered = format!("{verify:?}");
        assert!(
            !rendered.contains("wrong"),
            "Debug leaked the password: {rendered}"
        );
        assert!(
            !rendered.contains("77726f6e67"),
            "Debug leaked the hex encoding: {rendered}"
        );
        assert!(
            rendered.contains("alice"),
            "the user name is not a secret and stays readable: {rendered}"
        );
    }

    #[test]
    fn a_verification_carries_its_confirmation_or_it_does_not_carry_one() {
        match Request::parse("AUTH VERIFY alice 77726f6e67 CONFIRM") {
            Ok(Request::Action {
                action: SystemAction::Auth(AuthAction::Verify { user, password }),
                consent,
            }) => {
                assert_eq!(user, "alice");
                assert_eq!(password, "wrong");
                assert!(consent, "the word is the answer to the prompt");
            }
            other => panic!("did not parse: {other:?}"),
        }
        match Request::parse("AUTH VERIFY alice 77726f6e67") {
            Ok(Request::Action { consent, .. }) => {
                assert!(!consent, "no word, no consent");
            }
            other => panic!("did not parse: {other:?}"),
        }
        // A misspelled confirmation is a refusal, not a best-effort read.
        assert!(Request::parse("AUTH VERIFY alice 77726f6e67 sure").is_err());
        let refused =
            Request::parse("AUTH VERIFY alice 77726f6e67 CONFIRM CONFIRM").expect_err("too many");
        assert_eq!(refused.kind(), "bad-arity");
    }

    #[test]
    fn account_creation_and_change_parse_without_echoing_a_secret() {
        // `correct` in hex, twice: the password and its confirmation.
        let hex = "636f7272656374";
        match Request::parse(&format!("AUTH CREATE alice {hex} {hex} CONFIRM")) {
            Ok(Request::Action {
                action:
                    SystemAction::Auth(AuthAction::Create {
                        user,
                        password,
                        confirm,
                    }),
                consent,
            }) => {
                assert_eq!(user, "alice");
                assert_eq!(password, "correct");
                assert_eq!(confirm, "correct");
                assert!(consent, "the word is the answer to the prompt");
            }
            other => panic!("did not parse: {other:?}"),
        }
        let change = Request::parse(&format!(
            "AUTH CHANGE alice {hex} 616e6f74686572 616e6f74686572 CONFIRM"
        ))
        .expect("a change");
        match &change {
            Request::Action {
                action:
                    SystemAction::Auth(AuthAction::Change {
                        user,
                        current,
                        new_password,
                        confirm,
                    }),
                consent,
            } => {
                assert_eq!(user, "alice");
                assert_eq!(current, "correct");
                assert_eq!(new_password, "another");
                assert_eq!(confirm, "another");
                assert!(*consent);
            }
            other => panic!("did not parse: {other:?}"),
        }
        let rendered = format!("{change:?}");
        assert!(!rendered.contains(hex), "Debug leaked the hex: {rendered}");
        assert!(!rendered.contains("correct"), "Debug leaked the password");
        assert!(rendered.contains("alice"), "the account is not a secret");

        // A field that is not hex is refused without repeating it.
        let refused = Request::parse("AUTH CREATE alice zzqq zzqq CONFIRM").expect_err("not hex");
        assert_eq!(refused.kind(), "bad-argument");
        assert!(
            !refused.to_string().contains("zzqq"),
            "the refusal echoed the payload: {refused}"
        );
    }

    /// Compose the canonical line for an auth action in the spelling this
    /// build prefers: short form when the confirmation equals the password,
    /// long form when it does not, and the legacy line for a verification.
    fn compose(action: &SystemAction, consent: bool) -> String {
        fn hex(text: &str) -> String {
            text.bytes().map(|b| format!("{b:02x}")).collect()
        }
        let SystemAction::Auth(action) = action else {
            panic!("not an auth action: {action:?}")
        };
        let mut line = match action {
            AuthAction::Verify { user, password } => {
                format!("AUTH VERIFY {user} {}", hex(password))
            }
            AuthAction::Create {
                user,
                password,
                confirm,
            } if password == confirm => format!("AUTH CREATE {user} {}", hex(password)),
            AuthAction::Create {
                user,
                password,
                confirm,
            } => format!("AUTH CREATE {user} {} {}", hex(password), hex(confirm)),
            AuthAction::Change {
                user,
                current,
                new_password,
                confirm,
            } if new_password == confirm => {
                format!("AUTH CHANGE {user} {} {}", hex(current), hex(new_password))
            }
            AuthAction::Change {
                user,
                current,
                new_password,
                confirm,
            } => format!(
                "AUTH CHANGE {user} {} {} {}",
                hex(current),
                hex(new_password),
                hex(confirm)
            ),
        };
        if consent {
            line.push_str(" CONFIRM");
        }
        line
    }

    #[test]
    fn auth_lines_are_stable_across_parse_and_recomposition() {
        // The legacy verification line is byte-identical: the parser and the
        // canonical composer agree on the same spelling it has always had.
        let legacy = "AUTH VERIFY alice 77726f6e67 CONFIRM";
        let parsed = Request::parse(legacy).expect("legacy verify");
        match &parsed {
            Request::Action { action, consent } => {
                assert_eq!(compose(action, *consent), legacy);
                assert_eq!(
                    Request::parse(&compose(action, *consent)),
                    Ok(parsed.clone()),
                    "recomposing the legacy line changed it"
                );
            }
            other => panic!("not an action: {other:?}"),
        }

        for line in [
            // The short spec forms, with and without the consent word.
            "AUTH CREATE alice 636f727265637420686f727365  CONFIRM",
            "AUTH CREATE alice 636f727265637420686f727365",
            "AUTH CHANGE alice 77726f6e67 616e6f7468657220686f727365 616e6f7468657220686f727365 CONFIRM",
            // The long form, where the confirmation is its own field.
            "AUTH CREATE bob 636f727265637420686f727365 616e6f7468657220686f727365 CONFIRM",
            "AUTH CHANGE alice 77726f6e67 616e6f7468657220686f727365 736f6d657468696e6720656c7365 CONFIRM",
        ] {
            let parsed = Request::parse(line).expect(line);
            match &parsed {
                Request::Action { action, consent } => {
                    let once = compose(action, *consent);
                    let twice = Request::parse(&once).expect(&once);
                    match &twice {
                        Request::Action {
                            action: again,
                            consent: again_consent,
                        } => assert_eq!(compose(again, *again_consent), once, "{line}"),
                        other => panic!("not an action: {other:?}"),
                    }
                    assert_eq!(Request::parse(&once), Ok(parsed.clone()), "{line}");
                }
                other => panic!("not an action: {other:?}"),
            }
        }
    }

    #[test]
    fn unknown_provider_subcommands_are_refused_by_name() {
        for line in [
            "AUDIO LOUDER 5",
            "NET FLUSH eth0",
            "NET WIFI FORGET-EVERYTHING now",
            "BT DISCOVER",
            "AUTH FORGET alice",
        ] {
            let err = Request::parse(line).expect_err(line);
            assert_eq!(err.kind(), "bad-argument", "{line}");
        }
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
    fn power_actions_parse_and_carry_consent() {
        for (word, action) in [
            ("SUSPEND", PowerAction::Suspend),
            ("reboot", PowerAction::Reboot),
            ("PowerOff", PowerAction::PowerOff),
        ] {
            assert_eq!(
                Request::parse(&format!("POWER {word}")),
                Ok(Request::Power {
                    action,
                    consent: false
                })
            );
            assert_eq!(
                Request::parse(&format!("power {word} confirm")),
                Ok(Request::Power {
                    action,
                    consent: true
                })
            );
        }
    }

    #[test]
    fn a_power_request_that_is_not_understood_is_refused_rather_than_guessed() {
        // `hibernate` is a real verb of the tool and not an action this system
        // has: it must fail loudly, not be folded into the nearest action.
        assert!(matches!(
            Request::parse("POWER HIBERNATE"),
            Err(ProtoError::BadArgument {
                name: "power action",
                ..
            })
        ));
        // A misspelled confirmation is not consent.
        assert!(matches!(
            Request::parse("POWER POWEROFF yes"),
            Err(ProtoError::BadArgument {
                name: "power confirmation",
                ..
            })
        ));
        assert!(matches!(
            Request::parse("POWER"),
            Err(ProtoError::BadArity { verb: "POWER", .. })
        ));
        assert!(matches!(
            Request::parse("POWER POWEROFF confirm please"),
            Err(ProtoError::BadArity { verb: "POWER", .. })
        ));
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
    fn update_requests_parse_with_an_explicit_health_and_confirmation() {
        assert_eq!(
            Request::parse("UPDATE STATUS"),
            Ok(Request::UpdateStatus { schema: None })
        );
        assert_eq!(
            Request::parse("UPDATE STATUS 2"),
            Ok(Request::UpdateStatus { schema: Some(2) })
        );
        assert_eq!(
            Request::parse("UPDATE STATUS 999")
                .expect_err("not a u8")
                .kind(),
            "bad-argument"
        );
        // 255 is a valid u8; whether this daemon speaks it is a version
        // question, answered by `state` as `schema-unsupported`, not a parse
        // error.
        assert_eq!(
            Request::parse("UPDATE STATUS 255"),
            Ok(Request::UpdateStatus { schema: Some(255) })
        );
        assert_eq!(
            Request::parse("update stage 1.1.0 release CONFIRM"),
            Ok(Request::UpdateStage {
                version: Version::parse("1.1.0").expect("version"),
                source: "release".to_owned(),
                consent: true,
            })
        );
        assert_eq!(
            Request::parse("UPDATE COMMIT PASSED confirm"),
            Ok(Request::UpdateCommit {
                health: HealthSignal::Passed,
                consent: true,
            })
        );
        assert_eq!(
            Request::parse("UPDATE ROLLBACK CONFIRM"),
            Ok(Request::UpdateRollback { consent: true })
        );
    }

    #[test]
    fn an_update_without_confirmation_or_with_a_bad_version_is_refused() {
        assert!(matches!(
            Request::parse("UPDATE STAGE 1.1.0 release"),
            Err(ProtoError::BadArity {
                verb: "UPDATE STAGE",
                ..
            })
        ));
        assert!(matches!(
            Request::parse("UPDATE STAGE 1.1.0 release yes"),
            Err(ProtoError::BadArgument {
                name: "update confirmation",
                ..
            })
        ));
        assert!(matches!(
            Request::parse("UPDATE STAGE nope release CONFIRM"),
            Err(ProtoError::BadArgument {
                name: "update version",
                ..
            })
        ));
        assert!(matches!(
            Request::parse("UPDATE COMMIT maybe CONFIRM"),
            Err(ProtoError::BadArgument {
                name: "health signal",
                ..
            })
        ));
        assert!(matches!(
            Request::parse("UPDATE NONSENSE"),
            Err(ProtoError::BadArgument {
                name: "UPDATE subcommand",
                ..
            })
        ));
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
