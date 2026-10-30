//! Account creation and password change for the privileged daemon.
//!
//! The shell never spawns a credential helper and never keeps a password in an
//! argument vector or an environment block. `providers.rs` records the decision
//! this module follows: no password database, no hash format and no PAM FFI in
//! the tree. Passwords go to the distribution's own helpers on standard input,
//! the same channel `unix_chkpwd` already uses for verification.
//!
//! Scrubbing is best effort. [`overwrite`] zeroes the bytes a buffer holds and
//! [`scrub`] then clears and releases it, but this crate forbids `unsafe` code,
//! so the module cannot promise the compiler or allocator never copied those
//! bytes elsewhere. A hard guarantee needs the `zeroize` crate or a scoped
//! `unsafe` `write_volatile`; both are tree-wide decisions, not ones this module
//! can take on its own.

use std::collections::{BTreeMap, HashMap};

use crate::action::{Runner, SystemRunner};

/// The shortest password this build accepts.
pub const MIN_PASSWORD_CHARS: usize = 12;

/// The longest account name this build accepts, in bytes.
pub const MAX_ACCOUNT_NAME_CHARS: usize = 32;

/// `useradd`, from the distribution.
const USERADD: &str = "/usr/sbin/useradd";

/// `chpasswd`, which reads `name:password` on standard input.
const CHPASSWD: &str = "/usr/sbin/chpasswd";

/// The helper `pam_unix` itself runs, already used by verification.
const UNIX_CHKPWD: &str = "/usr/bin/unix_chkpwd";

/// Why an account request was refused.
///
/// The words are what a reply carries; no variant contains a secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    /// The account name does not match the accepted shape.
    InvalidName,
    /// The password is empty or shorter than [`MIN_PASSWORD_CHARS`].
    WeakPassword,
    /// The confirmation does not match the password.
    PasswordMismatch,
    /// The password is the account name itself.
    PasswordEqualsName,
    /// The account already exists.
    AlreadyExists,
    /// The current password was not accepted.
    VerifyFailed,
    /// The helper could not be started.
    SpawnFailed,
    /// The helper exited non-zero. The payload is its status, never its output.
    NonZeroExit(u8),
    /// Anything the daemon could not classify.
    InternalError,
}

impl AuthError {
    /// A sentence for the reply, with no secret in it.
    #[must_use]
    pub const fn detail(self) -> &'static str {
        match self {
            Self::InvalidName => "the account name is not one this build accepts",
            Self::WeakPassword => "the password is too short",
            Self::PasswordMismatch => "the two passwords do not match",
            Self::PasswordEqualsName => "the password is the account name",
            Self::AlreadyExists => "the account already exists",
            Self::VerifyFailed => "the current password was not accepted",
            Self::SpawnFailed => "the helper could not be started",
            Self::NonZeroExit(_) => "the helper exited with a failure status",
            Self::InternalError => "the operation could not be completed",
        }
    }

    /// The stable word a reply carries for this refusal.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::InvalidName => "invalid-name",
            Self::WeakPassword => "weak-password",
            Self::PasswordMismatch => "password-mismatch",
            Self::PasswordEqualsName => "password-equals-name",
            Self::AlreadyExists => "already-exists",
            Self::VerifyFailed => "verify-failed",
            Self::SpawnFailed => "spawn-failed",
            Self::NonZeroExit(_) => "non-zero-exit",
            Self::InternalError => "internal-error",
        }
    }
}

/// Whether `name` is an account name this build accepts.
///
/// `[a-z_][a-z0-9_-]{0,31}`: lower-case letters, digits, `_` and `-`, never
/// starting with a digit or a dash. The same rule is applied by the shell before
/// it sends anything, and again here, because the daemon does not trust a
/// request that arrived over a socket.
#[must_use]
pub fn valid_account_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_ACCOUNT_NAME_CHARS {
        return false;
    }
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first == b'_') {
        return false;
    }
    bytes.all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
    })
}

/// Validate a name, a password and its confirmation together.
///
/// # Errors
///
/// [`AuthError::InvalidName`], [`AuthError::WeakPassword`],
/// [`AuthError::PasswordMismatch`] or [`AuthError::PasswordEqualsName`].
pub fn validate_password(name: &str, password: &str, confirm: &str) -> Result<(), AuthError> {
    if !valid_account_name(name) {
        return Err(AuthError::InvalidName);
    }
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(AuthError::WeakPassword);
    }
    if password != confirm {
        return Err(AuthError::PasswordMismatch);
    }
    if password == name {
        return Err(AuthError::PasswordEqualsName);
    }
    Ok(())
}

/// How many failed verifications one account may accumulate inside
/// [`VERIFY_WINDOW_SECS`] before it is locked out.
pub const VERIFY_FAILURE_LIMIT: u32 = 5;

