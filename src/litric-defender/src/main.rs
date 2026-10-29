//! `litric-defender` — the on-demand scanner process.
//!
//! ADR-0022 separates the **decision** from the **enforcement**: the library is
//! the pure policy core, and this binary is the process that is allowed to touch
//! untrusted bytes. Until this file existed the crate built no binary at all, so
//! `image/units-pending/litric-defender.service` named an `ExecStart` the image
//! could not install, and the Defender application scanned inside the desktop
//! process — the one place ADR-0022 says content scanning must not happen.
//!
//! # What it is, and what it is not
//!
//! It is an **on-demand** scanner: `--scan PATH` for one answer, `--stdio` for a
//! framed worker a parent process can drive. It is **not** the on-access daemon:
//! that needs the fanotify transport (ADR-0022 Phase 2), which needs
//! `CAP_SYS_ADMIN` and a kernel-facing layer this crate does not have. The
//! parked unit therefore stays parked, and this binary is honest about being the
//! worker, not the hook.
//!
//! # Cost
//!
//! Memory is bounded by [`MAX_SCAN_BYTES`]: the file is read through `take`, so
//! a 4 GiB file costs 8 MiB and one extra byte, not 4 GiB. Above the ceiling the
//! file is reported `unknown` with `reason=too-large` rather than silently
//! scanned in part — a partial scan that says "clean" is worse than no scan.
//! There is no per-frame work anywhere: this process runs when it is asked.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::ExitCode;

use litric_defender::denylist::Denylist;
use litric_defender::engine::{Engine, ScanReport};
use litric_defender::verdict::{resolve, Decision, Mode, Observation, Tier};
use litric_defender::MAX_SCAN_BYTES;

const USAGE: &str = "\
litric-defender — on-demand scanner (ADR-0022 Phase 0 policy core)

USAGE:
    litric-defender --scan PATH [--denylist FEED] [--enforce]
    litric-defender --stdio [--denylist FEED] [--enforce]
    litric-defender --version

OPTIONS:
    --scan PATH        Scan one file and print one verdict.
    --stdio            Worker mode: read SCAN <hex-path> lines, answer OK/ERR.
    --denylist FEED    SHA-256 feed: 64-hex-digest label, one per line.
    --enforce          Unknown files are denied (Mode::Enforce). A detection is
                       always denied in either mode.
    -h, --help         This text.
    -V, --version      Version.

EXIT:
    0 allow or unknown, 1 detection, 2 usage or I/O error.
";

/// One process's engine and the mode it was started with.
struct Worker {
    engine: Engine,
    mode: Mode,
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut scan: Option<String> = None;
    let mut stdio = false;
    let mut feed: Option<String> = None;
    let mut enforce = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scan" => match args.next() {
                Some(path) => scan = Some(path),
                None => return usage_error("--scan needs a path"),
            },
            "--stdio" => stdio = true,
            "--denylist" => match args.next() {
                Some(path) => feed = Some(path),
                None => return usage_error("--denylist needs a path"),
            },
            "--enforce" => enforce = true,
            "--version" | "-V" => {
                println!("litric-defender {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--help" | "-h" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => return usage_error(&format!("unknown argument: {other}")),
        }
    }

    if scan.is_some() && stdio {
        return usage_error("--scan and --stdio are different modes");
    }
    if scan.is_none() && !stdio {
        print!("{USAGE}");
        return ExitCode::FAILURE;
    }

    let worker = match build_worker(feed.as_deref(), enforce) {
        Ok(worker) => worker,
        Err(error) => return usage_error(&error),
    };

    if let Some(path) = scan {
        return scan_once(worker, &path);
    }
    serve_stdio(worker)
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("litric-defender: {message}");
    eprintln!();
    eprint!("{USAGE}");
    ExitCode::FAILURE
}

/// Assemble the engine from the optional feed.
fn build_worker(feed: Option<&str>, enforce: bool) -> Result<Worker, String> {
    let mut engine = Engine::new();
    let mode = if enforce {
        Mode::Enforce
    } else {
        Mode::Permissive
    };
    engine.set_mode(mode);
    if let Some(path) = feed {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read denylist {path}: {error}"))?;
        let list = Denylist::parse(&text);
        // A feed that lost every line is a broken feed, not an empty one: the
        // difference matters because "no definitions" scans everything clean.
        if list.is_empty() && !list.rejected().is_empty() {
            return Err(format!(
                "denylist {path} has no usable entries ({} rejected)",
                list.rejected().len()
            ));
        }
        engine.load_denylist(list);
    }
    Ok(Worker { engine, mode })
}

fn scan_once(mut worker: Worker, path: &str) -> ExitCode {
    match scan_path(&mut worker, path) {
        Ok(report) => {
            for line in report_lines(&report) {
                println!("{line}");
            }
            exit_for(&report)
        }
        Err(error) => {
            println!("ERR {error}");
            ExitCode::FAILURE
        }
    }
}

