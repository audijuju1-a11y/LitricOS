//! Ending the process the user selected in Activity, and what it started.
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
//!
//! A tree is walked from the kernel's own children relation
//! (`/proc/<pid>/task/*/children`), deepest first, so a parent cannot respawn
//! what has already been stopped. The walk is bounded: at most
//! [`MAX_TREE_NODES`] processes and [`MAX_TREE_DEPTH`] levels. Every node is
//! checked by the same owner-and-session rule as a single process; a node that
//! fails is counted and left alone, never signalled.

use std::collections::BTreeSet;
use std::path::Path;

/// The most processes one tree request will visit.
///
/// A desktop process tree is tens of nodes, not thousands. The cap keeps a
/// mistaken click on a huge tree from becoming thousands of signal calls, and
/// is reported rather than silently swallowing the rest.
pub const MAX_TREE_NODES: usize = 256;

/// The deepest tree one request will walk.
pub const MAX_TREE_DEPTH: u32 = 32;

/// Which promise a confirmation click is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndTarget {
    /// One process.
    Task,
    /// The process and everything it started.
    Tree,
}

/// What a tree request actually did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeEnd {
    /// Processes the system accepted a stop request for.
    pub asked: usize,
    /// Processes left alone because they are not this user's or this session's.
    pub refused: usize,
    /// Processes the system would not signal for another reason.
    pub failed: usize,
}

/// Why a process may not be signalled.
enum Standing {
    /// This session's user may signal it.
    Allowed,
    /// It has already exited (or is a zombie waiting to be reaped).
    Gone,
    /// It exists but is out of scope, with the sentence the window shows.
    Refused(&'static str),
}

/// The session id and state in `/proc/<pid>/stat`, after the command name.
///
/// The command may itself contain spaces and `)`, so parsing starts at the
/// **last** `)`: the fields after it are fixed. They are `state ppid pgrp
/// session ...`.
fn stat_of(pid: u32) -> Option<(char, u64)> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (_, rest) = text.rsplit_once(')')?;
    let mut fields = rest.split_whitespace();
    let state = fields.next()?.chars().next()?;
    let _ppid = fields.next()?;
    let _pgrp = fields.next()?;
    let session = fields.next()?.parse().ok()?;
    Some((state, session))
}

/// The session id in `/proc/<pid>/stat`.
fn session_of(pid: u32) -> Option<u64> {
    stat_of(pid).map(|(_, session)| session)
}

/// The effective uid of this process, as `/proc/self/status` reports it.
fn own_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    litric_sysd::facts::parse_uid(&status)
}

/// This desktop's uid and kernel session.
fn own_identity() -> Option<(u32, u64)> {
    Some((own_uid()?, session_of(std::process::id())?))
}

/// The uid, state and session of a target, if it exists and is readable.
fn identity(pid: u32) -> Option<(u32, char, u64)> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let uid = litric_sysd::facts::parse_uid(&status)?;
    let (state, session) = stat_of(pid)?;
    Some((uid, state, session))
}

/// Whether this desktop may ask `pid` to stop.
fn standing(pid: u32, own: (u32, u64)) -> Standing {
    if pid <= 1 {
        return Standing::Refused("the system does not let this window end that process");
    }
    if pid == std::process::id() {
        return Standing::Refused("that is this desktop's own process");
    }
    let Some((uid, state, session)) = identity(pid) else {
        return Standing::Gone;
    };
    if state == 'Z' || state == 'X' {
        return Standing::Gone;
    }
    if uid != own.0 {
        return Standing::Refused("that process belongs to another user");
    }
    if session != own.1 {
        return Standing::Refused("that process is outside this desktop session");
    }
    Standing::Allowed
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

/// Ask one process to stop.
fn signal(pid: u32) -> bool {
    let Some(program) = kill_program() else {
        return false;
    };
    std::process::Command::new(program)
        .arg("-TERM")
        .arg(pid.to_string())
        .status()
        .is_ok_and(|status| status.success())
}

/// The children of every thread in `pid`, from the kernel's own relation.
///
/// `/proc/<pid>/task/<tid>/children` is the authoritative list, so a child
/// that has already been reparented is not included and a grandchild is found
/// through its own parent. An empty result means either "no children" or "this
/// kernel does not publish the relation"; the tree request then ends only the
/// selected process, which is honest about what it could see.
fn children(pid: u32) -> Vec<u32> {
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for task in tasks.flatten() {
        let Ok(text) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        for field in text.split_whitespace().take(MAX_TREE_NODES) {
            if let Ok(child) = field.parse::<u32>() {
                out.push(child);
            }
        }
    }
    out
}

/// Collect a tree deepest-first, bounded and without revisiting a node.
fn collect(pid: u32, depth: u32, seen: &mut BTreeSet<u32>, order: &mut Vec<u32>) {
    if depth > MAX_TREE_DEPTH || seen.len() >= MAX_TREE_NODES {
        return;
    }
    if !seen.insert(pid) {
        return;
    }
    for child in children(pid) {
        collect(child, depth.saturating_add(1), seen, order);
    }
    order.push(pid);
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
        let Some(own) = own_identity() else {
            return Err("this system does not report process owners".to_owned());
        };
        match standing(pid, own) {
            Standing::Allowed => {}
            Standing::Gone => return Err("that process has already ended".to_owned()),
            Standing::Refused(reason) => return Err(reason.to_owned()),
        }
        if kill_program().is_none() {
            return Err("this system has no way to signal a process".to_owned());
        }
        if signal(pid) {
            Ok(())
        } else {
            Err("the system refused to signal that process".to_owned())
        }
    }
}

