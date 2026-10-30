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

use std::collections::BTreeMap;

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
pub trait AuthProvider {
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
