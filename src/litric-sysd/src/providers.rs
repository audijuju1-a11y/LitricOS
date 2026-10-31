//! The write half of four wave-1 providers: audio, network, Bluetooth and
//! authentication.
//!
//! [`crate::action`] owns the runner seam and the one transition that existed
//! before this module — a power request. This module is the rest of
//! `docs/design/wave1/`, and it exists because "the facts are already read" was
//! the state of all four of them: a Settings pane could say what the machine
//! *is* and nothing else.
//!
//! # How each one is reached, and why nothing here is a client library
//!
//! ADR-0023 decision 2: the base tool that already owns the interface, spoken to
//! the way it is meant to be spoken to.
//!
//! | Area | Tool | Why that tool |
//! |---|---|---|
//! | Audio | `/usr/bin/wpctl` | WirePlumber owns the PipeWire graph; `wpctl` is its own command-line client, shipped by `wireplumber`, which the desktop profile already carries |
//! | Network | `/usr/bin/ip`, `/usr/bin/networkctl`, and `iwd`'s credential file | `networkd` reads drop-ins and `iwd` reads `/var/lib/iwd/*.psk`; joining a **known** network is a file write, which is what `00-network.md` §2 decided |
//! | Bluetooth | `/usr/bin/bluetoothctl` | `bluez-utils`' client for the daemon; BlueZ is D-Bus-only, and this is the base tool rather than a D-Bus client in the daemon |
//! | Authentication | `/usr/bin/unix_chkpwd` | the helper `pam_unix` itself executes, setuid root, reading the password on **stdin** so it never reaches an argument vector or a log |
//!
//! No dependency is added, no socket is opened, and `unsafe` is still forbidden
//! in this crate. `unix_chkpwd` is the distribution's credential store spoken to
//! through the distribution's own helper — which is why there is no password
//! database, no hash format and no PAM FFI anywhere in this tree.
//!
//! # The shape of an action
//!
//! Every action is [`SystemAction`]: a closed enumeration, parsed from a token,
//! validated at construction, and turned into a [`Plan`] *before* anything
//! happens. The plan names the program, its exact argument vector, any standard
//! input, and any file the action writes. [`perform`] then applies it.
//!
//! That split is what makes the module testable on a host with no sound server,
//! no network daemon and no Bluetooth radio: a test builds an action, reads the
//! plan, and asserts the argv and the file contents without running anything —
//! and where running matters, it substitutes [`crate::action::testing::RecordingRunner`],
//! which records instead of spawning.
//!
//! # What a request can and cannot contribute
//!
//! Arguments are validated before a plan exists ([`Refusal`] names the rule that
//! was broken), the program path is a `const`, and the argument vector is built
//! from validated values, never from a shell string. Secrets — a Wi-Fi
//! passphrase, a password — travel as hex on the request line, are written to a
//! file or to a child's stdin, and are **never** echoed in a reply line:
//! [`Plan::command_line`] is what the reply carries, and it names the program
//! and non-secret arguments only.

use core::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use litric_caps::{Capability, CapabilityId, Scope};

use crate::action::{Outcome, Runner};
use crate::services::ServiceAction;
use crate::store::StoreAction;

/// WirePlumber's own client, which owns the PipeWire graph's default sink.
pub const WPCTL: &str = "/usr/bin/wpctl";

/// iproute2's `ip`, already a documented base-tool dependency of this build for
/// IPv4 addresses (`platform::net`). Used here for link administration.
pub const IP: &str = "/usr/bin/ip";

/// `networkctl`, `systemd-networkd`'s own client.
pub const NETWORKCTL: &str = "/usr/bin/networkctl";

/// `iwd`'s own client, shipped by the `iwd` package the core profile carries.
pub const IWCTL: &str = "/usr/bin/iwctl";

/// BlueZ's client, shipped by `bluez-utils`.
pub const BLUETOOTHCTL: &str = "/usr/bin/bluetoothctl";

/// `pam_unix`'s credential helper.
///
/// Setuid root on an Arch install, reads the password on standard input, and
/// answers with its exit status. It is the same binary `pam_unix` runs when a
/// program logs in, which is the point: the credential store stays the
/// distribution's, and this daemon holds no hash format of its own.
pub const UNIX_CHKPWD: &str = "/usr/bin/unix_chkpwd";

/// Where `iwd` keeps the credentials of known networks, relative to the root the
/// daemon was given.
const IWD_STATE_DIR: &str = "var/lib/iwd";

/// The deadline for a tool that answers from a local socket.
pub const SOCKET_ROUND_TRIP_SECS: u32 = 10;

/// The deadline for a tool that may have to wait for a radio, a daemon or a
/// device at the other end.
pub const RADIO_OPERATION_SECS: u32 = 30;

/// The deadline for a change that is applied to the kernel through a tool that
/// answers immediately.
pub const LOCAL_CHANGE_SECS: u32 = 15;

/// The largest Wi-Fi passphrase this build accepts, from WPA-PSK.
const MAX_PASSPHRASE: usize = 63;

/// The smallest, likewise.
const MIN_PASSPHRASE: usize = 8;

/// The longest 802.11 service set identifier, in bytes.
const MAX_SSID: usize = 32;

/// A request that cannot be turned into a plan.
///
/// Refused *before* anything runs or is written: a value that breaks a rule this
/// build states. There is one kind, and it is a value rule, because this build's
/// four providers have no "not implemented yet" branch left — what is not
/// implemented (a Wi-Fi scan, power profiles) has no action to reach it, rather
/// than an action that answers "no".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The machine-readable half of the answer.
    pub token: &'static str,
    /// The sentence a person reads.
    pub detail: String,
}

impl Refusal {
    /// A value that breaks a stated rule.
    pub(crate) fn illegal(what: &str, why: &str) -> Self {
        Self {
            token: "bad-argument",
            detail: format!("{what} {why}"),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.token, self.detail)
    }
}

// ---------------------------------------------------------------------------
// validation
// ---------------------------------------------------------------------------

/// Whether `name` is usable as a network interface name.
///
/// The kernel's own limit is 15 bytes plus a terminator. The rest of the rules
/// are about what must not reach an argument vector: a name that begins with
/// `-` would be read as an option by `ip`, and a name containing `/` or a
/// control character is not an interface the kernel would have created.
#[must_use]
pub fn valid_iface(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 15
        && name != "."
        && name != ".."
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':' | b'@'))
}

/// Whether `name` is usable as an account name on this distribution.
#[must_use]
pub fn valid_user(name: &str) -> bool {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    name.len() <= 32
        && (first.is_ascii_lowercase() || first == b'_')
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Whether `addr` is a Bluetooth device address, in the form `bluetoothctl`
/// prints and accepts: six colon-separated hex pairs.
///
/// A radio's own address is never all zeroes, so that one is refused: an
/// all-zero address is what a caller that filled in nothing would send, and it
/// must not reach a pairing request.
#[must_use]
pub fn valid_bdaddr(addr: &str) -> bool {
    if addr.len() != 17 {
        return false;
    }
    let mut groups = 0_u8;
    for (i, part) in addr.split(':').enumerate() {
        if part.len() != 2 || !part.bytes().all(|b| b.is_ascii_hexdigit()) {
            return false;
        }
        if i > 5 {
            return false;
        }
        groups = groups.saturating_add(1);
    }
    groups == 6 && addr.bytes().any(|b| b != b'0' && b != b':')
}

/// Whether `bytes` is a name this build will write to `iwd`'s state directory.
///
/// The rules are the intersection of three: 802.11's 32-byte limit, what `iwd`
/// stores as a file name, and what this project is willing to write. A byte over
/// `0x7f`, a control character, a `/` or a backslash is **refused rather than
/// escaped**: `iwd` has its own escaping for a file name, and guessing at a
/// second implementation of it is how a credential ends up in a file the daemon
/// reads under a different name. The limitation is recorded in
/// `docs/design/wave1/00-network.md` §8.
#[must_use]
pub fn valid_ssid(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len() <= MAX_SSID
        && bytes.iter().all(|b| (0x20..=0x7e).contains(b))
        && !bytes.contains(&b'/')
        && !bytes.contains(&b'\\')
        && bytes.first() != Some(&b'.')
        && bytes != b"."
        && bytes != b".."
}

/// Whether `bytes` is a passphrase this build will write to a credential file.
///
/// WPA-PSK is 8 to 63 printable ASCII characters. The upper bound is the
/// standard's; the "printable ASCII" part is this build's, because a passphrase
/// containing a newline would end the `Passphrase=` line and start an attribute
/// nobody asked for. Non-ASCII passphrases are refused rather than truncated,
/// and the limitation is recorded in the same §8.
#[must_use]
pub fn valid_passphrase(bytes: &[u8]) -> bool {
    (MIN_PASSPHRASE..=MAX_PASSPHRASE).contains(&bytes.len())
        && bytes.iter().all(|b| (0x20..=0x7e).contains(b))
        && !bytes.contains(&b'\\')
}

/// Decode a hex string, the only encoding a secret travels in on the wire.
///
/// The protocol is line-oriented and splits on whitespace, so a secret has to be
/// re-encoded whatever happens; hex is chosen over any quoting scheme because it
/// has one rule and no escape cases to get wrong. It is **not** a security
/// measure and is not claimed as one: the transport is a `0600` unix socket
/// owned by this daemon, and what protects the secret is that the daemon never
/// puts it in a log, a reply or an audit entry.
///
/// # Errors
/// `None` for an odd length or any character that is not a hex digit.
#[must_use]
pub fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || text.is_empty() {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len().saturating_div(2));
    // Paired by `chunks`, so the two digits of a byte are the two digits of an
    // element rather than an index and the one after it.
    for pair in bytes.chunks(2) {
        let hi = (*pair.first()? as char).to_digit(16)?;
        let lo = (*pair.get(1)? as char).to_digit(16)?;
        out.push(u8::try_from(hi.saturating_mul(16).saturating_add(lo)).ok()?);
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// the actions
// ---------------------------------------------------------------------------

/// A volume change, in whole percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioAction {
    /// Set the default sink's volume, clamped by the tool's own limit.
    Volume(u8),
    /// Mute or unmute the default sink.
    Mute(bool),
}

impl AudioAction {
    /// The word the protocol and the reply use.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Volume(_) => "volume",
            Self::Mute(_) => "mute",
        }
    }

    /// The capability that authorises it, scoped to the default sink.
    #[must_use]
    pub fn capability(self) -> Option<Capability> {
        let scope = Scope::parse_resource("audio://default-sink").ok()?;
        Capability::new(CapabilityId::AudioSet, scope)
    }
}