/// The window a failure stays counted in, in seconds.
pub const VERIFY_WINDOW_SECS: u64 = 60;

/// How long an account is refused after [`VERIFY_FAILURE_LIMIT`] failures.
pub const VERIFY_LOCKOUT_SECS: u64 = 60;

/// Failed verifications across every account inside one window before the
/// whole verification path is refused.
///
/// The per-account limit alone can be dodged by spreading guesses over many
/// account names. This ceiling bounds the total number of helpers the daemon
/// will spawn for failed guesses, whatever names the caller uses.
pub const VERIFY_GLOBAL_FAILURE_LIMIT: u32 = 20;

/// How long the whole verification path is refused after the global limit.
pub const VERIFY_GLOBAL_LOCKOUT_SECS: u64 = 30;

/// The most account names one daemon remembers at once.
///
/// A caller can send any name it likes, so the table needs a ceiling. Reaching
/// it cannot make the per-account limit disappear: the global ceiling above
/// still applies, and an existing entry is never evicted to make room for a
/// new one.
pub const MAX_TRACKED_ACCOUNTS: usize = 256;

/// Failures and the moment the lockout ends, for one account or for the daemon.
#[derive(Debug, Clone, Copy)]
struct Failures {
    count: u32,
    window_start: u64,
    locked_until: u64,
}

impl Failures {
    /// What this entry contributes to the durable file, or `None` when it can
    /// no longer affect a decision.
    fn exported(&self, user: Option<String>, now: u64, limit: u32) -> Option<LockoutRecord> {
        let locked = self.locked_until > now;
        let window_alive = now.saturating_sub(self.window_start) < VERIFY_WINDOW_SECS;
        if self.count == 0 || (!window_alive && !locked) {
            return None;
        }
        Some(LockoutRecord {
            user,
            count: self.count.min(limit),
            window_start: self.window_start,
            locked_until: if locked { self.locked_until } else { 0 },
        })
    }

    /// Rebuild an entry from a file, refusing values that would turn a damaged
    /// record into a long denial.
    ///
    /// The deadline is clamped to `max_lockout` from *now*, a window start in
    /// the future is treated as now (the clock moved backwards), an expired
    /// window with no live lock is dropped, and a zero count is dropped rather
    /// than locked.
    fn restored(record: LockoutRecord, now: u64, limit: u32, max_lockout: u64) -> Option<Self> {
        let count = record.count.min(limit);
        if count == 0 {
            return None;
        }
        let window_start = record.window_start.min(now);
        let locked_until = if record.locked_until > now {
            record.locked_until.min(now.saturating_add(max_lockout))
        } else {
            0
        };
        let window_alive = now.saturating_sub(window_start) < VERIFY_WINDOW_SECS;
        if !window_alive && locked_until == 0 {
            return None;
        }
        Some(Self {
            count,
            window_start,
            locked_until,
        })
    }

    const fn new(now: u64) -> Self {
        Self {
            count: 0,
            window_start: now,
            locked_until: 0,
        }
    }

    /// Seconds left before this entry is tried again, or `None`.
    fn locked_for(&self, now: u64) -> Option<u64> {
        if now < self.locked_until {
            Some(self.locked_until.saturating_sub(now))
        } else {
            None
        }
    }

    /// Count one failure, starting a fresh window if the previous one is over.
    ///
    /// A failure that arrives while the entry is locked does not extend the
    /// lockout: the caller is refused before a helper runs, so there is nothing
    /// new to count, and an attacker cannot keep an account locked forever by
    /// retrying during the lock.
    fn failed(&mut self, now: u64, limit: u32, lockout: u64) {
        if self.locked_for(now).is_some() {
            return;
        }
        let lock_just_ended = self.locked_until > 0 && now >= self.locked_until;
        if lock_just_ended || now.saturating_sub(self.window_start) >= VERIFY_WINDOW_SECS {
            *self = Self::new(now);
        }
        self.count = self.count.saturating_add(1);
        if self.count >= limit {
            self.locked_until = now.saturating_add(lockout);
        }
    }
}

/// Metering for password verification.
///
/// It holds counters only: no password, no hash and no account data beyond the
/// name the caller sent. Time is supplied by the caller, like the rest of the
/// daemon, so the whole policy is testable without sleeping. The daemon writes
/// [`Throttle::export`] to disk and rebuilds it with [`Throttle::restore`], so a
/// restart does not reset the meter; the file format lives in [`crate::lockout`].
#[derive(Debug)]
pub struct Throttle {
    accounts: HashMap<String, Failures>,
    global: Failures,
}

/// One counter as it is written to disk.
///
/// `user` is `None` for the daemon-wide counter. It is the only text in the
/// file, and it is validated on the way back in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockoutRecord {
    /// The account, or `None` for the daemon-wide counter.
    pub user: Option<String>,
    /// Failed attempts counted in the current window.
    pub count: u32,
    /// When the current window started, in seconds since the epoch.
    pub window_start: u64,
    /// When the lockout ends, or 0 when the entry is not locked.
    pub locked_until: u64,
}

