//! The write half of the wave-1 providers: what the session *does* to the
//! machine, as opposed to what [`crate::platform`] reads from it.
//!
//! # What is here
//!
//! The runner seam, and the power transition that was the first action to
//! execute. [`crate::providers`] is the rest of the wave-1 write half — audio,
//! network, Bluetooth and authentication — and it is a separate module because
//! this one is about *how a program is run* and that one is about *which program
//! and with what arguments*.
//!
//! # How it is reached
//!
//! ADR-0023 decision 2: the base tools that already own the interface, spoken
//! to the way they are meant to be spoken to. PID 1 owns the power
//! transitions, `systemctl` is how a person asks PID 1 for one, so the daemon
//! runs `systemctl suspend|reboot|poweroff` — through [`Runner`], whose only
//! implementation that matters spawns a process, and whose test implementation
//! records the argument vector and runs nothing.
//!
//! That seam is the whole reason this file is testable on a Windows host with
//! no `systemctl` and no PID 1 to talk to: the *decision* to run something, the
//! exact argv, and the interpretation of the result are all separate from the
//! act of spawning. A test that had to suspend the machine to check the
//! argument vector would be a test nobody runs.
//!
//! # What is deliberately not flexible
//!
//! The program path is absolute. A relative `systemctl` found through `PATH` is
//! an abuse case (P7 in `docs/design/wave1/03-power.md`): anything that can
//! write to a directory earlier in `PATH` would choose what runs, with the
//! daemon's privilege, at the moment the user asks to power off. There is no
//! argument a request can add — the verb is a value from a closed enumeration —
//! so no request can turn this into an arbitrary-command primitive.

use core::fmt;
use std::io;
use std::io::{Read as _, Write as _};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use litric_caps::{Capability, CapabilityId, Scope};

/// The absolute path of the tool that asks PID 1 for a power transition.
///
/// Absolute on purpose; see the module comment.
pub const SYSTEMCTL: &str = "/usr/bin/systemctl";

/// How long a power transition is given before it is killed.
pub const POWER_TIMEOUT_SECS: u32 = 30;

/// A machine transition the session can request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    /// Suspend to RAM. Resumable, and the only one of the three that is.
    Suspend,
    /// Restart.
    Reboot,
    /// Power off.
    PowerOff,
}

impl PowerAction {
    /// Every action, in the order a menu shows them.
    pub const ALL: [Self; 3] = [Self::Suspend, Self::Reboot, Self::PowerOff];

    /// The word the protocol and the fact lines use.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Suspend => "suspend",
            Self::Reboot => "reboot",
            Self::PowerOff => "poweroff",
        }
    }

    /// The `systemctl` verb. The same word as the token, kept separate because
    /// they are two vocabularies that merely coincide today.
    #[must_use]
    pub const fn verb(self) -> &'static str {
        self.token()
    }

    /// Parse a token, case-insensitively.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|a| a.token().eq_ignore_ascii_case(s))
    }

    /// The capability that authorises this action.
    ///
    /// `PowerControl` scoped to one verb, so a grant to dim the backlight — the
    /// same capability — cannot suspend the machine, and so a grant to suspend
    /// cannot power it off. `Scope::is_destructive` is what makes the broker
    /// demand consent in the moment for all three.
    ///
    /// `None` when the scope string would not parse, which cannot happen for
    /// the three constants above; the `Option` keeps the failure at the caller
    /// rather than in a panic, and a test asserts the whole enumeration parses.
    #[must_use]
    pub fn capability(self) -> Option<Capability> {
        let scope = Scope::parse_resource(&format!("power://system/{}", self.token())).ok()?;
        Capability::new(CapabilityId::PowerControl, scope)
    }

    /// The words a prompt uses. Sentence case, and specific: "Suspend the
    /// machine?" is answerable, "Are you sure?" is not.
    #[must_use]
    pub const fn prompt(self) -> &'static str {
        match self {
            Self::Suspend => "Suspend the machine?",
            Self::Reboot => "Restart the machine?",
            Self::PowerOff => "Power the machine off?",
        }
    }

    /// What the user is about to lose, said plainly.
    #[must_use]
    pub const fn consequence(self) -> &'static str {
        match self {
            Self::Suspend => "The session stays open and this machine can be woken again.",
            Self::Reboot => "Everything running stops and the machine restarts.",
            Self::PowerOff => "Everything running stops and the machine stays off.",
        }
    }
}