/// A network change.
#[derive(Clone, PartialEq, Eq)]
pub enum NetAction {
    /// Bring an interface administratively up or down.
    SetLink {
        /// The interface name.
        iface: String,
        /// Up (`true`) or down.
        up: bool,
    },
    /// Ask `systemd-networkd` to re-read its configuration for one interface.
    Reconfigure(String),
    /// Write the credentials of a known Wi-Fi network, so that it can be joined.
    JoinWifi {
        /// The network's name, as bytes on the wire and as text here.
        ssid: String,
        /// The passphrase, which is written to the credential file and never to
        /// an argument vector.
        passphrase: String,
    },
    /// Remove a network's stored credentials.
    ForgetWifi(String),
}

impl NetAction {
    /// The word the protocol and the reply use.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::SetLink { .. } => "link",
            Self::Reconfigure(_) => "reconfigure",
            Self::JoinWifi { .. } => "wifi-join",
            Self::ForgetWifi(_) => "wifi-forget",
        }
    }

    /// The capability that authorises it.
    ///
    /// Scoped to the interface or the network it names, so a grant to bring one
    /// link down is not a grant to join any network. `wifi://` with no path is
    /// the scope that covers every network, and it is spelled that way on
    /// purpose (see `Scope::parse_resource`).
    #[must_use]
    pub fn capability(&self) -> Option<Capability> {
        let raw = match self {
            Self::SetLink { iface, .. } | Self::Reconfigure(iface) => format!("iface://{iface}"),
            Self::JoinWifi { ssid, .. } | Self::ForgetWifi(ssid) => format!("wifi://{ssid}"),
        };
        let scope = Scope::parse_resource(&raw).ok()?;
        Capability::new(CapabilityId::NetConfigure, scope)
    }

    /// The state directory `iwd` keeps credentials in, under `root`.
    #[must_use]
    pub fn iwd_dir(root: &Path) -> PathBuf {
        root.join(IWD_STATE_DIR)
    }

    /// The credential file for a network, under `root`.
    ///
    /// `None` when the name could not be written to a file at all, which the
    /// caller has already refused by then; returning `None` rather than a path
    /// keeps the "no path is built from an unvalidated name" rule visible at the
    /// one place a path is built.
    #[must_use]
    pub fn credential_path(root: &Path, ssid: &str) -> Option<PathBuf> {
        if !valid_ssid(ssid.as_bytes()) {
            return None;
        }
        Some(Self::iwd_dir(root).join(format!("{ssid}.psk")))
    }
}

/// A Bluetooth change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtAction {
    /// Turn the radio on or off.
    Power(bool),
    /// Start or stop discovery.
    Scan(bool),
    /// Pair with a device.
    Pair(String),
    /// Connect to a paired device.
    Connect(String),
    /// Remove a device's pairing.
    Forget(String),
}

impl BtAction {
    /// The word the protocol and the reply use.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Power(_) => "power",
            Self::Scan(_) => "scan",
            Self::Pair(_) => "pair",
            Self::Connect(_) => "connect",
            Self::Forget(_) => "forget",
        }
    }

    /// The capability that authorises it.
    ///
    /// Radio-level operations are scoped to `bt://` — every Bluetooth resource
    /// on this machine, which is what "turn the radio on" means; a per-device
    /// operation is scoped to the address it names.
    #[must_use]
    pub fn capability(&self) -> Option<Capability> {
        let raw = match self {
            Self::Power(_) | Self::Scan(_) => "bt://".to_owned(),
            Self::Pair(addr) | Self::Connect(addr) | Self::Forget(addr) => {
                format!("bt://{addr}")
            }
        };
        let scope = Scope::parse_resource(&raw).ok()?;
        Capability::new(CapabilityId::BluetoothAdmin, scope)
    }
}

/// An authentication action.
#[derive(Clone, PartialEq, Eq)]
pub enum AuthAction {
    /// Verify a password for an account, through the distribution's helper.
    Verify {
        /// The account.
        user: String,
        /// The password, which goes to the helper's stdin and nowhere else.
        password: String,
    },
    /// Create an account and give it a password.
    Create {
        /// The account.
        user: String,
        /// The password, which goes to the helper's stdin and nowhere else.
        password: String,
        /// The same password again; the two must match.
        confirm: String,
    },
    /// Change an account's password after checking the current one.
    Change {
        /// The account.
        user: String,
        /// The password in use now, checked before anything is written.
        current: String,
        /// The replacement, which goes to the helper's stdin and nowhere else.
        new_password: String,
        /// The replacement again; the two must match.
        confirm: String,
    },
}

impl AuthAction {
    /// The word the protocol and the reply use.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Verify { .. } => "verify",
            Self::Create { .. } => "create",
            Self::Change { .. } => "change",
        }
    }

    /// The account this action names.
    #[must_use]
    pub fn user(&self) -> &str {
        match self {
            Self::Verify { user, .. } | Self::Create { user, .. } | Self::Change { user, .. } => {
                user
            }
        }
    }

    /// The capability that authorises it.
    ///
    /// `AuthSession` is a live-consent capability in the vocabulary, and it
    /// stays one here: the request must carry the confirmation word, because a
    /// verification attempt is the answer to a prompt and a prompt nobody
    /// answered is not a login.
    #[must_use]
    pub fn capability(&self) -> Option<Capability> {
        let user = match self {
            Self::Verify { user, .. } | Self::Create { user, .. } | Self::Change { user, .. } => {
                user
            }
        };
        let scope = Scope::parse_resource(&format!("lock://{user}")).ok()?;
        Capability::new(CapabilityId::AuthSession, scope)
    }
}

/// Every machine action the session can take beyond a power transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemAction {
    /// Sound.
    Audio(AudioAction),
    /// Network configuration.
    Net(NetAction),
    /// Bluetooth.
    Bluetooth(BtAction),
    /// A bounded reload of an installed system service.
    Service(ServiceAction),
    /// Authentication.
    Auth(AuthAction),
    /// Install or remove a Flatpak application.
    Store(StoreAction),
}

impl SystemAction {
    /// The capability that authorises this action.
    ///
    /// `None` only when a scope string that this module built from a validated
    /// value would not parse, which a test asserts cannot happen for every
    /// constructed action.
    #[must_use]
    pub fn capability(&self) -> Option<Capability> {
        match self {
            Self::Audio(a) => a.capability(),
            Self::Net(n) => n.capability(),
            Self::Bluetooth(b) => b.capability(),
            Self::Service(s) => {
                let scope = Scope::parse_resource(s.scope_token()).ok()?;
                Capability::new(CapabilityId::ServiceControl, scope)
            }
            Self::Auth(a) => a.capability(),
            Self::Store(s) => {
                let scope = Scope::parse_resource(&s.scope()?).ok()?;
                Capability::new(CapabilityId::AppManage, scope)
            }
        }
    }

