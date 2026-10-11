//! Times `litric_browser::jpeg::decode` on one file.
//!
//! `jpeg-bench FILE [ROUNDS]` decodes the file ROUNDS times (default 50) after
//! one untimed warm-up decode and prints the mean and best wall-clock time.
//! Build with `--release` for numbers that mean anything.

use std::process::ExitCode;
use std::time::Instant;

use litric_browser::jpeg;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: jpeg-bench FILE [ROUNDS]");
        return ExitCode::from(2);
    };
    let rounds: u32 = args
        .next()
        .and_then(|r| r.parse().ok())
        .filter(|&r| r > 0)
        .unwrap_or(50);
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (width, height) = match jpeg::decode(&data) {
        Ok(image) => (image.width, image.height),
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut best = f64::MAX;
    let mut total = 0.0f64;
    for _ in 0..rounds {
        let started = Instant::now();
        if let Err(e) = jpeg::decode(&data) {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        best = best.min(ms);
        total += ms;
    }
    let mean = total / f64::from(rounds);
    println!(
        "{path}: {width}x{height}, {} bytes, {rounds} rounds: mean {mean:.2} ms, best {best:.2} ms",
        data.len()
    );
    ExitCode::SUCCESS
}
