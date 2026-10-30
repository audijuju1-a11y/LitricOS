//! What a real compositor costs, in bytes the kernel reports.
//!
//! `docs/PERF-budget.md` §3d counts canvases by arithmetic (4 bytes a pixel).
//! That is the right figure for a budget and the wrong one for a claim: it says
//! what the buffers should weigh, not what this process allocated. This probe
//! builds the shipped [`Compositor`] at each mode the shell accepts, and prints
//! `VmRSS` before and after, so the difference is the resident cost the kernel
//! measured rather than the product of a multiplication.
//!
//! ```text
//! cargo run --release -p litric-shell --example canvas_bytes
//! ```
//!
//! Nothing is drawn after construction: the number is what holding a desktop
//! costs at rest, which is what `litric-shell.slice`'s `MemoryMax` has to cover.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use litric_design::{Theme, ThemeKind};
use litric_shell::desktop::{Compositor, Wallpaper};

/// Resident set size in KiB, from `/proc/self/statm` (field 2 is resident pages).
fn rss_kb() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = text.split_whitespace().nth(1)?.parse().ok()?;
    // 4 KiB a page on every target this shell runs on; stated rather than
    // probed, because a wrong page size would make the figure conservative.
    Some(pages.saturating_mul(4))
}

/// `WxH`, default 1280x800. One mode per process, because a second compositor
/// in the same process would reuse the first one's freed heap and the RSS
/// difference would then describe the allocator, not the desktop.
fn mode(arg: Option<String>) -> (u32, u32) {
    let Some(text) = arg else {
        return (1280, 800);
    };
    let mut parts = text.split('x');
    let w = parts.next().and_then(|v| v.parse().ok());
    let h = parts.next().and_then(|v| v.parse().ok());
    match (w, h) {
        (Some(w), Some(h)) => (w, h),
        _ => (1280, 800),
    }
}

fn main() -> ExitCode {
    let (w, h) = mode(std::env::args().nth(1));
    let theme = Theme::of(ThemeKind::Dark);
    let before = rss_kb().unwrap_or(0);
    let compositor = Compositor::new(w, h, &theme, Wallpaper::System);
    let after = rss_kb().unwrap_or(0);
    // Touch the frame so the buffers are resident, not merely reserved.
    let touched = compositor.scene.pixels().len();
    let after_touch = rss_kb().unwrap_or(0);
    println!(
        "mode={w}x{h} canvas_bytes={} rss_before_kb={before} rss_after_kb={after} \
         compositor_kb={} rss_after_touch_kb={after_touch} scene_px={touched}",
        u64::from(w).saturating_mul(u64::from(h)).saturating_mul(4),
        after.saturating_sub(before)
    );
    ExitCode::SUCCESS
}