impl fmt::Display for PowerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// How a program is run.
///
/// One method, and the two implementations are [`SystemRunner`], which really
/// spawns, and the recorder in this module's tests, which does not. Anything
/// that needs a shell, a pipe or an environment is not expressible here, which
/// is the point: `systemctl` needs none of them, and a runner that could give
/// an argument to a shell would hand the two functions below a way to become
/// something much worse than they are.
pub trait Runner: fmt::Debug + Send + Sync {
    /// Run `program` with `args`, optionally writing `stdin` to it, and report
    /// its exit status. A program that has not exited within `timeout_secs` is
    /// killed and reported as [`io::ErrorKind::TimedOut`].
    ///
    /// One method rather than three, and the timeout is not optional: **a tool
    /// that hangs must not hang the daemon.** That property was not hypothetical
    /// — the first booted image that exercised these actions reported
    /// `litric.sysd.audio_mute` and then stopped, because `bluetoothctl` on a
    /// machine with no radio waits for a D-Bus name that never appears. A
    /// deadlock in a privileged daemon is worse than a failed action, and the
    /// only place to prevent it is here.
    ///
    /// Standard input is fed by the same call because the alternative — a
    /// separate "run with stdin" entry point — needs a default implementation
    /// for every runner, and a default that quietly runs without the password is
    /// how a failed verification starts looking like a wrong password.
    ///
    /// # Errors
    /// The I/O error from spawning or waiting — a program that is not there, a
    /// permission refusal, a daemon that cannot fork — or [`io::ErrorKind::TimedOut`]
    /// when the program was killed. A program that ran and failed is
    /// `Ok(status)`, and the caller distinguishes the three.
    fn run(
        &self,
        program: &str,
        args: &[&str],
        stdin: Option<&[u8]>,
        timeout_secs: u32,
    ) -> io::Result<i32>;

    /// Run `program` and this time keep what it printed.
    ///
    /// A second method rather than a flag on the first: [`Runner::run`] throws
    /// standard output away on purpose, and every action is written against
    /// that. The one caller of this is a *question* (`wpctl get-volume`), where
    /// the exit status says only that the tool ran and the answer is the text.
    /// The timeout is the same bounded wait, and the output is capped, so a tool
    /// that streams forever cannot grow the daemon's heap.
    ///
    /// # Errors
    /// The same three as [`Runner::run`]: a spawn or wait error, or
    /// [`io::ErrorKind::TimedOut`] when the program was killed.
    fn capture(&self, program: &str, args: &[&str], timeout_secs: u32)
        -> io::Result<(i32, String)>;
}

/// The most standard output [`Runner::capture`] will keep, in bytes.
///
/// The tools this is used for answer in one short line (`Volume: 0.40`). A cap
/// exists so that a tool printing without end costs a fixed amount of memory
/// rather than all of it.
pub const CAPTURE_LIMIT: u64 = 4096;

/// The runner that spawns a real process.
#[derive(Debug)]
pub struct SystemRunner;