    /// The family word.
    #[must_use]
    pub const fn family(&self) -> &'static str {
        match self {
            Self::Audio(_) => "audio",
            Self::Net(_) => "net",
            Self::Bluetooth(_) => "bluetooth",
            Self::Service(_) => "service",
            Self::Auth(_) => "auth",
            Self::Store(_) => "store",
        }
    }

    /// The sub-word, for the reply line.
    #[must_use]
    pub fn token(&self) -> &'static str {
        match self {
            Self::Audio(a) => a.token(),
            Self::Net(n) => n.token(),
            Self::Bluetooth(b) => b.token(),
            Self::Service(s) => s.token(),
            Self::Auth(a) => a.token(),
            Self::Store(s) => s.token(),
        }
    }

    /// Whether this action needs the caller's report that the user answered a
    /// prompt.
    ///
    /// True for a verification and for nothing else here: `AuthSession` is a
    /// live-consent capability, and the answer to the prompt is the password
    /// itself. The three providers are reversible (a volume, a link, a radio)
    /// and are checked without consent, which is what makes a Settings toggle a
    /// toggle rather than a dialog.
    #[must_use]
    pub const fn requires_consent(&self) -> bool {
        matches!(self, Self::Auth(_) | Self::Service(_) | Self::Store(_))
    }
}

// ---------------------------------------------------------------------------
// the plan
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// redaction
// ---------------------------------------------------------------------------

/// The text a secret is replaced with. One spelling, so a test can search for it.
const REDACTED: &str = "<redacted>";

impl fmt::Debug for NetAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetLink { iface, up } => f
                .debug_struct("SetLink")
                .field("iface", iface)
                .field("up", up)
                .finish(),
            Self::Reconfigure(iface) => f.debug_tuple("Reconfigure").field(iface).finish(),
            // A passphrase is never rendered, not even by a `{:?}` somebody adds
            // later for a diagnosis. The field stays visible so a reader can see
            // that one exists and was removed.
            Self::JoinWifi { ssid, .. } => f
                .debug_struct("JoinWifi")
                .field("ssid", ssid)
                .field("passphrase", &REDACTED)
                .finish(),
            Self::ForgetWifi(ssid) => f.debug_tuple("ForgetWifi").field(ssid).finish(),
        }
    }
}

impl fmt::Debug for AuthAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verify { user, .. } => f
                .debug_struct("Verify")
                .field("user", user)
                .field("password", &REDACTED)
                .finish(),
            Self::Create { user, .. } => f
                .debug_struct("Create")
                .field("user", user)
                .field("password", &REDACTED)
                .field("confirm", &REDACTED)
                .finish(),
            Self::Change { user, .. } => f
                .debug_struct("Change")
                .field("user", user)
                .field("current", &REDACTED)
                .field("new_password", &REDACTED)
                .field("confirm", &REDACTED)
                .finish(),
        }
    }
}

impl fmt::Debug for FileChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write {
                path,
                contents,
                private,
            } => {
                // Only a file the plan itself marked private is redacted: a
                // networkd drop-in is not a secret, and hiding it would make the
                // Debug output less useful than the write it describes.
                let shown = if *private {
                    REDACTED
                } else {
                    contents.as_str()
                };
                f.debug_struct("Write")
                    .field("path", path)
                    .field("contents", &shown)
                    .field("private", private)
                    .finish()
            }
            Self::Remove { path } => f.debug_struct("Remove").field("path", path).finish(),
        }
    }
}

impl fmt::Debug for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `stdin` carries a password and the files may carry a passphrase; both
        // are reported as present, never as content.
        let stdin = self.stdin.as_ref().map(|_| REDACTED);
        f.debug_struct("Plan")
            .field("program", &self.program)
            .field("args", &self.args)
            .field("stdin", &stdin)
            .field("files", &self.files)
            .field("notes", &self.notes)
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

/// A file an action changes.
///
/// Described rather than done, so that a test can assert the exact contents and
/// the exact mode without a filesystem, and so that the whole set can be applied
/// in one place with one error path.
#[derive(Clone, PartialEq, Eq)]
pub enum FileChange {
    /// Create or replace a file.
    Write {
        /// Where.
        path: PathBuf,
        /// What.
        contents: String,
        /// Whether the file is written `0600`.
        ///
        /// True for anything holding a credential. A world-readable Wi-Fi
        /// passphrase is the defect this flag exists to make impossible to
        /// introduce by forgetting something.
        private: bool,
    },
    /// Remove a file.
    Remove {
        /// Where.
        path: PathBuf,
    },
}

/// What an action will do, decided before it does anything.
#[derive(Clone, PartialEq, Eq)]
pub struct Plan {
    /// The absolute path of the program to run.
    pub program: &'static str,
    /// Its argument vector, in order.
    pub args: Vec<String>,
    /// What to write to its standard input, if anything. A password goes here
    /// and nowhere else.
    pub stdin: Option<Vec<u8>>,
    /// Files to change before the program runs.
    pub files: Vec<FileChange>,
    /// Extra reply lines, with no secret in them.
    pub notes: Vec<String>,
    /// How long the program is given before it is killed.
    ///
    /// Per action rather than one global number, because the honest answers
    /// differ: a volume change is a local socket round trip, while asking a
    /// device to pair and waiting for the tool to come back is not. Nothing here
    /// is unbounded — the first booted image that ran these actions stopped
    /// mid-report because `bluetoothctl` waited forever for a D-Bus name that
    /// never appeared (`docs/GATES.md`, 2026-10-12, records it as the defect it
    /// was), and an unbounded wait inside a privileged daemon is worse than a
    /// refused action.
    pub timeout_secs: u32,
}

impl Plan {
    /// The reply's `command=` line: the program, and the arguments that are not
    /// secrets.
    ///
    /// A plan carries no secret in its arguments today — the passphrase goes to
    /// a file and the password to standard input — and this is the function that
    /// keeps it that way: the reply is built from here, so an argument vector
    /// that ever grew a secret would have to be added here on purpose.
    #[must_use]
    pub fn command_line(&self) -> String {
        let mut out = String::from(self.program);
        for a in &self.args {
            out.push(' ');
            out.push_str(a);
        }
        out
    }
}

impl SystemAction {
    /// Turn the action into a plan, or refuse it before anything happens.
    ///
    /// # Errors
    /// [`Refusal`] naming the rule that was broken, or the implementation this
    /// build does not have.
    pub fn plan(&self, root: &Path) -> Result<Plan, Refusal> {
        match self {
            Self::Audio(a) => Ok(Self::audio_plan(*a)),
            Self::Net(n) => Self::net_plan(n, root),
            Self::Bluetooth(b) => Self::bt_plan(b),
            Self::Service(s) => s.plan(),
            Self::Auth(a) => Self::auth_plan(a),
            Self::Store(s) => s.plan(),
        }
    }

    fn audio_plan(action: AudioAction) -> Plan {
        match action {
            AudioAction::Volume(percent) => {
                let clamped = percent.min(100);
                // `--limit 1.0` is not optional: without it WirePlumber allows
                // amplification past 100%, and a slider that can reach 150% is a
                // slider that makes the speakers crackle for a reason the user
                // did not ask for.
                Plan {
                    program: WPCTL,
                    args: vec![
                        "set-volume".to_owned(),
                        "--limit".to_owned(),
                        "1.0".to_owned(),
                        "@DEFAULT_AUDIO_SINK@".to_owned(),
                        format!("{}.{:02}", clamped.saturating_div(100), clamped % 100),
                    ],
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![format!("percent={clamped}")],
                    timeout_secs: SOCKET_ROUND_TRIP_SECS,
                }
            }
            AudioAction::Mute(on) => Plan {
                program: WPCTL,
                args: vec![
                    "set-mute".to_owned(),
                    "@DEFAULT_AUDIO_SINK@".to_owned(),
                    if on { "1" } else { "0" }.to_owned(),
                ],
                stdin: None,
                files: Vec::new(),
                notes: vec![format!("muted={on}")],
                timeout_secs: SOCKET_ROUND_TRIP_SECS,
            },
        }
    }

