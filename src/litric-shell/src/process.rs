//! Ending one process the user selected in Activity.
//!
//! This is deliberately **not** a system-service operation. The kernel already
//! decides who may signal whom; this module only adds the two checks that make
//! the window's promise honest:
//!
//! * the target must run as this session's user, and
//! * it must be in this desktop session (the same kernel session id).
//!
//! Those two rules keep the button to the processes the person in front of the
//! screen started, and refuse the machine's services even when the shell itself
//! runs as root. Nothing here escalates: the signal is sent as the shell's own
//! user, and the kernel enforces the rest.
//!
//! `SIGTERM` is asked for, not `SIGKILL`: a process that can save its work is
//! given the chance, and a process that ignores the request is reported as
//! still running by the next facts sample rather than claimed as ended.

use std::path::Path;

/// The session id in `/proc/<pid>/stat`, after the command name.
///
/// The command may itself contain spaces and `)`, so parsing starts at the
/// **last** `)`: the fields after it are fixed. They are `state ppid pgrp
/// session ...`; the session is the fourth.
fn session_of(pid: u32) -> Option<u64> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (_, rest) = text.rsplit_once(')')?;
    let mut fields = rest.split_whitespace();
    let _state = fields.next()?;
    let _ppid = fields.next()?;
    let _pgrp = fields.next()?;
    fields.next()?.parse().ok()
}

/// The effective uid of this process, as `/proc/self/status` reports it.
fn own_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    litric_sysd::facts::parse_uid(&status)
}

/// The uid and session of a target, if it exists and is readable.
fn identity(pid: u32) -> Option<(u32, u64)> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let uid = litric_sysd::facts::parse_uid(&status)?;
    Some((uid, session_of(pid)?))
}

/// The absolute `kill` this image ships.
///
/// An absolute path, never `PATH`: a desktop that runs whatever `kill` an
/// environment variable points at is a desktop that can be made to run a
/// different program from the one it names.
fn kill_program() -> Option<&'static str> {
    ["/bin/kill", "/usr/bin/kill"]
        .into_iter()
        .find(|path| Path::new(path).is_file())
}

/// Ask one process in this session to stop.
///
/// # Errors
/// A sentence for the Activity window: the target is the machine's init or this
/// desktop, is not running, belongs to another user or session, or the signal
/// could not be sent. Nothing is claimed about a process that was merely
/// signalled; the next facts sample says whether it went.
pub fn terminate(pid: u32) -> Result<(), String> {
    #[cfg(not(unix))]
    {
        let _ = pid;
        return Err("ending processes is not available on this system".to_owned());
    }
    #[cfg(unix)]
    {
        if pid <= 1 {
            return Err("the system does not let this window end that process".to_owned());
        }
        if pid == std::process::id() {
            return Err("that is this desktop's own process".to_owned());
        }
        let Some((uid, session)) = identity(pid) else {
            return Err("that process has already ended".to_owned());
        };
        let Some(own) = own_uid() else {
            return Err("this system does not report process owners".to_owned());
        };
        if uid != own {
            return Err("that process belongs to another user".to_owned());
        }
        if session_of(std::process::id()) != Some(session) {
            return Err("that process is outside this desktop session".to_owned());
        }
        let Some(program) = kill_program() else {
            return Err("this system has no way to signal a process".to_owned());
        };
        let status = std::process::Command::new(program)
            .arg("-TERM")
            .arg(pid.to_string())
            .status()
            .map_err(|_| "the system refused to signal that process".to_owned())?;
        if status.success() {
            Ok(())
        } else {
            Err("the system refused to signal that process".to_owned())
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    // Test setup fails loudly rather than returning a fake result: the
    // product code above is the part that must never panic or unwrap.
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::time::{Duration, Instant};

    fn child() -> std::process::Child {
        let child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .unwrap();
        // /proc entries appear with the process; the signal path reads them.
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(2) {
            if Path::new(&format!("/proc/{}/stat", child.id())).is_file() {
                return child;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        child
    }

    fn wait_for_exit(child: &mut std::process::Child) -> bool {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(3) {
            if let Ok(Some(_)) = child.try_wait() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    #[test]
    fn a_child_in_this_session_is_really_terminated() {
        let mut child = child();
        let pid = child.id();
        assert_eq!(terminate(pid), Ok(()));
        assert!(wait_for_exit(&mut child), "the child survived SIGTERM");
    }

    #[test]
    fn init_and_this_desktop_are_refused() {
        assert!(terminate(1).is_err(), "init must never be a target");
        assert!(
            terminate(0).is_err(),
            "there is no process 0 to end from a window"
        );
        let own = terminate(std::process::id()).unwrap_err();
        assert!(own.contains("desktop's own process"), "{own}");
    }

    #[test]
    fn a_process_that_has_gone_reports_that_it_has_gone() {
        let mut child = child();
        let pid = child.id();
        let _ = child.kill();
        let _ = child.wait();
        let gone = terminate(pid).unwrap_err();
        assert!(gone.contains("already ended"), "{gone}");
    }
}