impl Runner for SystemRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        stdin: Option<&[u8]>,
        timeout_secs: u32,
    ) -> io::Result<i32> {
        // stdout and stderr are discarded rather than inherited. A tool's own
        // words about a failure go to stderr, and the daemon has nowhere to put
        // them that is not a log a password might reach; the exit status is the
        // whole answer.
        let mut child = Command::new(program)
            .args(args)
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        if let Some(input) = stdin {
            if let Some(mut sink) = child.stdin.take() {
                // A helper that exits before it has read the whole password — a
                // user that does not exist is the common case — closes the pipe
                // early. That is a failed verification reported by the exit
                // status, not an I/O error about the pipe, so the write result
                // is dropped on purpose. Dropping the handle closes the pipe,
                // which is what tells the helper the password ended.
                let _ = sink.write_all(input);
                drop(sink);
            }
        }
        // A bounded wait. `wait()` would block until the tool decides to exit,
        // and a tool waiting for a D-Bus name that never appears never does.
        //
        // The wait is a duration since the start rather than a deadline instant:
        // adding a duration to an `Instant` can overflow, and there is no
        // timeout so large that it would be worth an `unwrap` to allow it.
        let budget = Duration::from_secs(u64::from(timeout_secs));
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(status.code().unwrap_or(-1));
            }
            if Instant::now().saturating_duration_since(started) >= budget {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("{program} did not exit within {timeout_secs}s"),
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn capture(
        &self,
        program: &str,
        args: &[&str],
        timeout_secs: u32,
    ) -> io::Result<(i32, String)> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        // Read on another thread, before waiting: a tool that fills the pipe
        // and a daemon that waits for it to exit would wait forever.
        let reader = child.stdout.take().map(|out| {
            thread::Builder::new()
                .name("litric-sysd-capture".to_owned())
                .spawn(move || {
                    let mut kept = Vec::new();
                    let _ = io::Read::take(out, CAPTURE_LIMIT)
                        .read_to_end(&mut kept)
                        .map(|_| ());
                    kept
                })
        });
        let budget = Duration::from_secs(u64::from(timeout_secs));
        let started = Instant::now();
        let code = loop {
            if let Some(status) = child.try_wait()? {
                break status.code().unwrap_or(-1);
            }
            if Instant::now().saturating_duration_since(started) >= budget {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("{program} did not exit within {timeout_secs}s"),
                ));
            }
            thread::sleep(Duration::from_millis(20));
        };
        let printed = reader
            .and_then(Result::ok)
            .and_then(|handle| handle.join().ok())
            .map_or_else(String::new, |bytes| {
                String::from_utf8_lossy(&bytes).into_owned()
            });
        Ok((code, printed))
    }
}

/// What an action did.
///
/// `applied` is separate from the lines because the caller branches on it: a
/// shell drawing a notification needs to know whether to say "the machine is
/// going down" or "that did not work", and parsing its own reply text to find
/// out is how an interface starts disagreeing with itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Whether the machine is doing what was asked.
    pub applied: bool,
    /// The reply lines, in order, for the protocol to carry.
    pub lines: Vec<String>,
}

impl Outcome {
    fn apply(action: PowerAction, exit: i32) -> Self {
        Self {
            applied: true,
            lines: vec![
                format!("action={}", action.token()),
                format!("command={SYSTEMCTL} {}", action.verb()),
                format!("exit={exit}"),
                "applied=true".to_owned(),
            ],
        }
    }

    fn failed(action: PowerAction, reason: &str, detail: &str) -> Self {
        Self {
            applied: false,
            lines: vec![
                format!("action={}", action.token()),
                format!("command={SYSTEMCTL} {}", action.verb()),
                "applied=false".to_owned(),
                format!("reason={reason}"),
                format!("detail={detail}"),
            ],
        }
    }
}

/// Ask PID 1 for a power transition.
///
/// The only action this module performs, and the only one this build has: the
/// capability was checked before this is called (by [`crate::state`], against
/// the broker, with consent) and there is nothing here that can be reached
/// without having passed that.
#[must_use]
pub fn power(action: PowerAction, runner: &dyn Runner) -> Outcome {
    // `systemctl` hands the request to PID 1 and returns, so this timeout is not
    // about a slow transition; it is the backstop that keeps a wedged D-Bus from
    // becoming a daemon that never answers again. Thirty seconds is longer than
    // a suspend request takes to be *accepted* on any machine this has run on.
    match runner.run(SYSTEMCTL, &[action.verb()], None, POWER_TIMEOUT_SECS) {
        Ok(0) => Outcome::apply(action, 0),
        // A non-zero exit is a refusal from the system: no logind, no
        // permission, a unit that would not start. The status is reported
        // rather than folded into "failed", because `systemctl`'s statuses are
        // the only diagnosis the caller is going to get.
        Ok(code) => Outcome::failed(action, "exit", &format!("systemctl exited {code}")),
        Err(e) => Outcome::failed(action, "spawn-failed", &e.to_string()),
    }
}