    fn net_plan(action: &NetAction, root: &Path) -> Result<Plan, Refusal> {
        match action {
            NetAction::SetLink { iface, up } => {
                if !valid_iface(iface) {
                    return Err(Refusal::illegal(
                        "interface name",
                        "is not a name the kernel would give an interface",
                    ));
                }
                Ok(Plan {
                    // `ip`, not `networkctl`: networkd manages addressing, but
                    // whether a link is administratively up is the kernel's, and
                    // `ip link set` is the interface that owns it.
                    program: IP,
                    args: vec![
                        "link".to_owned(),
                        "set".to_owned(),
                        iface.clone(),
                        if *up { "up" } else { "down" }.to_owned(),
                    ],
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![format!("iface={iface}"), format!("up={up}")],
                    timeout_secs: LOCAL_CHANGE_SECS,
                })
            }
            NetAction::Reconfigure(iface) => {
                if !valid_iface(iface) {
                    return Err(Refusal::illegal(
                        "interface name",
                        "is not a name the kernel would give an interface",
                    ));
                }
                Ok(Plan {
                    program: NETWORKCTL,
                    args: vec!["reconfigure".to_owned(), iface.clone()],
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![format!("iface={iface}")],
                    timeout_secs: LOCAL_CHANGE_SECS,
                })
            }
            NetAction::JoinWifi { ssid, passphrase } => {
                let bytes = ssid.as_bytes();
                if !valid_ssid(bytes) {
                    return Err(Refusal::illegal(
                        "network name",
                        "is not one this build can store a credential for",
                    ));
                }
                if !valid_passphrase(passphrase.as_bytes()) {
                    return Err(Refusal::illegal(
                        "passphrase",
                        "is not 8 to 63 printable ASCII characters",
                    ));
                }
                let Some(path) = NetAction::credential_path(root, ssid) else {
                    return Err(Refusal::illegal(
                        "network name",
                        "is not one this build can store a credential for",
                    ));
                };
                // iwd reads either a pre-shared key or a passphrase from this
                // file, and a passphrase is what a person typed. Writing the
                // derived key would mean implementing PBKDF2 here to duplicate
                // work the provider already does.
                let contents = format!("[Security]\nPassphrase={passphrase}\n");
                Ok(Plan {
                    program: IWCTL,
                    args: vec![
                        "station".to_owned(),
                        // `iwd` names a station by its interface. There is one
                        // wireless interface on every machine this build has been
                        // run on, and asking for it by name would need the facts
                        // reader here; the note in §8 records the limitation.
                        first_wireless(root).unwrap_or_else(|| "wlan0".to_owned()),
                        "connect".to_owned(),
                        ssid.clone(),
                    ],
                    stdin: None,
                    files: vec![FileChange::Write {
                        path,
                        contents,
                        private: true,
                    }],
                    notes: vec![format!("ssid={ssid}"), "credential=written".to_owned()],
                    // A join waits for an association, and how long that takes is
                    // the radio's business rather than this daemon's.
                    timeout_secs: RADIO_OPERATION_SECS,
                })
            }
            NetAction::ForgetWifi(ssid) => {
                if !valid_ssid(ssid.as_bytes()) {
                    return Err(Refusal::illegal(
                        "network name",
                        "is not one this build can store a credential for",
                    ));
                }
                let Some(path) = NetAction::credential_path(root, ssid) else {
                    return Err(Refusal::illegal(
                        "network name",
                        "is not one this build can store a credential for",
                    ));
                };
                Ok(Plan {
                    program: NETWORKCTL,
                    args: vec!["reload".to_owned()],
                    stdin: None,
                    files: vec![FileChange::Remove { path }],
                    notes: vec![format!("ssid={ssid}"), "credential=removed".to_owned()],
                    timeout_secs: LOCAL_CHANGE_SECS,
                })
            }
        }
    }

    fn bt_plan(action: &BtAction) -> Result<Plan, Refusal> {
        let addr = |a: &str| -> Result<String, Refusal> {
            if valid_bdaddr(a) {
                Ok(a.to_ascii_lowercase())
            } else {
                Err(Refusal::illegal(
                    "Bluetooth address",
                    "is not six colon-separated hex pairs",
                ))
            }
        };
        let (args, notes) = match action {
            BtAction::Power(on) => (
                vec![
                    "power".to_owned(),
                    if *on { "on" } else { "off" }.to_owned(),
                ],
                vec![format!("radio={on}")],
            ),
            BtAction::Scan(on) => (
                vec!["scan".to_owned(), if *on { "on" } else { "off" }.to_owned()],
                vec![format!("scanning={on}")],
            ),
            BtAction::Pair(a) => (
                vec!["pair".to_owned(), addr(a)?],
                vec!["paired=requested".to_owned()],
            ),
            BtAction::Connect(a) => (
                vec!["connect".to_owned(), addr(a)?],
                vec!["connected=requested".to_owned()],
            ),
            BtAction::Forget(a) => (
                vec!["remove".to_owned(), addr(a)?],
                vec!["forgotten=requested".to_owned()],
            ),
        };
        // A pairing and a connection are the two that hide behind a prompt: the
        // tool's exit status says the request was accepted, not that a person at
        // the other end pressed the button. The reply says `requested`, so a UI
        // that showed "paired" from this line would be inventing a fact — the
        // device's state comes from the next facts read, which is the only thing
        // that knows.
        let mut notes = notes;
        notes.push("state=from-next-read".to_owned());
        // A pairing and a connection can wait for a device at the other end; a
        // radio's own state cannot. Both are bounded — see `Plan::timeout_secs`.
        let timeout_secs = match action {
            BtAction::Pair(_) | BtAction::Connect(_) => RADIO_OPERATION_SECS,
            _ => LOCAL_CHANGE_SECS,
        };
        Ok(Plan {
            program: BLUETOOTHCTL,
            args,
            stdin: None,
            files: Vec::new(),
            notes,
            timeout_secs,
        })
    }

    fn auth_plan(action: &AuthAction) -> Result<Plan, Refusal> {
        let AuthAction::Verify { user, password } = action else {
            // Creation and change are two steps and two plan objects. They are
            // executed by `auth::SystemAuthProvider`, which owns the ordering;
            // this single-plan path is for the one-step verification only.
            return Err(Refusal::illegal(
                "account change",
                "takes two steps and is not a single plan",
            ));
        };
        if !valid_user(user) {
            return Err(Refusal::illegal(
                "account name",
                "is not a name this build accepts",
            ));
        }
        if password.is_empty() {
            return Err(Refusal::illegal("password", "is empty"));
        }
        Ok(Plan {
            program: UNIX_CHKPWD,
            // `nonull`: a verification of an empty password is not a
            // verification. This is the argument vector `pam_unix` itself uses.
            args: vec![user.clone(), "nonull".to_owned()],
            stdin: Some({
                let mut input = password.as_bytes().to_vec();
                input.push(b'\n');
                input
            }),
            files: Vec::new(),
            notes: vec![format!("user={user}"), "input=stdin".to_owned()],
            timeout_secs: SOCKET_ROUND_TRIP_SECS,
        })
    }
}

/// The first wireless interface under `root`, if any.
///
/// `iwd` names a station by its interface, and the interface is whatever the
/// machine has. A fixture root with no wireless interface answers `None`, and the
/// plan falls back to `wlan0` with the limitation recorded: the fallback makes
/// the request reach `iwctl`, whose own error is then the honest answer rather
/// than a refusal this daemon invented.
fn first_wireless(root: &Path) -> Option<String> {
    let dir = root.join("sys/class/net");
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if valid_iface(&name) && entry.path().join("wireless").is_dir() {
            return Some(name);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// applying a plan
// ---------------------------------------------------------------------------

/// Apply one file change.
///
/// `root` is the filesystem root the daemon was configured with, and it is
/// used for one thing: **a removal is refused unless the path is directly
/// inside the credential directory this daemon builds paths in.** This process
/// runs as root, so a deletion here has the largest blast radius in the tree
/// and the smallest test coverage, and the bound on it belongs on the path of
/// every removal rather than only in the planner that built the path. The
/// planner already validates the network name (`valid_ssid` refuses `/`, `\`
/// and a leading `.`), so this is the second, independent check: a future plan
/// that removes something else fails here, by name, instead of deleting it.
/// `ci/lint-file-safety.py` requires this site to be reviewed; this is the
/// review, expressed as code.
fn apply_file(change: &FileChange, root: &Path) -> std::io::Result<()> {
    match change {
        FileChange::Write {
            path,
            contents,
            private,
        } => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, contents)?;
            if *private {
                set_private(path)?;
            }
            Ok(())
        }
        FileChange::Remove { path } => {
            let allowed = NetAction::iwd_dir(root);
            if path.parent() != Some(allowed.as_path()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "refusing to remove {}: it is not a credential under {}",
                        path.display(),
                        allowed.display()
                    ),
                ));
            }
            match fs::remove_file(path) {
                Ok(()) => Ok(()),
                // A credential file that is not there is the state the caller
                // asked for, so forgetting an unknown network is not a failure.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            }
        }
    }
}

/// Make a file readable only by its owner.
#[cfg(unix)]
fn set_private(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, mode)
}