impl Default for Throttle {
    fn default() -> Self {
        Self::new()
    }
}

impl Throttle {
    /// A throttle with no failures recorded.
    #[must_use]
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            global: Failures::new(0),
        }
    }

    /// Seconds the caller must wait before verifying `account`, or `None`.
    ///
    /// The larger of the account's remaining lockout and the daemon-wide one:
    /// the caller is told one number, and it is always enough.
    #[must_use]
    pub fn retry_after(&self, account: &str, now: u64) -> Option<u64> {
        let account_wait = self.accounts.get(account).and_then(|f| f.locked_for(now));
        let global_wait = self.global.locked_for(now);
        match (account_wait, global_wait) {
            (Some(a), Some(g)) => Some(a.max(g)),
            (a, g) => a.or(g),
        }
    }

    /// Record one failed verification of `account`.
    pub fn record_failure(&mut self, account: &str, now: u64) {
        self.global
            .failed(now, VERIFY_GLOBAL_FAILURE_LIMIT, VERIFY_GLOBAL_LOCKOUT_SECS);
        if self.accounts.len() >= MAX_TRACKED_ACCOUNTS && !self.accounts.contains_key(account) {
            self.prune(now);
        }
        if self.accounts.len() >= MAX_TRACKED_ACCOUNTS && !self.accounts.contains_key(account) {
            // Full of live entries. The global limit still bounds guesses, and
            // leaving the existing entries alone is what keeps an attacker from
            // evicting an account's lockout by spraying new names.
            return;
        }
        self.accounts
            .entry(account.to_owned())
            .or_insert_with(|| Failures::new(now))
            .failed(now, VERIFY_FAILURE_LIMIT, VERIFY_LOCKOUT_SECS);
    }

    /// The counters worth writing down, global first and accounts sorted.
    ///
    /// Entries whose window has passed and whose lock has ended are omitted, so
    /// the file shrinks as the meter forgets. The result is bounded by the
    /// throttle's own table: at most one global record and
    /// [`MAX_TRACKED_ACCOUNTS`] accounts.
    #[must_use]
    pub fn export(&self, now: u64) -> Vec<LockoutRecord> {
        let mut out = Vec::new();
        if let Some(global) = self.global.exported(None, now, VERIFY_GLOBAL_FAILURE_LIMIT) {
            out.push(global);
        }
        let mut accounts: Vec<(&String, &Failures)> = self.accounts.iter().collect();
        accounts.sort_by(|a, b| a.0.cmp(b.0));
        for (user, failures) in accounts {
            if let Some(record) = failures.exported(Some(user.clone()), now, VERIFY_FAILURE_LIMIT) {
                out.push(record);
            }
        }
        out
    }

    /// Rebuild a throttle from records read from disk.
    ///
    /// Every value is treated as untrusted: the count cannot exceed the
    /// configured limit, a lockout cannot last longer than the configured one
    /// from `now`, future window starts become `now`, invalid account names are
    /// ignored, and only [`MAX_TRACKED_ACCOUNTS`] entries are accepted. A
    /// damaged file therefore fails open at the first bad value instead of
    /// producing a lockout nobody can clear.
    #[must_use]
    pub fn restore(records: Vec<LockoutRecord>, now: u64) -> Self {
        let mut throttle = Self::new();
        for record in records {
            if let Some(user) = record.user.clone() {
                if !valid_account_name(&user) {
                    continue;
                }
                if throttle.accounts.len() >= MAX_TRACKED_ACCOUNTS {
                    continue;
                }
                if let Some(failures) =
                    Failures::restored(record, now, VERIFY_FAILURE_LIMIT, VERIFY_LOCKOUT_SECS)
                {
                    throttle.accounts.insert(user, failures);
                }
            } else if let Some(failures) = Failures::restored(
                record,
                now,
                VERIFY_GLOBAL_FAILURE_LIMIT,
                VERIFY_GLOBAL_LOCKOUT_SECS,
            ) {
                throttle.global = failures;
            }
        }
        throttle
    }

    /// Forget `account`'s failures after a correct password.
    ///
    /// The daemon-wide counter is deliberately not cleared: one success says
    /// nothing about the guesses that came before it from other names.
    pub fn record_success(&mut self, account: &str) {
        self.accounts.remove(account);
    }

    /// Drop entries that can no longer matter, then make room if needed.
    fn prune(&mut self, now: u64) {
        self.accounts.retain(|_, f| {
            f.locked_for(now).is_some() || now.saturating_sub(f.window_start) < VERIFY_WINDOW_SECS
        });
        if self.accounts.len() < MAX_TRACKED_ACCOUNTS {
            return;
        }
        let victim = self
            .accounts
            .iter()
            .filter(|(_, f)| f.locked_for(now).is_none())
            .min_by_key(|(_, f)| f.window_start)
            .map(|(name, _)| name.clone());
        // The table is full of live entries and none of them is safe to drop:
        // dropping one is how a caller would evict an account's lockout by
        // spraying names. The new name is simply not tracked; the global
        // ceiling already bounds how many guesses can happen anyway.
        let _ = victim;
    }
}