/// A runner that records instead of spawning.
///
/// Every test in this crate that reaches an action builds one of these, and the
/// alternative — running `systemctl` for real — would mean a test suite that
/// powers the machine off. It lives outside `mod tests` because the daemon's
/// request tests need it too, and a `pub` item inside a test module is not
/// reachable from another module: the seam has to be reachable to be the seam.
#[cfg(test)]
pub(crate) mod testing {
    use super::{io, Runner};
    use std::sync::{Arc, Mutex};

    /// One recorded call: the program, and its arguments.
    type Call = (String, Vec<String>);

    /// The stack of calls an inspection reads through.
    type Calls = Arc<Mutex<Vec<Call>>>;

    /// A runner that records its calls and runs nothing.
    ///
    /// Cloneable, and every clone shares one record: a test hands a clone to
    /// the daemon, keeps the original, and asserts afterwards on what the
    /// daemon would have run. A runner that could not be inspected after being
    /// given away would only be able to assert that *something* happened.
    #[derive(Debug, Default, Clone)]
    pub struct RecordingRunner {
        calls: Calls,
        result: Arc<Mutex<Option<io::Result<i32>>>>,
        /// How many bytes the last call was fed on standard input.
        stdin: Arc<Mutex<Option<usize>>>,
        /// The deadline the last call was given.
        timeout: Arc<Mutex<Option<u32>>>,
        /// What [`Runner::capture`] answers with: `(exit status, stdout)`.
        ///
        /// `None` means no answer was set, and a capture then fails the way a
        /// missing tool does — so a test that forgets to set one cannot quietly
        /// pass by reading an empty string as a real measurement.
        capture: Arc<Mutex<Option<(i32, String)>>>,
    }