/// Nothing to do off Unix, where the daemon does not run.
#[cfg(not(unix))]
fn set_private(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Carry out an action: write what it writes, then run what it runs.
///
/// `root` is the filesystem root the daemon was configured with — `/` in
/// production, a temporary directory in tests — and is the same seam
/// [`crate::platform`] reads through.
///
/// The order matters and is not an accident: a Wi-Fi credential is written
/// *before* `iwctl` is asked to connect, because the tool reads it. An action
/// whose file change fails never runs its program — the tool would then fail in
/// its own words, which is a worse answer than the one this function can give.
///
/// Every reply line here is one of: what was asked, what would be run, the
/// action's own notes, and the verdict. No line carries a secret, and the tests
/// assert that by searching the encoded reply for the passphrase and the
/// password.
#[must_use]
pub fn perform(action: &SystemAction, runner: &dyn Runner, root: &Path) -> Outcome {
    let plan = match action.plan(root) {
        Ok(p) => p,
        Err(refusal) => {
            // Nothing was planned, so the reply names no command: there is no
            // argument vector to report, and inventing one would be describing
            // work that was refused.
            return Outcome {
                applied: false,
                lines: vec![
                    format!("action={}.{}", action.family(), action.token()),
                    "applied=false".to_owned(),
                    format!("reason={}", refusal.token),
                    format!("detail={}", refusal.detail),
                ],
            };
        }
    };

    let lines = describe(action, &plan);
    for change in &plan.files {
        if let Err(e) = apply_file(change, root) {
            return finish(lines, false, "file", &e.to_string());
        }
    }

    let args: Vec<&str> = plan.args.iter().map(String::as_str).collect();
    let result = runner.run(
        plan.program,
        &args,
        plan.stdin.as_deref(),
        plan.timeout_secs,
    );
    match result {
        Ok(0) => finish(lines, true, "", ""),
        Ok(code) => finish(
            lines,
            false,
            "exit",
            &format!("{} exited {code}", plan.program),
        ),
        // A tool that had to be killed is its own diagnosis. It is *not* an exit
        // status, because a caller reading "exit 143" (what a killed process
        // reports through a shell) would not know whether the tool refused or
        // hung, and those two need different answers from a person.
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
            finish(lines, false, "timeout", &e.to_string())
        }
        Err(e) => finish(lines, false, "spawn-failed", &e.to_string()),
    }
}

/// The command that asks the sound server what the default sink is set to.
///
/// A question, not an action: it changes nothing, needs no capability and writes
/// no file. It is here rather than in `platform::audio` because the sound server
/// has no file interface — the answer comes from WirePlumber's own client,
/// exactly as the changes do.
#[must_use]
pub fn audio_level_plan() -> Plan {
    Plan {
        program: WPCTL,
        args: vec!["get-volume".to_owned(), "@DEFAULT_AUDIO_SINK@".to_owned()],
        stdin: None,
        files: Vec::new(),
        notes: vec!["query=audio.level".to_owned()],
        timeout_secs: SOCKET_ROUND_TRIP_SECS,
    }
}

/// What `wpctl get-volume` prints, read as a percentage and a mute flag.
///
/// One shape, from WirePlumber's own client: `Volume: 0.40`, and `Volume: 0.40
/// [MUTED]` when the sink is muted. A line that does not carry a number this can
/// read is `None` — an unreadable answer is not "volume zero". A level above 1.0
/// is clamped, because the action that sets one is limited to 1.0 and a reading
/// past it would be a value this daemon could not produce.
#[must_use]
pub fn parse_audio_level(printed: &str) -> Option<(u8, bool)> {
    let line = printed
        .lines()
        .find(|l| l.trim_start().starts_with("Volume:"))?;
    let number = line.split_whitespace().nth(1)?;
    let fraction: f32 = number.parse().ok()?;
    if !fraction.is_finite() || fraction < 0.0 {
        return None;
    }
    let percent = (fraction * 100.0).round().clamp(0.0, 100.0);
    let muted = line.contains("[MUTED]");
    Some((percent as u8, muted))
}

/// Ask the sound server where the volume is.
///
/// # Errors
/// A sentence naming what happened: no control tool in this image, a tool that
/// failed, or output that is not a level. The daemon turns this into a reply the
/// interface can show; it never guesses a number.
pub fn audio_level(runner: &dyn Runner) -> Result<(u8, bool), String> {
    let plan = audio_level_plan();
    let args: Vec<&str> = plan.args.iter().map(String::as_str).collect();
    let (code, printed) = runner
        .capture(plan.program, &args, plan.timeout_secs)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                format!("{} did not answer in time", plan.program)
            } else {
                format!("{} could not be run: {e}", plan.program)
            }
        })?;
    if code != 0 {
        return Err(format!("{} exited {code}", plan.program));
    }
    parse_audio_level(&printed).ok_or_else(|| {
        format!(
            "{} answered with something that is not a level: {}",
            plan.program,
            printed.trim()
        )
    })
}

/// The command that asks the Bluetooth daemon whether the radio is on.
///
/// A question, not an action, on the same terms as [`audio_level_plan`]: it
/// changes nothing, needs no capability and writes no file. It is here because
/// the radio's power state is **not** in sysfs -- `platform::bluetooth` reads
/// adapters and rfkill, neither of which changes when a controller is powered
/// off -- so without asking BlueZ the interface could only show the state it
/// last requested, which is a value the client typed rather than the machine.
#[must_use]
pub fn bt_status_plan() -> Plan {
    // `show` rather than `info`: with no argument it names the default
    // controller and its properties, which is exactly the question.
    Plan {
        program: BLUETOOTHCTL,
        args: vec!["show".to_owned()],
        stdin: None,
        files: Vec::new(),
        notes: vec!["query=bt.powered".to_owned()],
        // The same deadline as a sound-server question, and bounded for the
        // reason `docs/GATES.md` records: this tool waits for a D-Bus name that
        // never appears on a machine whose `bluetoothd` is not running.
        timeout_secs: SOCKET_ROUND_TRIP_SECS,
    }
}

/// What `bluetoothctl show` prints, read as a power state.
///
/// One shape, from BlueZ's own client: a `Powered:` line whose value is `yes` or
/// `no`. Every other answer is `None`, including the `No default controller
/// available` a machine with no radio prints -- an unreadable answer is not
/// "powered off", and a panel that showed one would be describing a radio that
/// is not there.
#[must_use]
pub fn parse_bt_powered(printed: &str) -> Option<bool> {
    let line = printed
        .lines()
        .find(|l| l.trim_start().starts_with("Powered:"))?;
    match line.split_once(':')?.1.trim() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Ask the Bluetooth daemon whether the default controller is powered on.
///
/// # Errors
/// A sentence naming what happened: no client tool in this image, a tool that
/// failed or timed out, or an answer that carries no power state. The daemon
/// turns this into a reply the interface can show; it never guesses.
pub fn bt_powered(runner: &dyn Runner) -> Result<bool, String> {
    let plan = bt_status_plan();
    let args: Vec<&str> = plan.args.iter().map(String::as_str).collect();
    let (code, printed) = runner
        .capture(plan.program, &args, plan.timeout_secs)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                format!("{} did not answer in time", plan.program)
            } else {
                format!("{} could not be run: {e}", plan.program)
            }
        })?;
    if code != 0 {
        return Err(format!("{} exited {code}", plan.program));
    }
    parse_bt_powered(&printed).ok_or_else(|| {
        format!(
            "{} answered with something that is not a power state: {}",
            plan.program,
            printed.trim()
        )
    })
}

/// The reply lines that describe the plan, without the verdict.
fn describe(action: &SystemAction, plan: &Plan) -> Vec<String> {
    let mut lines = vec![
        format!("action={}.{}", action.family(), action.token()),
        format!("command={}", plan.command_line()),
    ];
    lines.extend(plan.notes.iter().cloned());
    lines
}

