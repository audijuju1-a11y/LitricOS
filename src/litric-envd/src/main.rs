//! `litric-envd` — the environment daemon binary.
//!
//! A supervisor loop, deliberately small: it owns the [`Supervisor`], polls
//! guest exits, and records state where the shell can read it. The full
//! control protocol (hotkey wiring, the compositor's surface swap) lands
//! with the shell integration; what ships today is the part that must be
//! correct before anything sits on top of it — the launch/supervise/record
//! core, running as a real process.

#![forbid(unsafe_code)]

use litric_envd::{GuestExit, Supervisor};
use std::path::PathBuf;
use std::time::Duration;

/// Where the daemon keeps its per-run state.
const RUN_DIR: &str = "/run/litric/env";

fn main() {
    let mut supervisor = Supervisor::new();
    let run_dir = PathBuf::from(RUN_DIR);
    let _ = std::fs::create_dir_all(&run_dir);

    println!(
        "litric-envd: supervisor ready, run dir {}",
        run_dir.display()
    );

    // The loop the whole crate exists to support: poll, reap, record, sleep.
    // A guest exit is written to the run dir so the shell (and the recovery
    // story) can answer "what happened to Windows?" without asking the dead
    // process.
    loop {
        for (id, exit) in supervisor.poll() {
            let line = match exit {
                GuestExit::Clean => format!("{}: exited cleanly", id.token()),
                GuestExit::Crashed { code } => format!("{}: crashed (code {code})", id.token()),
                GuestExit::NeverLaunched => format!("{}: never launched", id.token()),
            };
            println!("litric-envd: {line}");
            let _ = std::fs::write(run_dir.join(format!("{}.last-exit", id.token())), &line);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