    impl RecordingRunner {
        /// A runner whose next call reports `code`.
        #[must_use]
        pub fn succeeding(code: i32) -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                result: Arc::new(Mutex::new(Some(Ok(code)))),
                stdin: Arc::new(Mutex::new(None)),
                timeout: Arc::new(Mutex::new(None)),
                capture: Arc::new(Mutex::new(None)),
            }
        }

        /// A runner whose next call fails before the program runs.
        #[must_use]
        pub fn failing(kind: io::ErrorKind, what: &str) -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                result: Arc::new(Mutex::new(Some(Err(io::Error::new(kind, what.to_owned()))))),
                stdin: Arc::new(Mutex::new(None)),
                timeout: Arc::new(Mutex::new(None)),
                capture: Arc::new(Mutex::new(None)),
            }
        }

        /// The exit status and standard output the next [`Runner::capture`]
        /// answers with.
        #[must_use]
        pub fn answering(mut self, code: i32, stdout: &str) -> Self {
            self.capture = Arc::new(Mutex::new(Some((code, stdout.to_owned()))));
            self
        }

        /// Every call made, as `(program, args)`.
        #[must_use]
        pub fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().map(|c| c.clone()).unwrap_or_default()
        }

        /// How many bytes of standard input the last call was given.
        ///
        /// The bytes themselves are deliberately not kept; see the implementation
        /// of [`Runner::run`] above.
        #[must_use]
        pub fn stdin_len(&self) -> Option<usize> {
            self.stdin.lock().map(|v| *v).unwrap_or(None)
        }

        /// The deadline the last call was given, in seconds.
        #[must_use]
        pub fn timeout_secs(&self) -> Option<u32> {
            self.timeout.lock().map(|v| *v).unwrap_or(None)
        }

        fn record(
            &self,
            program: &str,
            args: &[&str],
            stdin_len: Option<usize>,
            timeout_secs: u32,
        ) -> io::Result<i32> {
            match self.calls.lock() {
                Ok(mut c) => c.push((
                    program.to_owned(),
                    args.iter().map(|a| (*a).to_owned()).collect(),
                )),
                Err(_) => return Err(io::Error::other("runner lock poisoned")),
            }
            if let Ok(mut slot) = self.stdin.lock() {
                *slot = stdin_len;
            }
            if let Ok(mut slot) = self.timeout.lock() {
                *slot = Some(timeout_secs);
            }
            match self.result.lock() {
                Ok(mut r) => r.take().unwrap_or(Ok(0)),
                Err(_) => Err(io::Error::other("runner lock poisoned")),
            }
        }
    }

    impl Runner for RecordingRunner {
        fn run(
            &self,
            program: &str,
            args: &[&str],
            stdin: Option<&[u8]>,
            timeout_secs: u32,
        ) -> io::Result<i32> {
            // What is recorded about standard input is its *length* and nothing
            // else. A recorder that kept the bytes would put a password in a
            // test's memory and, the first time a test printed its calls, into a
            // log — which is exactly the property `providers` is built to keep.
            // The timeout is recorded so a test can assert that an action asked
            // for a bounded wait rather than an unbounded one.
            self.record(program, args, stdin.map(<[u8]>::len), timeout_secs)
        }

        fn capture(
            &self,
            program: &str,
            args: &[&str],
            timeout_secs: u32,
        ) -> io::Result<(i32, String)> {
            self.record(program, args, None, timeout_secs)?;
            match self.capture.lock() {
                Ok(mut slot) => slot.take().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "this test set no captured output".to_owned(),
                    )
                }),
                Err(_) => Err(io::Error::other("runner lock poisoned")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::RecordingRunner;
    use super::*;

    fn line<'a>(out: &'a Outcome, prefix: &str) -> Option<&'a String> {
        out.lines.iter().find(|l| l.starts_with(prefix))
    }

    /// The capturing call keeps what the program printed, and reports a status
    /// a caller can act on. It runs a real program because that is the half the
    /// recorder cannot prove: a pipe, a reader thread and a bounded wait.
    #[cfg(unix)]
    #[test]
    fn a_captured_call_returns_the_printed_answer_and_the_status() {
        let echo = ["/bin/echo", "/usr/bin/echo"]
            .into_iter()
            .find(|p| std::path::Path::new(p).is_file());
        let Some(echo) = echo else {
            // No `echo` on this host: nothing to prove here, and a test that
            // pretended otherwise would be measuring nothing.
            return;
        };
        let runner = SystemRunner;
        let (code, printed) = runner.capture(echo, &["Volume: 0.40"], 5).expect("runs");
        assert_eq!(code, 0);
        assert_eq!(printed.trim(), "Volume: 0.40");

        // A program that is not there is an error, not an empty answer: the
        // difference is what stops "no sound server" reading as "volume zero".
        let missing = runner.capture("/nonexistent/litric-no-such-tool", &[], 5);
        assert!(missing.is_err(), "{missing:?}");
    }

    #[test]
    fn every_action_parses_its_own_token_and_has_a_capability() {
        for a in PowerAction::ALL {
            assert_eq!(PowerAction::parse(a.token()), Some(a));
            assert_eq!(PowerAction::parse(&a.token().to_ascii_uppercase()), Some(a));
            let cap = a.capability().expect("every verb has a scope");
            assert_eq!(cap.id(), CapabilityId::PowerControl);
            assert!(
                cap.requires_live_consent(),
                "{a} ends the session and must need consent"
            );
        }
        assert_eq!(PowerAction::parse("hibernate"), None);
        assert_eq!(PowerAction::parse(""), None);
    }

    #[test]
    fn the_three_verbs_do_not_authorise_each_other() {
        for a in PowerAction::ALL {
            for b in PowerAction::ALL {
                let (ca, cb) = (a.capability().expect("v"), b.capability().expect("v"));
                assert_eq!(ca.authorises(&cb), a == b, "{a} vs {b}");
            }
        }
    }

    #[test]
    fn a_transition_runs_the_base_tool_with_a_fixed_argv() {
        let r = RecordingRunner::succeeding(0);
        let out = power(PowerAction::Reboot, &r);
        assert!(out.applied);
        assert_eq!(
            r.calls(),
            vec![(SYSTEMCTL.to_owned(), vec!["reboot".to_owned()])]
        );
        assert_eq!(line(&out, "exit="), Some(&"exit=0".to_owned()));
    }

    #[test]
    fn the_argv_is_never_built_from_anything_a_caller_said() {
        // The one thing a request contributes is the verb, and the verb is a
        // value from a closed enumeration: no quoting, no separators, no
        // arguments. This asserts the shape rather than trusting the reader of
        // `power` to notice.
        for a in PowerAction::ALL {
            let r = RecordingRunner::succeeding(0);
            let out = power(a, &r);
            assert!(out.applied);
            let calls = r.calls();
            let (program, args) = calls.first().expect("one call");
            assert_eq!(program, SYSTEMCTL);
            assert_eq!(args.len(), 1, "{a} passes exactly one argument");
            assert_eq!(args.first().map(String::as_str), Some(a.verb()));
            assert!(!a.verb().contains('/') && !a.verb().contains(' '));
        }
    }

    #[test]
    fn a_nonzero_exit_is_not_success_and_says_why() {
        let r = RecordingRunner::succeeding(1);
        let out = power(PowerAction::Suspend, &r);
        assert!(!out.applied);
        assert_eq!(line(&out, "applied="), Some(&"applied=false".to_owned()));
        assert_eq!(line(&out, "reason="), Some(&"reason=exit".to_owned()));
    }

    #[test]
    fn a_missing_systemctl_is_reported_rather_than_hidden() {
        // This is the state a machine without systemd is in, and the state
        // every test on a non-Linux host is in. Reporting it as a failed
        // transition, with the I/O error, is the honest answer.
        let r = RecordingRunner::failing(io::ErrorKind::NotFound, "no such file");
        let out = power(PowerAction::PowerOff, &r);
        assert!(!out.applied);
        assert_eq!(
            line(&out, "reason="),
            Some(&"reason=spawn-failed".to_owned())
        );
        assert!(line(&out, "detail=").is_some_and(|d| d.contains("no such file")));
    }

    #[test]
    fn the_real_runner_reports_a_program_that_is_not_there_rather_than_panicking() {
        let err = SystemRunner
            .run("/nonexistent-litric-no-such-tool", &["--version"], None, 5)
            .expect_err("spawning a missing program is an error");
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[cfg(unix)]
    #[test]
    fn the_real_runner_feeds_a_program_its_standard_input() {
        // The plumbing `unix_chkpwd` depends on, tested against a real program
        // rather than a recorder: the shell exits 0 only if the line it read is
        // the one this test wrote, so a silently empty pipe fails here.
        let code = SystemRunner
            .run(
                "/bin/sh",
                &["-c", "read line && [ \"$line\" = secret ]"],
                Some(b"secret\n"),
                10,
            )
            .expect("a program that exists");
        assert_eq!(code, 0, "the password reached the program's stdin");
    }

    #[cfg(unix)]
    #[test]
    fn a_tool_that_never_exits_is_killed_and_reported_as_a_timeout() {
        // The defect the first booted image found: `bluetoothctl` waiting for a
        // D-Bus name that never appears. An unbounded wait inside the privileged
        // daemon is worse than a failed action, so the bound is tested against a
        // program that really does not exit.
        let start = Instant::now();
        let err = SystemRunner
            .run("/bin/sleep", &["30"], None, 1)
            .expect_err("a program that outlives its deadline is not a result");
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "the wait was bounded by the deadline, not by the tool: {:?}",
            start.elapsed()
        );
    }
}