/// The framed worker: `SCAN <hex path>`, `PING`, `QUIT`.
///
/// The path is hex-encoded because a desktop file name may contain spaces and
/// the protocol splits on whitespace. Hex, not a custom escape grammar: it is
/// the same choice the browser broker made for its staged downloads, and it
/// cannot be ambiguous.
fn serve_stdio(mut worker: Worker) -> ExitCode {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in BufReader::new(stdin.lock()).lines() {
        let Ok(line) = line else {
            return ExitCode::FAILURE;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let verb = parts.next().unwrap_or("");
        match verb.to_ascii_uppercase().as_str() {
            "PING" => {
                if writeln!(out, "OK 1\nversion={}", env!("CARGO_PKG_VERSION")).is_err() {
                    return ExitCode::FAILURE;
                }
                let _ = out.flush();
            }
            "QUIT" => {
                let _ = writeln!(out, "OK 0");
                let _ = out.flush();
                return ExitCode::SUCCESS;
            }
            "SCAN" => {
                let Some(raw) = parts.next() else {
                    let _ = writeln!(out, "ERR bad-argument missing-path");
                    let _ = out.flush();
                    continue;
                };
                let Some(bytes) = unhex(raw) else {
                    let _ = writeln!(out, "ERR bad-argument path-not-hex");
                    let _ = out.flush();
                    continue;
                };
                let Ok(path) = String::from_utf8(bytes) else {
                    let _ = writeln!(out, "ERR bad-argument path-not-utf8");
                    let _ = out.flush();
                    continue;
                };
                match scan_path(&mut worker, &path) {
                    Ok(report) => {
                        let lines = report_lines(&report);
                        if writeln!(out, "OK {}", lines.len()).is_err() {
                            return ExitCode::FAILURE;
                        }
                        for line in &lines {
                            if writeln!(out, "{line}").is_err() {
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                    Err(error) => {
                        if writeln!(out, "ERR {error}").is_err() {
                            return ExitCode::FAILURE;
                        }
                    }
                }
                let _ = out.flush();
            }
            other => {
                let _ = writeln!(out, "ERR bad-argument unknown-verb {other}");
                let _ = out.flush();
            }
        }
    }
    ExitCode::SUCCESS
}

/// Scan one path, reading at most [`MAX_SCAN_BYTES`] plus one byte.
fn scan_path(worker: &mut Worker, path: &str) -> Result<ScanReport, String> {
    let meta = std::fs::metadata(path).map_err(|error| format!("cannot-stat {error}"))?;
    if !meta.is_file() {
        return Err("not-a-regular-file".to_owned());
    }
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());

    if size > MAX_SCAN_BYTES {
        return Ok(unscannable(worker.mode, size, "too-large"));
    }

    let file = File::open(path).map_err(|error| format!("cannot-open {error}"))?;
    // +1 so a file that grew past the ceiling between stat and read is caught
    // instead of silently truncated.
    let mut buffer = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
    file.take(MAX_SCAN_BYTES.saturating_add(1))
        .read_to_end(&mut buffer)
        .map_err(|error| format!("cannot-read {error}"))?;
    if u64::try_from(buffer.len()).unwrap_or(u64::MAX) > MAX_SCAN_BYTES {
        return Ok(unscannable(worker.mode, size, "too-large"));
    }
    Ok(worker.engine.scan(path, size, mtime, &buffer))
}

fn unscannable(mode: Mode, size: u64, reason: &str) -> ScanReport {
    ScanReport {
        decision: resolve(
            &Observation::Unscannable {
                reason: reason.to_owned(),
            },
            mode,
        ),
        digest_hex: String::new(),
        size,
        from_cache: false,
    }
}

/// One line per verdict, in the `key=value` shape the rest of the system uses.
fn report_lines(report: &ScanReport) -> Vec<String> {
    let (verdict, tier, id, reason) = match &report.decision {
        Decision::Allow => ("allow", "-".to_owned(), "-".to_owned(), "-".to_owned()),
        Decision::AllowUnknown { reason } => {
            ("unknown", "-".to_owned(), "-".to_owned(), clean(reason))
        }
        Decision::Deny { tier, id } => (
            "deny",
            match tier {
                Tier::Hash => "hash".to_owned(),
                Tier::Pattern => "pattern".to_owned(),
            },
            clean(id),
            "-".to_owned(),
        ),
    };
    vec![format!(
        "verdict={verdict} tier={tier} id={id} reason={reason} size={} digest={} from_cache={}",
        report.size,
        if report.digest_hex.is_empty() {
            "-"
        } else {
            report.digest_hex.as_str()
        },
        u8::from(report.from_cache)
    )]
}

/// Replace whitespace so one verdict stays one line and one field.
fn clean(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_whitespace() { '_' } else { c })
        .collect()
}

fn exit_for(report: &ScanReport) -> ExitCode {
    match report.decision {
        Decision::Deny { .. } => ExitCode::from(1),
        _ => ExitCode::SUCCESS,
    }
}

fn unhex(raw: &str) -> Option<Vec<u8>> {
    let bytes = raw.as_bytes();
    if !bytes.chunks_exact(2).remainder().is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len().saturating_div(2));
    for pair in bytes.chunks_exact(2) {
        let mut pair = pair.iter().copied();
        let hi = hex_value(pair.next()?)?;
        let lo = hex_value(pair.next()?)?;
        out.push(hi.saturating_mul(16) | lo);
    }
    Some(out)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(byte.wrapping_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Some(byte.wrapping_sub(b'A').saturating_add(10)),
        _ => None,
    }
}
