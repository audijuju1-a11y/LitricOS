# BUG-0027 — The Terminal showed nothing until the command exited

```
Bug ID        : BUG-0027
Severity      : S3 (MEDIUM) — no data is lost and nothing is corrupted, but the
                Terminal is indistinguishable from a hung window for the whole
                life of a slow command, and a command killed at the 30 s
                timeout showed no output at all
Class         : UI / TERMINAL (output buffered to process exit instead of read
                as it is produced)
Subsystem     : src/litric-shell/src/shell.rs (`Shell::terminal_run`),
                src/litric-shell/src/apps.rs (`TerminalApp::push_line`,
                `TerminalApp::MAX_LINES`)
First Observed: 2026-10-27, reading the terminal path against the report's
                finding on streaming output
Status        : FIXED (one regression test)
```

## Why this exists

`terminal_run` spawned a worker thread around a blocking call:

```rust
let out = std::process::Command::new("timeout")
    .args(["30", "/bin/sh", "-c", &cmd])
    .envs(envs)
    .output();
```

`Command::output()` reads both pipes to end and only returns when the child has
exited. Every line was therefore queued at once, after the command finished:

* a `cargo build` of forty seconds showed forty seconds of an empty window and a
  spinner-less `busy` flag the user cannot see;
* a command that hit `timeout 30` produced **nothing** — `output()` returns
  `Err` on a signal-terminated process only in the sense of a failed status, and
  the old code took the `Ok(o)` arm only, so its output was discarded with it;
* the 300-line truncation was applied to the joined text, so the *last* 300
  lines were never shown, only the *first*.

## Fix

The worker pipes both streams and reads them as they arrive:

```rust
let child = std::process::Command::new("timeout")
    .args(["30", "/bin/sh", "-c", &cmd])
    .envs(envs)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
```

* **A line is a line.** `BufReader::lines()` on stdout pushes each line into the
  existing channel, so `tick`'s `terminal_pump` moves it into the scrollback on
  the next frame.
* **stderr has its own reader.** Two pipes read by one thread deadlock as soon as
  the process fills the buffer of the pipe the thread is not reading.
* **The exit code still arrives.** After the pipes close, `child.wait()` reports
  `[exit N]` for a failure; an unreadable status reports itself rather than
  vanishing.
* **The cap moved to the model.** `TerminalApp::MAX_LINES` (400) already bounded
  the scrollback; the extra `take(300)` in the worker was a second, invisible
  ceiling that dropped the tail of long output.

## Regression test

`shell::scan_worker_tests::a_terminal_line_arrives_while_the_command_is_still_running`
runs `echo first; sleep 1; echo second` through `terminal_run` and pumps frames
the way the real loop does. It asserts that `first` was in the scrollback **while
`terminal.busy` was still true** — the property `output()` cannot satisfy — and
that `second` arrives before the command finishes.

## Verification

* `cargo test --offline --workspace` — 1594 passed, 0 failed
  (`docs/evidence/2026-10-27/workspace-tests-after-interaction-repair.log`).
* `cargo test --offline -p litric-shell scan_worker_tests` — 6 passed
  (`docs/evidence/2026-10-27/scan-banner-terminal-tests.log`).
* `ci/verify.sh` — 29 pass, 0 fail, 3 skip (image, boot, idle metrics)
  (`docs/evidence/2026-10-27/verify-after-interaction-repair.log`).

## Residual risk

* **No pseudo-terminal.** The child gets pipes, not a pty, so interactive
  programs (`vim`, `less`), progress bars that redraw with `\r`, and anything
  that checks `isatty` still behave as they do in a script rather than in a
  terminal. `\r` redraws arrive as several lines.
* **No stdin.** The child's stdin is `/dev/null`; a program that waits for input
  sees EOF immediately. That was already true before this repair.
* The command still runs with the shell's own privileges under `timeout 30`;
  that is the report's finding on the Terminal's trust boundary, a decision about
  the sandbox rather than about output, and it is unchanged.