/// One program invocation, with any secret on standard input only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The absolute path of the helper.
    pub program: &'static str,
    /// Its arguments. A password never appears here.
    pub args: Vec<String>,
    /// Standard input, the only place a password travels.
    pub stdin: Option<Vec<u8>>,
}

/// The two steps that create an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePlans {
    /// `useradd`, which creates the account without a password.
    pub useradd: Plan,
    /// `chpasswd`, which reads the password on standard input.
    pub chpasswd: Plan,
}

/// The two steps that change a password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangePlans {
    /// `unix_chkpwd`, which checks the current password first.
    pub verify: Plan,
    /// `chpasswd`, which reads the new password on standard input.
    pub chpasswd: Plan,
}

/// Zero every byte a buffer currently holds.
pub fn overwrite(secret: &mut [u8]) {
    for byte in secret.iter_mut() {
        *byte = 0;
    }
}

/// Zero, clear and release a secret buffer.
///
/// See the module note: without `unsafe` this is best effort, not a guarantee.
pub fn scrub(secret: &mut Vec<u8>) {
    overwrite(secret.as_mut_slice());
    secret.clear();
    secret.shrink_to_fit();
}

/// The `name:password\n` line `chpasswd` expects.
fn chpasswd_input(name: &str, password: &str) -> Vec<u8> {
    let capacity = name.len().saturating_add(password.len()).saturating_add(2);
    let mut input = Vec::with_capacity(capacity);
    input.extend_from_slice(name.as_bytes());
    input.push(b':');
    input.extend_from_slice(password.as_bytes());
    input.push(b'\n');
    input
}

/// The single line `unix_chkpwd` reads for one attempt.
fn chkpwd_input(password: &str) -> Vec<u8> {
    let capacity = password.len().saturating_add(1);
    let mut input = Vec::with_capacity(capacity);
    input.extend_from_slice(password.as_bytes());
    input.push(b'\n');
    input
}

/// Build the two steps that create `name` with `password`.
///
/// # Errors
///
/// [`AuthError::InvalidName`] or [`AuthError::WeakPassword`].
pub fn create_plans(name: &str, password: &str) -> Result<CreatePlans, AuthError> {
    if !valid_account_name(name) {
        return Err(AuthError::InvalidName);
    }
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(AuthError::WeakPassword);
    }
    Ok(CreatePlans {
        useradd: Plan {
            program: USERADD,
            args: vec!["-m".to_owned(), "--".to_owned(), name.to_owned()],
            stdin: None,
        },
        chpasswd: Plan {
            program: CHPASSWD,
            args: Vec::new(),
            stdin: Some(chpasswd_input(name, password)),
        },
    })
}

/// Build the steps that change `name`'s password from `current` to `new`.
///
/// The current password is verified before anything is written, so a mistyped
/// password cannot overwrite the account's credential.
///
/// # Errors
///
/// [`AuthError::InvalidName`], [`AuthError::WeakPassword`] or
/// [`AuthError::PasswordEqualsName`].
pub fn change_plans(name: &str, current: &str, new: &str) -> Result<ChangePlans, AuthError> {
    if !valid_account_name(name) {
        return Err(AuthError::InvalidName);
    }
    if new.chars().count() < MIN_PASSWORD_CHARS {
        return Err(AuthError::WeakPassword);
    }
    if new == name {
        return Err(AuthError::PasswordEqualsName);
    }
    Ok(ChangePlans {
        verify: Plan {
            program: UNIX_CHKPWD,
            args: vec![name.to_owned(), "nonull".to_owned()],
            stdin: Some(chkpwd_input(current)),
        },
        chpasswd: Plan {
            program: CHPASSWD,
            args: Vec::new(),
            stdin: Some(chpasswd_input(name, new)),
        },
    })
}

/// The privileged operations the daemon needs, behind a seam tests can replace.
///
/// Production runs the distribution's helpers; tests use [`FakeAuth`], which
/// keeps accounts in memory. The trait is deliberately free of UI types: the
/// shell talks to the daemon over the protocol, not to this trait.
pub trait AuthProvider: std::fmt::Debug + Send {
    /// Create `name` with `password`.
    ///
    /// # Errors
    ///
    /// Any [`AuthError`] the backend reports.
    fn create(&mut self, name: &str, password: &str) -> Result<(), AuthError>;
    /// Change `name`'s password after verifying `current`.
    ///
    /// # Errors
    ///
    /// [`AuthError::VerifyFailed`] when `current` is wrong, or any other
    /// [`AuthError`] the backend reports.
    fn change(&mut self, name: &str, current: &str, new: &str) -> Result<(), AuthError>;
    /// Check `password` for `name`.
    ///
    /// # Errors
    ///
    /// [`AuthError::VerifyFailed`] when the password is wrong.
    fn verify(&self, name: &str, password: &str) -> Result<(), AuthError>;
}