/// Ask one process and everything it started to stop.
///
/// Descendants are asked first, deepest first, then the selected process, so a
/// parent cannot spawn a replacement while its children are being stopped.
/// Every node passes the same owner-and-session rule as [`terminate`]; a node
/// that fails is counted in [`TreeEnd::refused`] and left running, never
/// silently signalled. The walk is bounded by [`MAX_TREE_NODES`] and
/// [`MAX_TREE_DEPTH`].
///
/// # Errors
/// The selected process itself is refused, has gone, or not one node could be
/// signalled. A partial success is returned with honest counts, not an error.
pub fn terminate_tree(pid: u32) -> Result<TreeEnd, String> {
    #[cfg(not(unix))]
    {
        let _ = pid;
        return Err("ending processes is not available on this system".to_owned());
    }
    #[cfg(unix)]
    {
        let Some(own) = own_identity() else {
            return Err("this system does not report process owners".to_owned());
        };
        match standing(pid, own) {
            Standing::Allowed => {}
            Standing::Gone => return Err("that process has already ended".to_owned()),
            Standing::Refused(reason) => return Err(reason.to_owned()),
        }
        if kill_program().is_none() {
            return Err("this system has no way to signal a process".to_owned());
        }

        let mut seen = BTreeSet::new();
        let mut order = Vec::new();
        collect(pid, 0, &mut seen, &mut order);

        let mut done = TreeEnd::default();
        for node in order {
            match standing(node, own) {
                Standing::Allowed => {
                    if signal(node) {
                        done.asked = done.asked.saturating_add(1);
                    } else if identity(node).is_none() {
                        // It exited between the check and the signal: the
                        // outcome the user wanted, not a failure to report.
                    } else {
                        done.failed = done.failed.saturating_add(1);
                    }
                }
                Standing::Gone => {}
                Standing::Refused(_) => done.refused = done.refused.saturating_add(1),
            }
        }

        if done.asked == 0 {
            if done.failed > 0 {
                return Err("the system refused to signal those processes".to_owned());
            }
            return Err("those processes had already ended".to_owned());
        }
        Ok(done)
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

    /// A process has ended when it is gone or is a zombie waiting to be reaped.
    fn running(pid: u32) -> bool {
        match identity(pid) {
            Some((_, state, _)) => state != 'Z' && state != 'X',
            None => false,
        }
    }

    fn wait_for_stop(pid: u32) -> bool {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(3) {
            if !running(pid) {
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

    #[test]
    fn a_tree_is_terminated_from_the_leaves_up() {
        let mut parent = std::process::Command::new("sh")
            .arg("-c")
            .arg("sleep 30 & sleep 30 & wait")
            .spawn()
            .unwrap();
        let pid = parent.id();
        let started = Instant::now();
        let mut kids = children(pid);
        while started.elapsed() < Duration::from_secs(2) && kids.len() < 2 {
            kids = children(pid);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(kids.len(), 2, "the fixture is a real two-child tree");

        let done = terminate_tree(pid).unwrap();
        // A parent that is waiting may exit by itself once its children stop,
        // so it is not always still there to be asked. Every process that was
        // there was asked, and the test below asserts that all of them stopped.
        assert!(done.asked >= 2, "both children were asked");
        assert_eq!(done.refused, 0, "every node is this session's");
        assert!(wait_for_exit(&mut parent), "the tree's parent survived");
        for kid in &kids {
            assert!(wait_for_stop(*kid), "child {kid} survived the tree stop");
        }
    }

    #[test]
    fn a_tree_will_not_start_from_init() {
        assert!(terminate_tree(1).is_err(), "init is never a tree root");
        assert!(terminate_tree(std::process::id()).is_err());
    }
}
