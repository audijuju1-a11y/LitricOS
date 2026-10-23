//! Write the LITRICOS wordmark as an SVG file, from the same strokes the shell draws.
//!
//! ```text
//! cargo run -p litric-shell --example wordmark_svg -- ../assets/brand/litric-wordmark.svg
//! ```
//!
//! A test holds the committed file to this output, so regenerate it here
//! whenever the strokes change.

#![forbid(unsafe_code)]

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "litric-wordmark.svg".to_owned());
    if let Err(err) = std::fs::write(&out, litric_shell::wordmark::svg("#F4F2ED")) {
        eprintln!("wordmark_svg: cannot write {out}: {err}");
        std::process::exit(1);
    }
    println!("wrote {out}");
}