/// An in-memory [`AuthProvider`] for tests.
///
/// It stores passwords in clear text on purpose: it is a stand-in for the
/// distribution's helpers, never a production backend.
#[derive(Debug, Default)]
pub struct FakeAuth {
    users: BTreeMap<String, String>,
}

impl FakeAuth {
    /// An empty fake.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A fake that already holds `name` with `password`.
    #[must_use]
    pub fn with_user(name: &str, password: &str) -> Self {
        let mut fake = Self::default();
        fake.users.insert(name.to_owned(), password.to_owned());
        fake
    }
}

impl AuthProvider for FakeAuth {
    fn create(&mut self, name: &str, password: &str) -> Result<(), AuthError> {
        if !valid_account_name(name) {
            return Err(AuthError::InvalidName);
        }
        if self.users.contains_key(name) {
            return Err(AuthError::AlreadyExists);
        }
        if password.chars().count() < MIN_PASSWORD_CHARS {
            return Err(AuthError::WeakPassword);
        }
        self.users.insert(name.to_owned(), password.to_owned());
        Ok(())
    }

    fn change(&mut self, name: &str, current: &str, new: &str) -> Result<(), AuthError> {
        self.verify(name, current)?;
        if new.chars().count() < MIN_PASSWORD_CHARS {
            return Err(AuthError::WeakPassword);
        }
        if new == name {
            return Err(AuthError::PasswordEqualsName);
        }
        self.users.insert(name.to_owned(), new.to_owned());
        Ok(())
    }

    fn verify(&self, name: &str, password: &str) -> Result<(), AuthError> {
        match self.users.get(name) {
            Some(stored) if stored == password => Ok(()),
            _ => Err(AuthError::VerifyFailed),
        }
    }
}

/// How long one helper step is given before it is killed.
const AUTH_STEP_TIMEOUT_SECS: u32 = 10;

/// A [`AuthProvider`] that runs the distribution's helpers for real.
///
/// The process seam is [`Runner`], the same one the rest of the daemon uses, so
/// tests can record invocations without root and without touching `/etc`.
#[derive(Debug)]
pub struct SystemAuthProvider<R> {
    runner: R,
}

impl SystemAuthProvider<SystemRunner> {
    /// A provider that spawns real processes.
    #[must_use]
    pub fn system() -> Self {
        Self {
            runner: SystemRunner,
        }
    }
}

impl<R: Runner> SystemAuthProvider<R> {
    /// A provider over a caller-supplied runner.
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }

    /// Run one step, scrubbing its standard input afterwards.
    fn step(&self, plan: &mut Plan) -> Result<(), AuthError> {
        let args: Vec<&str> = plan.args.iter().map(String::as_str).collect();
        let status = self
            .runner
            .run(
                plan.program,
                &args,
                plan.stdin.as_deref(),
                AUTH_STEP_TIMEOUT_SECS,
            )
            .map_err(|_| AuthError::SpawnFailed)?;
        if let Some(input) = plan.stdin.as_mut() {
            scrub(input);
        }
        if status == 0 {
            Ok(())
        } else {
            Err(AuthError::NonZeroExit(
                u8::try_from(status).unwrap_or(u8::MAX),
            ))
        }
    }
}

impl<R: Runner> AuthProvider for SystemAuthProvider<R> {
    fn create(&mut self, name: &str, password: &str) -> Result<(), AuthError> {
        let mut plans = create_plans(name, password)?;
        self.step(&mut plans.useradd)?;
        self.step(&mut plans.chpasswd)
    }

    fn change(&mut self, name: &str, current: &str, new: &str) -> Result<(), AuthError> {
        let mut plans = change_plans(name, current, new)?;
        // A wrong current password is an answer about the attempt, not a daemon
        // failure; the account is untouched and no second step runs.
        self.step(&mut plans.verify).map_err(|error| match error {
            AuthError::NonZeroExit(_) => AuthError::VerifyFailed,
            other => other,
        })?;
        self.step(&mut plans.chpasswd)
    }

