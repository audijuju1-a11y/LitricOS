//! Render the boot intro frame by frame, from the shell's own drawing code.
//!
//! The intro draws only the wordmark, so this needs no installed fonts. The
//! frames are written as P6 PPM files; `tools/ppm2png.py` or Pillow turns them
//! into images or an animated GIF.
//!
//! ```text
//! cargo run -p litric-shell --example boot_intro_dump -- OUT_DIR
//! ```

#![forbid(unsafe_code)]

use std::io::Write;

use litric_design::{Theme, ThemeKind};
use litric_locale::Regional;
use litric_shell::canvas::Canvas;
use litric_shell::shell::{Shell, BOOT_INTRO_MS};
use litric_shell::ui::Tokens;

/// Milliseconds between captured frames: ten frames a second.
const STEP_MS: u64 = 100;

fn write_ppm(path: &str, canvas: &Canvas) -> std::io::Result<()> {
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(out, "P6\n{} {}\n255\n", canvas.width(), canvas.height())?;
    for &p in canvas.pixels() {
        out.write_all(&[(p >> 16) as u8, (p >> 8) as u8, p as u8])?;
    }
    out.flush()
}

fn main() -> std::io::Result<()> {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "boot-intro".to_owned());
    std::fs::create_dir_all(&dir)?;

    let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
    let mut shell = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("boot"));
    shell.start_boot_intro(0);

    // Stop before the intro ends: the desktop needs a font, and this example
    // is about the mark, not the handover.
    let mut t: u64 = 0;
    let mut index: usize = 0;
    while t < BOOT_INTRO_MS {
        shell.tick(t);
        let mut canvas = Canvas::new(1280, 800, 0);
        litric_shell::views::draw(&mut canvas, &mut shell);
        write_ppm(&format!("{dir}/{index:02}.ppm"), &canvas)?;
        t = t.saturating_add(STEP_MS);
        index = index.saturating_add(1);
    }
    println!("wrote {index} frames to {dir}");
    Ok(())
}