/// Add the verdict to a reply.
fn finish(mut lines: Vec<String>, applied: bool, reason: &str, detail: &str) -> Outcome {
    lines.push(format!("applied={applied}"));
    if !applied {
        lines.push(format!("reason={reason}"));
        lines.push(format!("detail={detail}"));
    }
    Outcome { applied, lines }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::testing::RecordingRunner;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("litric-providers-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a temporary directory");
        dir
    }

    fn line<'a>(out: &'a Outcome, prefix: &str) -> Option<&'a String> {
        out.lines.iter().find(|l| l.starts_with(prefix))
    }

    #[test]
    fn a_level_is_read_from_what_wireplumber_prints() {
        assert_eq!(parse_audio_level("Volume: 0.40\n"), Some((40, false)));
        assert_eq!(parse_audio_level("Volume: 1.00\n"), Some((100, false)));
        assert_eq!(parse_audio_level("Volume: 0.00\n"), Some((0, false)));
        assert_eq!(
            parse_audio_level("Volume: 0.75 [MUTED]\n"),
            Some((75, true)),
            "the flag is on the same line"
        );
        // Unreadable is not zero: a level nobody could read must not be shown
        // as a mute machine.
        assert_eq!(parse_audio_level(""), None);
        assert_eq!(parse_audio_level("Node: sink\n"), None);
        assert_eq!(parse_audio_level("Volume: loud\n"), None);
        assert_eq!(parse_audio_level("Volume: -0.5\n"), None);
        // Amplification past the limit the setter enforces is clamped, not
        // reported as a value this daemon could not have produced.
        assert_eq!(parse_audio_level("Volume: 1.50\n"), Some((100, false)));
        assert_eq!(parse_audio_level("Volume: 0.333\n"), Some((33, false)));
    }

    #[test]
    fn reading_the_level_asks_the_tool_and_reports_its_answers() {
        let runner = RecordingRunner::succeeding(0).answering(0, "Volume: 0.35\n");
        assert_eq!(audio_level(&runner), Ok((35, false)));
        assert_eq!(
            runner.calls(),
            vec![(
                WPCTL.to_owned(),
                vec!["get-volume".to_owned(), "@DEFAULT_AUDIO_SINK@".to_owned()]
            )],
            "the question is the one WirePlumber's client answers"
        );

        // A tool that is not installed: the sentence names it, and no number is
        // invented.
        let missing = RecordingRunner::failing(std::io::ErrorKind::NotFound, "no such file");
        assert!(audio_level(&missing).is_err_and(|e| e.contains(WPCTL)));

        // A tool that ran and failed is its own answer.
        let failed = RecordingRunner::succeeding(0).answering(1, "");
        assert!(audio_level(&failed).is_err_and(|e| e.contains("exited 1")));

        // A tool that answered with prose is not a measurement.
        let prose = RecordingRunner::succeeding(0).answering(0, "there is no default sink");
        assert!(audio_level(&prose).is_err_and(|e| e.contains("not a level")));
    }

    #[test]
    fn a_power_state_is_read_from_what_bluetoothctl_prints() {
        // The shape `bluetoothctl show` gives on a machine with a controller.
        let yes = "Controller 00:1A:7D:DA:71:13 (public)\n\tName: litric\n\tAlias: litric\n\tPowered: yes\n\tDiscoverable: no\n";
        assert_eq!(parse_bt_powered(yes), Some(true));
        assert_eq!(parse_bt_powered(&yes.replace("yes", "no")), Some(false));
        // Two spaces after the colon is what some releases print.
        assert_eq!(parse_bt_powered("  Powered:  yes\n"), Some(true));

        // A machine with no radio says so, and that is not "powered off": a
        // panel showing `off` there would describe a controller that is not
        // there.
        assert_eq!(parse_bt_powered("No default controller available\n"), None);
        assert_eq!(parse_bt_powered(""), None);
        // A value that is neither yes nor no is not a state this build reads.
        assert_eq!(parse_bt_powered("Powered: maybe\n"), None);
        assert_eq!(parse_bt_powered("Powered:\n"), None);
        assert_eq!(parse_bt_powered("Discoverable: no\n"), None);
    }

    const SENTINEL: &str = "S3NT1NEL-passphrase-not-in-logs";

    #[test]
    fn a_secret_never_appears_in_a_debug_rendering() {
        let action = NetAction::JoinWifi {
            ssid: "litric-test".to_owned(),
            passphrase: SENTINEL.to_owned(),
        };
        let rendered = format!("{action:?}");
        assert!(
            !rendered.contains(SENTINEL),
            "network Debug leaked the passphrase: {rendered}"
        );

        let auth = AuthAction::Verify {
            user: "alice".to_owned(),
            password: SENTINEL.to_owned(),
        };
        let rendered = format!("{auth:?}");
        assert!(
            !rendered.contains(SENTINEL),
            "auth Debug leaked the password: {rendered}"
        );

        let root = Path::new("/tmp/litric-provider-redaction");
        let plan = SystemAction::Net(action.clone())
            .plan(root)
            .expect("a valid plan");
        for rendered in [
            format!("{plan:?}"),
            format!("{:?}", SystemAction::Net(action)),
            plan.command_line(),
        ] {
            assert!(
                !rendered.contains(SENTINEL),
                "a rendering leaked the passphrase: {rendered}"
            );
        }

        // Redaction must not mean refusal: the credential is still written, and
        // the private flag is still set. Hiding the secret from a log is the bug
        // fix; hiding it from the file would be a different, worse bug.
        let mut wrote = false;
        for change in &plan.files {
            if let FileChange::Write {
                contents, private, ..
            } = change
            {
                assert!(*private, "a credential file must be private");
                assert!(
                    contents.contains(SENTINEL),
                    "the credential must still reach the file"
                );
                let rendered = format!("{change:?}");
                assert!(
                    !rendered.contains(SENTINEL),
                    "FileChange Debug leaked the passphrase: {rendered}"
                );
                wrote = true;
            }
        }
        assert!(wrote, "the join plan must write a credential file");
    }

    #[test]
    fn reading_the_radio_asks_the_tool_and_reports_its_answers() {
        let runner = RecordingRunner::succeeding(0)
            .answering(0, "Controller 00:1A:7D:DA:71:13 (public)\n\tPowered: no\n");
        assert_eq!(bt_powered(&runner), Ok(false));
        assert_eq!(
            runner.calls(),
            vec![(BLUETOOTHCTL.to_owned(), vec!["show".to_owned()])],
            "the question is the one BlueZ's own client answers, and it is asked with no arguments that could change anything"
        );

        let missing = RecordingRunner::failing(std::io::ErrorKind::NotFound, "no such file");
        assert!(bt_powered(&missing).is_err_and(|e| e.contains(BLUETOOTHCTL)));

        let failed = RecordingRunner::succeeding(0).answering(1, "");
        assert!(bt_powered(&failed).is_err_and(|e| e.contains("exited 1")));

        let prose = RecordingRunner::succeeding(0).answering(0, "No default controller available");
        assert!(bt_powered(&prose).is_err_and(|e| e.contains("not a power state")));

        // A tool that never answers must not hold the daemon: the deadline is
        // the plan's own, and a timeout is reported as one.
        let stuck = RecordingRunner::failing(std::io::ErrorKind::TimedOut, "timed out");
        assert!(bt_powered(&stuck).is_err_and(|e| e.contains("did not answer in time")));
    }

    fn audio_volume(percent: u8) -> SystemAction {
        SystemAction::Audio(AudioAction::Volume(percent))
    }

    // ---- validation -------------------------------------------------------

    #[test]
    fn interface_names_are_checked_against_the_kernels_rules() {
        assert!(valid_iface("eth0"));
        assert!(valid_iface("wlan0"));
        assert!(valid_iface("enp3s0.100"));
        assert!(valid_iface("br-1a2b3c"));
        // A name that begins with a dash would be read as an option.
        assert!(!valid_iface("-eth0"));
        // Sixteen bytes is one more than the kernel allows.
        assert!(!valid_iface("abcdefghijklmnop"));
        assert!(!valid_iface(""));
        assert!(!valid_iface("eth 0"));
        assert!(!valid_iface("eth/0"));
        assert!(!valid_iface("eth\n0"));
    }

    #[test]
    fn account_names_follow_the_distributions_rule() {
        assert!(valid_user("alice"));
        assert!(valid_user("root"));
        assert!(valid_user("user-1_2"));
        assert!(!valid_user("Alice"));
        assert!(!valid_user("1alice"));
        assert!(!valid_user(""));
        assert!(!valid_user("../../etc"));
        assert!(!valid_user("alice bob"));
    }

    #[test]
    fn bluetooth_addresses_are_six_hex_pairs_and_not_all_zero() {
        assert!(valid_bdaddr("AA:BB:CC:DD:EE:FF"));
        assert!(valid_bdaddr("00:11:22:33:44:55"));
        assert!(!valid_bdaddr("00:00:00:00:00:00"));
        assert!(!valid_bdaddr("aa:bb:cc:dd:ee"));
        assert!(!valid_bdaddr("aabbccddeeff"));
        assert!(!valid_bdaddr("aa:bb:cc:dd:ee:fg"));
        assert!(!valid_bdaddr("aa:bb:cc:dd:ee:ff:00"));
        assert!(!valid_bdaddr("--help"));
    }

    #[test]
    fn network_names_are_refused_rather_than_escaped() {
        assert!(valid_ssid(b"Home Wi-Fi"));
        assert!(!valid_ssid(b""));
        assert!(!valid_ssid(b"/etc/passwd"));
        assert!(!valid_ssid(b"../etc"));
        assert!(!valid_ssid(b".hidden"));
        assert!(!valid_ssid(&[0xff, 0xfe]));
        assert!(!valid_ssid(&[b'a'; 33]));
        assert!(!valid_ssid(b"new\nline"));
        // A path is never built from an unvalidated name.
        assert_eq!(
            NetAction::credential_path(Path::new("/"), "../etc/passwd"),
            None
        );
    }

    #[test]
    fn passphrases_are_the_wpa_range_and_ascii_only() {
        assert!(valid_passphrase(b"correct horse battery"));
        assert!(!valid_passphrase(b"short"));
        assert!(!valid_passphrase(&[b'a'; 64]));
        assert!(!valid_passphrase(b"tab\there!"));
    }

    #[test]
    fn hex_decoding_round_trips_and_refuses_anything_else() {
        assert_eq!(unhex("48656c6c6f").as_deref(), Some(&b"Hello"[..]));
        assert_eq!(unhex("0a").as_deref(), Some(&b"\n"[..]));
        assert_eq!(unhex("abc"), None, "odd length");
        assert_eq!(unhex("zz"), None, "not hex");
        assert_eq!(unhex(""), None, "empty");
    }

    // ---- capabilities -----------------------------------------------------

    #[test]
    fn every_action_carries_a_capability_for_its_own_scope() {
        let actions = [
            audio_volume(50),
            SystemAction::Audio(AudioAction::Mute(true)),
            SystemAction::Net(NetAction::SetLink {
                iface: "eth0".to_owned(),
                up: false,
            }),
            SystemAction::Net(NetAction::Reconfigure("eth0".to_owned())),
            SystemAction::Net(NetAction::JoinWifi {
                ssid: "Home".to_owned(),
                passphrase: "correct horse".to_owned(),
            }),
            SystemAction::Net(NetAction::ForgetWifi("Home".to_owned())),
            SystemAction::Bluetooth(BtAction::Power(true)),
            SystemAction::Bluetooth(BtAction::Scan(true)),
            SystemAction::Bluetooth(BtAction::Pair("AA:BB:CC:DD:EE:FF".to_owned())),
            SystemAction::Bluetooth(BtAction::Connect("AA:BB:CC:DD:EE:FF".to_owned())),
            SystemAction::Bluetooth(BtAction::Forget("AA:BB:CC:DD:EE:FF".to_owned())),
            SystemAction::Auth(AuthAction::Verify {
                user: "alice".to_owned(),
                password: "hunter2".to_owned(),
            }),
        ];
        for a in &actions {
            assert!(a.capability().is_some(), "{a:?} has no capability");
        }
    }

    #[test]
    fn a_grant_for_one_interface_does_not_cover_another() {
        let eth = SystemAction::Net(NetAction::SetLink {
            iface: "eth0".to_owned(),
            up: false,
        });
        let wlan = SystemAction::Net(NetAction::SetLink {
            iface: "wlan0".to_owned(),
            up: false,
        });
        let eth_cap = eth.capability().expect("a capability");
        let wlan_cap = wlan.capability().expect("a capability");
        assert!(!eth_cap.authorises(&wlan_cap));
        assert!(eth_cap.authorises(&eth_cap));
    }

    #[test]
    fn verification_and_service_reloads_demand_consent() {
        assert!(SystemAction::Auth(AuthAction::Verify {
            user: "alice".to_owned(),
            password: "hunter2".to_owned(),
        })
        .requires_consent());
        for action in crate::services::ServiceAction::ALL {
            assert!(
                SystemAction::Service(action).requires_consent(),
                "{} is administrative",
                action.token()
            );
        }
        assert!(!audio_volume(50).requires_consent());
        assert!(!SystemAction::Bluetooth(BtAction::Power(false)).requires_consent());
        assert!(!SystemAction::Net(NetAction::SetLink {
            iface: "eth0".to_owned(),
            up: false
        })
        .requires_consent());
    }

    // ---- plans ------------------------------------------------------------

    #[test]
    fn volume_becomes_a_fraction_with_a_ceiling() {
        let root = Path::new("/");
        let plan = audio_volume(40).plan(root).expect("a plan");
        assert_eq!(plan.program, WPCTL);
        assert_eq!(
            plan.args,
            vec![
                "set-volume",
                "--limit",
                "1.0",
                "@DEFAULT_AUDIO_SINK@",
                "0.40"
            ]
        );
        assert!(plan.stdin.is_none());
        // Over 100 is clamped, not passed through to a tool that would amplify.
        let loud = audio_volume(200).plan(root).expect("a plan");
        assert_eq!(
            loud.args.last().map(String::as_str),
            Some("1.00"),
            "{:?}",
            loud.args
        );
    }

    #[test]
    fn mute_is_a_number_because_that_is_what_the_tool_takes() {
        let plan = SystemAction::Audio(AudioAction::Mute(true))
            .plan(Path::new("/"))
            .expect("a plan");
        assert_eq!(plan.args, vec!["set-mute", "@DEFAULT_AUDIO_SINK@", "1"]);
    }

    #[test]
    fn a_link_change_runs_ip_with_a_fixed_argv() {
        let plan = SystemAction::Net(NetAction::SetLink {
            iface: "eth0".to_owned(),
            up: true,
        })
        .plan(Path::new("/"))
        .expect("a plan");
        assert_eq!(plan.program, IP);
        assert_eq!(plan.args, vec!["link", "set", "eth0", "up"]);
    }

    #[test]
    fn an_illegal_interface_never_becomes_a_plan() {
        let refused = SystemAction::Net(NetAction::SetLink {
            iface: "--help".to_owned(),
            up: true,
        })
        .plan(Path::new("/"))
        .expect_err("an option is not an interface");
        assert_eq!(refused.token, "bad-argument");
    }

    #[test]
    fn joining_a_wifi_network_writes_a_private_credential_and_then_connects() {
        let root = temp_root("join");
        let action = SystemAction::Net(NetAction::JoinWifi {
            ssid: "Home Wi-Fi".to_owned(),
            passphrase: "correct horse".to_owned(),
        });
        let plan = action.plan(&root).expect("a plan");
        assert_eq!(plan.program, IWCTL);
        assert_eq!(plan.args, vec!["station", "wlan0", "connect", "Home Wi-Fi"]);
        let change = plan.files.first().expect("a credential is written");
        match change {
            FileChange::Write {
                path,
                contents,
                private,
            } => {
                assert_eq!(path, &root.join("var/lib/iwd/Home Wi-Fi.psk"));
                assert_eq!(contents, "[Security]\nPassphrase=correct horse\n");
                assert!(*private, "a passphrase is not world-readable");
            }
            other => panic!("expected a write, got {other:?}"),
        }
        // The passphrase is not in the argument vector and not in the reply.
        assert!(
            !plan.args.iter().any(|a| a.contains("correct horse")),
            "{:?}",
            plan.args
        );
        assert!(!plan.command_line().contains("correct horse"));
    }

    #[test]
    fn the_wireless_interface_is_found_in_the_roots_sysfs() {
        let root = temp_root("wireless");
        fs::create_dir_all(root.join("sys/class/net/wlan0/wireless")).expect("fixture");
        fs::create_dir_all(root.join("sys/class/net/eth0")).expect("fixture");
        assert_eq!(first_wireless(&root).as_deref(), Some("wlan0"));
        let plan = SystemAction::Net(NetAction::JoinWifi {
            ssid: "Home".to_owned(),
            passphrase: "correct horse".to_owned(),
        })
        .plan(&root)
        .expect("a plan");
        assert_eq!(plan.args.first().map(String::as_str), Some("station"));
        assert_eq!(plan.args.get(1).map(String::as_str), Some("wlan0"));
    }

    #[test]
    fn forgetting_a_network_removes_the_credential() {
        let root = temp_root("forget");
        let stored = NetAction::credential_path(&root, "Home").expect("a path");
        fs::create_dir_all(stored.parent().expect("a parent")).expect("fixture");
        fs::write(&stored, "[Security]\nPassphrase=correct horse\n").expect("fixture");

        let action = SystemAction::Net(NetAction::ForgetWifi("Home".to_owned()));
        let runner = RecordingRunner::succeeding(0);
        let out = perform(&action, &runner, &root);
        assert!(out.applied, "{:?}", out.lines);
        assert!(!stored.exists(), "the credential is gone");
    }

    #[test]
    fn a_pairing_plan_does_not_claim_the_device_is_paired() {
        let plan = SystemAction::Bluetooth(BtAction::Pair("AA:BB:CC:DD:EE:FF".to_owned()))
            .plan(Path::new("/"))
            .expect("a plan");
        assert_eq!(plan.program, BLUETOOTHCTL);
        assert_eq!(plan.args, vec!["pair", "aa:bb:cc:dd:ee:ff"]);
        assert!(
            plan.notes.iter().any(|n| n == "state=from-next-read"),
            "the reply must say the state comes from the next read: {:?}",
            plan.notes
        );
    }

    #[test]
    fn a_radio_grant_covers_every_device_and_a_device_grant_covers_its_own() {
        let radio = SystemAction::Bluetooth(BtAction::Power(true));
        let phone = SystemAction::Bluetooth(BtAction::Connect("AA:BB:CC:DD:EE:01".to_owned()));
        let headset = SystemAction::Bluetooth(BtAction::Connect("AA:BB:CC:DD:EE:02".to_owned()));
        let radio_cap = radio.capability().expect("a capability");
        let phone_cap = phone.capability().expect("a capability");
        let headset_cap = headset.capability().expect("a capability");
        assert!(radio_cap.authorises(&radio_cap));
        // A grant of `bt://` is a grant over every Bluetooth resource, which is
        // what turning the radio on is about; the reverse is what must not
        // hold, or connecting to one device would authorise managing the radio.
        assert!(
            radio_cap.authorises(&phone_cap),
            "all of Bluetooth covers one device"
        );
        assert!(
            !phone_cap.authorises(&headset_cap),
            "one device is not another"
        );
        assert!(
            !phone_cap.authorises(&radio_cap),
            "a device grant is not a radio grant"
        );
    }

    #[test]
    fn a_password_travels_on_standard_input_and_never_on_the_command_line() {
        let plan = SystemAction::Auth(AuthAction::Verify {
            user: "alice".to_owned(),
            password: "hunter2".to_owned(),
        })
        .plan(Path::new("/"))
        .expect("a plan");
        assert_eq!(plan.program, UNIX_CHKPWD);
        assert_eq!(plan.args, vec!["alice", "nonull"]);
        assert_eq!(plan.stdin.as_deref(), Some(&b"hunter2\n"[..]));
        assert_eq!(plan.command_line(), "/usr/bin/unix_chkpwd alice nonull");
        assert!(!plan.command_line().contains("hunter2"));
        assert!(!plan.notes.iter().any(|n| n.contains("hunter2")));
    }

    #[test]
    fn an_empty_password_is_refused_rather_than_verified() {
        let refused = SystemAction::Auth(AuthAction::Verify {
            user: "alice".to_owned(),
            password: String::new(),
        })
        .plan(Path::new("/"))
        .expect_err("an empty password is not a password");
        assert_eq!(refused.token, "bad-argument");
    }

    // ---- performing -------------------------------------------------------

    #[test]
    fn performing_an_audio_change_runs_the_tool_once() {
        let runner = RecordingRunner::succeeding(0);
        let out = perform(&audio_volume(30), &runner, Path::new("/"));
        assert!(out.applied, "{:?}", out.lines);
        assert_eq!(
            runner.calls(),
            vec![(
                WPCTL.to_owned(),
                vec![
                    "set-volume".to_owned(),
                    "--limit".to_owned(),
                    "1.0".to_owned(),
                    "@DEFAULT_AUDIO_SINK@".to_owned(),
                    "0.30".to_owned()
                ]
            )]
        );
        assert_eq!(
            line(&out, "action="),
            Some(&"action=audio.volume".to_owned())
        );
        assert_eq!(line(&out, "applied="), Some(&"applied=true".to_owned()));
    }

    #[test]
    fn a_missing_tool_is_reported_as_a_failure_and_not_as_success() {
        // The state of every machine that does not have the provider: the
        // request is authorised, the tool is not there, and the answer says so
        // instead of reporting a volume change that did not happen.
        let runner = RecordingRunner::failing(std::io::ErrorKind::NotFound, "no such file");
        let out = perform(&audio_volume(30), &runner, Path::new("/"));
        assert!(!out.applied);
        assert_eq!(line(&out, "applied="), Some(&"applied=false".to_owned()));
        assert_eq!(
            line(&out, "reason="),
            Some(&"reason=spawn-failed".to_owned())
        );
    }

    #[test]
    fn a_refused_action_runs_nothing_at_all() {
        let runner = RecordingRunner::succeeding(0);
        let action = SystemAction::Net(NetAction::SetLink {
            iface: "eth/0".to_owned(),
            up: true,
        });
        let out = perform(&action, &runner, Path::new("/"));
        assert!(!out.applied);
        assert!(
            runner.calls().is_empty(),
            "a refused action must not reach a tool: {:?}",
            runner.calls()
        );
        assert_eq!(
            line(&out, "reason="),
            Some(&"reason=bad-argument".to_owned())
        );
    }

    #[test]
    fn a_failed_file_write_stops_before_the_tool_runs() {
        // `iwd` would read a credential that is not there and fail in its own
        // words. The answer this function can give is better, and it is only
        // available if the write is attempted first.
        let root = temp_root("no-write");
        fs::create_dir_all(root.join("var/lib")).expect("fixture");
        // A file where the directory holding the credential has to be.
        fs::write(root.join("var/lib/iwd"), b"not a directory").expect("fixture");

        let action = SystemAction::Net(NetAction::JoinWifi {
            ssid: "Home".to_owned(),
            passphrase: "correct horse".to_owned(),
        });
        let runner = RecordingRunner::succeeding(0);
        let out = perform(&action, &runner, &root);
        assert!(!out.applied, "the credential could not be written");
        assert_eq!(line(&out, "reason="), Some(&"reason=file".to_owned()));
        assert!(
            runner.calls().is_empty(),
            "the tool must not run without its credential: {:?}",
            runner.calls()
        );
    }

    #[test]
    fn the_credential_file_is_written_before_the_tool_is_asked_to_connect() {
        let root = temp_root("order");
        let action = SystemAction::Net(NetAction::JoinWifi {
            ssid: "Home".to_owned(),
            passphrase: "correct horse".to_owned(),
        });
        let runner = RecordingRunner::succeeding(0);
        let out = perform(&action, &runner, &root);
        assert!(out.applied, "{:?}", out.lines);
        let path = root.join("var/lib/iwd/Home.psk");
        assert!(path.exists(), "the credential really was written");
        let written = fs::read_to_string(&path).expect("readable");
        assert_eq!(written, "[Security]\nPassphrase=correct horse\n");
        assert_eq!(
            runner.calls().len(),
            1,
            "the tool ran once, after the write"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).expect("metadata").permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "a passphrase file is 0600");
        }
    }

    #[test]
    fn a_tool_that_had_to_be_killed_is_reported_as_a_timeout() {
        // The defect the first booted image found, as a test: a tool that waits
        // for something that never arrives must not leave the daemon waiting
        // with it, and the reply must say which of the three things happened.
        let runner = RecordingRunner::failing(
            std::io::ErrorKind::TimedOut,
            "bluetoothctl did not exit within 15s",
        );
        let out = perform(
            &SystemAction::Bluetooth(BtAction::Power(true)),
            &runner,
            Path::new("/"),
        );
        assert!(!out.applied, "{:?}", out.lines);
        assert_eq!(line(&out, "reason="), Some(&"reason=timeout".to_owned()));
        assert!(line(&out, "detail=").is_some_and(|d| d.contains("did not exit")));
    }

    #[test]
    fn every_plan_is_bounded_and_the_slow_ones_get_longer() {
        // No action may be given an unbounded wait, and the ones that may have to
        // wait for a radio get more time than the ones that answer from a local
        // socket. Asserted rather than trusted, because a zero here would mean
        // "killed immediately" on one implementation and "no deadline" on
        // another.
        let root = Path::new("/");
        let quick = audio_volume(50).plan(root).expect("a plan");
        assert_eq!(quick.timeout_secs, SOCKET_ROUND_TRIP_SECS);
        let radio = SystemAction::Bluetooth(BtAction::Pair("AA:BB:CC:DD:EE:FF".to_owned()))
            .plan(root)
            .expect("a plan");
        assert_eq!(radio.timeout_secs, RADIO_OPERATION_SECS);
        assert!(quick.timeout_secs > 0 && radio.timeout_secs >= quick.timeout_secs);
        let verified = SystemAction::Auth(AuthAction::Verify {
            user: "alice".to_owned(),
            password: "hunter2".to_owned(),
        })
        .plan(root)
        .expect("a plan");
        assert_eq!(verified.timeout_secs, SOCKET_ROUND_TRIP_SECS);
    }

    #[test]
    fn every_action_passes_its_deadline_to_the_runner() {
        let runner = RecordingRunner::succeeding(0);
        let out = perform(&audio_volume(40), &runner, Path::new("/"));
        assert!(out.applied, "{:?}", out.lines);
        assert_eq!(runner.timeout_secs(), Some(SOCKET_ROUND_TRIP_SECS));
    }

    #[test]
    fn a_failed_verification_is_a_failure_and_not_an_accepted_login() {
        let runner = RecordingRunner::succeeding(7);
        let out = perform(
            &SystemAction::Auth(AuthAction::Verify {
                user: "alice".to_owned(),
                password: "wrong".to_owned(),
            }),
            &runner,
            Path::new("/"),
        );
        assert!(!out.applied, "{:?}", out.lines);
        assert_eq!(line(&out, "reason="), Some(&"reason=exit".to_owned()));
    }

    #[test]
    fn the_password_reaches_the_helper_on_stdin_and_leaves_no_trace_in_the_reply() {
        let runner = RecordingRunner::succeeding(0);
        let out = perform(
            &SystemAction::Auth(AuthAction::Verify {
                user: "alice".to_owned(),
                password: "hunter2".to_owned(),
            }),
            &runner,
            Path::new("/"),
        );
        assert!(out.applied, "{:?}", out.lines);
        // Seven characters plus the newline that ends the helper's input.
        assert_eq!(runner.stdin_len(), Some(8), "the password was fed on stdin");
        assert_eq!(
            runner.calls(),
            vec![(
                UNIX_CHKPWD.to_owned(),
                vec!["alice".to_owned(), "nonull".to_owned()]
            )]
        );
        let reply = out.lines.join("\n");
        assert!(
            !reply.contains("hunter2"),
            "a password must not appear in a reply: {reply}"
        );
    }
}