    fn verify(&self, name: &str, password: &str) -> Result<(), AuthError> {
        if !valid_account_name(name) {
            return Err(AuthError::InvalidName);
        }
        let mut plan = Plan {
            program: UNIX_CHKPWD,
            args: vec![name.to_owned(), "nonull".to_owned()],
            stdin: Some(chkpwd_input(password)),
        };
        self.step(&mut plan).map_err(|error| match error {
            AuthError::NonZeroExit(_) => AuthError::VerifyFailed,
            other => other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        change_plans, create_plans, overwrite, scrub, valid_account_name, validate_password,
        AuthError, AuthProvider, FakeAuth, MIN_PASSWORD_CHARS,
    };

    const STRONG: &str = "correct horse battery";

    #[test]
    fn account_names_follow_the_stated_shape() {
        for name in ["litric", "a", "_svc", "user-1", "a_b-c9"] {
            assert!(valid_account_name(name), "{name} should be accepted");
        }
        for name in [
            "",
            "Litric",
            "1user",
            "-user",
            "user name",
            "usér",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            assert!(!valid_account_name(name), "{name} should be refused");
        }
    }

    #[test]
    fn passwords_are_validated_in_one_place() {
        let short = "a".repeat(MIN_PASSWORD_CHARS - 1);
        let cases = [
            ("Bad", STRONG, STRONG, Err(AuthError::InvalidName)),
            ("litric", "", "", Err(AuthError::WeakPassword)),
            (
                "litric",
                short.as_str(),
                short.as_str(),
                Err(AuthError::WeakPassword),
            ),
            (
                "litric",
                STRONG,
                "different password",
                Err(AuthError::PasswordMismatch),
            ),
            (
                "abcdefghijkl",
                "abcdefghijkl",
                "abcdefghijkl",
                Err(AuthError::PasswordEqualsName),
            ),
            ("litric", STRONG, STRONG, Ok(())),
        ];
        for (name, password, confirm, expected) in cases {
            assert_eq!(
                validate_password(name, password, confirm),
                expected,
                "{name}/{password}/{confirm}"
            );
        }
    }

    #[test]
    fn create_never_puts_the_password_in_an_argument() {
        let plans = create_plans("litric", STRONG).expect("valid request");
        assert_eq!(plans.useradd.program, "/usr/sbin/useradd");
        assert!(plans.useradd.stdin.is_none());
        for arg in &plans.useradd.args {
            assert!(!arg.contains(STRONG), "argv leaked the password");
        }
        assert_eq!(plans.chpasswd.program, "/usr/sbin/chpasswd");
        assert!(
            plans.chpasswd.args.is_empty(),
            "chpasswd takes no secrets in argv"
        );
        let stdin = plans.chpasswd.stdin.as_ref().expect("stdin");
        assert!(stdin.ends_with(b"\n"));
        assert!(stdin.starts_with(b"litric:"));
        assert!(stdin
            .windows(STRONG.len())
            .any(|window| window == STRONG.as_bytes()));
    }

    #[test]
    fn change_verifies_first_and_keeps_both_passwords_on_stdin() {
        let old = "old password eleven";
        let new = "new password twelve";
        let plans = change_plans("litric", old, new).expect("valid request");
        assert_eq!(plans.verify.program, "/usr/bin/unix_chkpwd");
        assert_eq!(
            plans.verify.args,
            vec!["litric".to_owned(), "nonull".to_owned()]
        );
        for arg in &plans.verify.args {
            assert!(!arg.contains(old) && !arg.contains(new));
        }
        let verify_stdin = plans.verify.stdin.as_ref().expect("stdin");
        assert_eq!(verify_stdin.as_slice(), format!("{old}\n").as_bytes());
        let stdin = plans.chpasswd.stdin.as_ref().expect("stdin");
        assert_eq!(stdin.as_slice(), format!("litric:{new}\n").as_bytes());
    }

    #[test]
    fn a_secret_buffer_is_overwritten_before_it_is_released() {
        let mut secret = STRONG.as_bytes().to_vec();
        overwrite(secret.as_mut_slice());
        assert!(
            secret.iter().all(|byte| *byte == 0),
            "bytes were not zeroed"
        );
        scrub(&mut secret);
        assert!(secret.is_empty());
    }

    #[test]
    fn create_change_and_verify_flow_through_a_fake_backend() {
        let mut auth = FakeAuth::new();
        auth.create("litric", STRONG).expect("create");
        auth.verify("litric", STRONG)
            .expect("the created password works");
        assert_eq!(auth.create("litric", STRONG), Err(AuthError::AlreadyExists));
        assert_eq!(
            auth.change("litric", "wrong password here", "another password"),
            Err(AuthError::VerifyFailed)
        );
        auth.verify("litric", STRONG)
            .expect("a failed change left the old password");
        auth.change("litric", STRONG, "another password 12")
            .expect("change");
        auth.verify("litric", "another password 12")
            .expect("the new password works");
        assert_eq!(auth.verify("litric", STRONG), Err(AuthError::VerifyFailed));
    }

    #[test]
    fn a_duplicate_create_is_refused_without_touching_the_account() {
        let mut auth = FakeAuth::with_user("litric", STRONG);
        assert_eq!(
            auth.create("litric", "another password 12"),
            Err(AuthError::AlreadyExists)
        );
        auth.verify("litric", STRONG)
            .expect("the original password still works");
    }

    type Call = (String, Vec<String>, Option<Vec<u8>>);
    type Script = std::collections::VecDeque<Result<i32, std::io::ErrorKind>>;

    #[derive(Debug, Default)]
    struct RecordingRunner {
        calls: std::sync::Arc<std::sync::Mutex<Vec<Call>>>,
        results: std::sync::Arc<std::sync::Mutex<Script>>,
    }

    impl RecordingRunner {
        fn with(results: Vec<Result<i32, std::io::ErrorKind>>) -> Self {
            Self {
                calls: std::sync::Arc::default(),
                results: std::sync::Arc::new(std::sync::Mutex::new(results.into())),
            }
        }
    }

    impl crate::action::Runner for RecordingRunner {
        fn run(
            &self,
            program: &str,
            args: &[&str],
            stdin: Option<&[u8]>,
            _timeout_secs: u32,
        ) -> std::io::Result<i32> {
            if let Ok(mut calls) = self.calls.lock() {
                calls.push((
                    program.to_owned(),
                    args.iter().map(|arg| (*arg).to_owned()).collect(),
                    stdin.map(<[u8]>::to_vec),
                ));
            }
            let next = self
                .results
                .lock()
                .ok()
                .and_then(|mut results| results.pop_front())
                .unwrap_or(Ok(0));
            match next {
                Ok(code) => Ok(code),
                Err(kind) => Err(std::io::Error::from(kind)),
            }
        }

        fn capture(
            &self,
            program: &str,
            args: &[&str],
            timeout_secs: u32,
        ) -> std::io::Result<(i32, String)> {
            self.run(program, args, None, timeout_secs)
                .map(|code| (code, String::new()))
        }
    }

    #[test]
    fn the_system_provider_creates_in_two_steps_without_a_password_in_argv() {
        let runner = RecordingRunner::with(vec![Ok(0), Ok(0)]);
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut provider = super::SystemAuthProvider::new(runner);
        provider.create("litric", STRONG).expect("create");
        let recorded = calls.lock().expect("calls");
        assert_eq!(recorded.len(), 2);
        let first = recorded.first().cloned().expect("useradd");
        let second = recorded.get(1).cloned().expect("chpasswd");
        assert_eq!(first.0, "/usr/sbin/useradd");
        assert!(!first.1.iter().any(|arg| arg.contains(STRONG)));
        assert_eq!(second.0, "/usr/sbin/chpasswd");
        assert!(!second.1.iter().any(|arg| arg.contains(STRONG)));
        assert_eq!(
            second.2.as_deref(),
            Some(format!("litric:{STRONG}\n").as_bytes())
        );
    }

    #[test]
    fn a_failed_first_step_never_runs_the_second() {
        let runner = RecordingRunner::with(vec![Ok(1)]);
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut provider = super::SystemAuthProvider::new(runner);
        assert_eq!(
            provider.create("litric", STRONG),
            Err(AuthError::NonZeroExit(1))
        );
        assert_eq!(
            calls.lock().expect("calls").len(),
            1,
            "chpasswd must not run"
        );
    }

    #[test]
    fn a_spawn_failure_and_a_bad_current_password_are_distinct() {
        let runner = RecordingRunner::with(vec![Err(std::io::ErrorKind::NotFound)]);
        let mut provider = super::SystemAuthProvider::new(runner);
        assert_eq!(
            provider.create("litric", STRONG),
            Err(AuthError::SpawnFailed)
        );

        let runner = RecordingRunner::with(vec![Ok(1)]);
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut provider = super::SystemAuthProvider::new(runner);
        assert_eq!(
            provider.change("litric", "wrong password here", "another password"),
            Err(AuthError::VerifyFailed)
        );
        assert_eq!(
            calls.lock().expect("calls").len(),
            1,
            "no write after a failed check"
        );
    }

    #[test]
    fn error_words_are_stable_and_carry_no_secret() {
        let words = [
            (AuthError::InvalidName, "invalid-name"),
            (AuthError::WeakPassword, "weak-password"),
            (AuthError::PasswordMismatch, "password-mismatch"),
            (AuthError::PasswordEqualsName, "password-equals-name"),
            (AuthError::AlreadyExists, "already-exists"),
            (AuthError::VerifyFailed, "verify-failed"),
            (AuthError::SpawnFailed, "spawn-failed"),
            (AuthError::NonZeroExit(7), "non-zero-exit"),
            (AuthError::InternalError, "internal-error"),
        ];
        for (error, word) in words {
            assert_eq!(error.token(), word);
            assert!(!error.token().contains(STRONG));
        }
    }
}

#[cfg(test)]
mod throttle_tests {
    use super::*;

    #[test]
    fn five_failures_lock_one_account_for_a_minute() {
        let mut throttle = Throttle::new();
        for attempt in 0..VERIFY_FAILURE_LIMIT {
            assert_eq!(
                throttle.retry_after("alice", 1_000),
                None,
                "attempt {attempt}"
            );
            throttle.record_failure("alice", 1_000);
        }
        assert_eq!(
            throttle.retry_after("alice", 1_000),
            Some(VERIFY_LOCKOUT_SECS)
        );
        assert_eq!(throttle.retry_after("alice", 1_059), Some(1));
        assert_eq!(throttle.retry_after("alice", 1_060), None);
        // Another account is unaffected by alice's lockout.
        assert_eq!(throttle.retry_after("bob", 1_000), None);
    }

    #[test]
    fn a_correct_password_clears_the_failures() {
        let mut throttle = Throttle::new();
        for _ in 0..VERIFY_FAILURE_LIMIT - 1 {
            throttle.record_failure("alice", 2_000);
        }
        throttle.record_success("alice");
        for attempt in 0..VERIFY_FAILURE_LIMIT {
            assert_eq!(
                throttle.retry_after("alice", 2_000),
                None,
                "attempt {attempt}"
            );
            throttle.record_failure("alice", 2_000);
        }
        assert_eq!(
            throttle.retry_after("alice", 2_000),
            Some(VERIFY_LOCKOUT_SECS)
        );
    }

    #[test]
    fn four_failures_export_and_the_next_attempt_locks_after_restore() {
        let mut throttle = Throttle::new();
        for _ in 0..VERIFY_FAILURE_LIMIT.saturating_sub(1) {
            throttle.record_failure("alice", 1_000);
        }
        assert_eq!(throttle.retry_after("alice", 1_000), None);
        let mut restored = Throttle::restore(throttle.export(1_000), 1_000);
        restored.record_failure("alice", 1_000);
        assert_eq!(
            restored.retry_after("alice", 1_000),
            Some(VERIFY_LOCKOUT_SECS)
        );
    }

    #[test]
    fn restore_clamps_a_deadline_and_drops_expired_entries() {
        let now = 10_000;
        let records = vec![
            LockoutRecord {
                user: Some("alice".to_owned()),
                count: 999,
                window_start: now,
                locked_until: now.saturating_add(10_000_000),
            },
            LockoutRecord {
                user: Some("bob".to_owned()),
                count: 3,
                window_start: now.saturating_sub(120),
                locked_until: 0,
            },
            LockoutRecord {
                user: Some("carol".to_owned()),
                count: 5,
                window_start: now,
                locked_until: now.saturating_add(30),
            },
            LockoutRecord {
                user: None,
                count: 999,
                window_start: now,
                locked_until: now.saturating_add(10_000_000),
            },
        ];
        let throttle = Throttle::restore(records, now);
        assert_eq!(
            throttle.retry_after("alice", now),
            Some(VERIFY_LOCKOUT_SECS),
            "a hostile deadline is clamped to the configured lockout"
        );
        assert_eq!(
            throttle.retry_after("alice", now.saturating_add(VERIFY_LOCKOUT_SECS)),
            None
        );
        assert_eq!(
            throttle.retry_after("bob", now.saturating_add(VERIFY_GLOBAL_LOCKOUT_SECS)),
            None,
            "an expired entry is dropped once the global lock also ends"
        );
        assert_eq!(throttle.retry_after("carol", now), Some(30));
        assert_eq!(
            throttle.retry_after("stranger", now),
            Some(VERIFY_GLOBAL_LOCKOUT_SECS)
        );
    }

    #[test]
    fn guessing_many_names_hits_the_daemon_wide_ceiling() {
        let mut throttle = Throttle::new();
        for i in 0..VERIFY_GLOBAL_FAILURE_LIMIT {
            throttle.record_failure(&format!("user{i}"), 5_000);
        }
        assert_eq!(
            throttle.retry_after("never-seen", 5_000),
            Some(VERIFY_GLOBAL_LOCKOUT_SECS)
        );
        assert_eq!(throttle.retry_after("never-seen", 5_030), None);
    }

    #[test]
    fn the_account_table_stays_bounded() {
        let mut throttle = Throttle::new();
        for i in 0..MAX_TRACKED_ACCOUNTS * 2 {
            throttle.record_failure(&format!("user{i}"), 10);
        }
        assert!(throttle.accounts.len() <= MAX_TRACKED_ACCOUNTS);
        // The newest name is refused tracking rather than evicting a live entry.
        assert!(!throttle
            .accounts
            .contains_key(&format!("user{}", MAX_TRACKED_ACCOUNTS * 2 - 1)));
    }
}
