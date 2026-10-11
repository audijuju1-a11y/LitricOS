//! Render benchmark for the desktop's frame path.
//!
//! Builds the first-run desktop `litric-shell` shows at start (the Activity and
//! Environments windows open, Activity focused) at 1280x800 and times the frame
//! path the binary's loop uses: a full frame, the partial frame after one
//! pointer move, the frame after opening a window, and presenting to a regular
//! file that stands in for `/dev/fb0`.
//!
//! The system facts are fixed numbers set here, not readings: the benchmark
//! measures drawing, and drawing the same text on every run is what makes two
//! runs comparable.
//!
//! ```text
//! cargo run --release -p litric-shell --example render_bench -- [DIR]
//! ```
//!
//! `DIR` is where the framebuffer stand-in file is written (default: the
//! system temporary directory).

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use litric_design::{Theme, ThemeKind};
use litric_input::event::{InputEvent, Point};
use litric_locale::Regional;
use litric_shell::canvas::{rgb, Canvas, Rect};
use litric_shell::desktop;
use litric_shell::fb::{FbInfo, Framebuffer};
use litric_shell::shell::{DesktopMode, EnvRow, Shell};
use litric_shell::shortcuts::App;
use litric_shell::ui::Tokens;
use litric_shell::views;

const W: u32 = 1280;
const H: u32 = 800;
const RUNS: usize = 15;

struct Bench {
    shell: Shell,
    wallpaper: Canvas,
    scene: Canvas,
}

fn env_row(token: &str, title: &str, state: &str, hotkey: &str) -> EnvRow {
    EnvRow {
        token: token.to_owned(),
        title: title.to_owned(),
        state: state.to_owned(),
        active: state == "running",
        hotkey: hotkey.to_owned(),
    }
}

/// The desktop `main.rs` `start()` builds, with fixed facts.
fn build() -> Bench {
    let theme = Theme::of(ThemeKind::Dark);
    let scale = 1.0_f32;
    let mut shell = Shell::new(
        Tokens::new(theme, scale),
        W as f32,
        H as f32,
        Regional::fallback("render-bench"),
    );
    shell.mode = DesktopMode::Ready;
    shell.facts.mem_total_mb = Some(7867);
    shell.facts.mem_used_mb = Some(527);
    shell.facts.cpus = Some(8);
    shell.facts.uptime_secs = Some(4_000.0);
    shell.facts.cpu_fraction = Some(0.07);
    shell.facts.root_fs = Some("erofs".to_owned());
    shell.facts.verity = Some(true);
    shell.facts.source = "render-bench fixture".to_owned();
    shell.environments = vec![
        env_row("windows", "Windows 11", "not-installed", "Win+1"),
        env_row("litric", "LitricOS", "running", "Win+2"),
        env_row("linux", "Linux", "not-installed", "Win+3"),
    ];

    // The same placement as main.rs start().
    let work = shell.wm.work_area();
    let pad = 64.0 * scale;
    let panel_w = (work.w * 0.34).clamp(320.0 * scale, 520.0 * scale);
    let activity = litric_wm::Rect::new(
        work.x + pad,
        work.y + pad,
        panel_w,
        (work.h * 0.46).max(260.0 * scale),
    );
    let environments = litric_wm::Rect::new(
        work.right() - pad - panel_w,
        work.y + pad,
        panel_w,
        (work.h * 0.34).max(220.0 * scale),
    );
    shell.wm.open_at(
        App::Environments.app_id(),
        App::Environments.title(),
        environments,
    );
    let act = shell
        .wm
        .open_at(App::Activity.app_id(), App::Activity.title(), activity);
    shell.wm.focus_window(act);

    let mut wallpaper = Canvas::new(W, H, rgb(0x000000));
    desktop::wallpaper(&mut wallpaper, &theme);
    let scene = wallpaper.clone();
    Bench {
        shell,
        wallpaper,
        scene,
    }
}

/// What one frame did.
struct Frame {
    full: bool,
    damage: Vec<Rect>,
    restored_px: f64,
    visited_px: u64,
}

/// One frame exactly as `litric-shell`'s loop drew it at the time of writing:
/// restore the wallpaper (everywhere, or inside each damage rectangle grown by
/// two pixels), then draw the whole desktop unclipped.
fn frame(b: &mut Bench) -> Frame {
    let full = b.shell.needs_full_redraw();
    let damage = b.shell.take_damage();
    let screen = Rect::new(0.0, 0.0, W as f32, H as f32);
    let mut restored_px = 0.0_f64;
    b.scene.reset_pixels_visited();
    if full {
        b.scene.blit_from(&b.wallpaper, screen);
        restored_px = f64::from(screen.w * screen.h);
    } else {
        for r in &damage {
            let grown = r.inflate(2.0);
            b.scene.blit_from(&b.wallpaper, grown);
            restored_px += screen
                .intersection(&grown)
                .map_or(0.0, |i| f64::from(i.w * i.h));
        }
    }
    views::draw(&mut b.scene, &mut b.shell);
    b.shell.counted_frame();
    Frame {
        full,
        damage,
        restored_px,
        visited_px: b.scene.pixels_visited(),
    }
}

fn elapsed_ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn report(name: &str, mut v: Vec<f64>) {
    v.sort_by(f64::total_cmp);
    let min = v.first().copied().unwrap_or(f64::NAN);
    let median = v
        .get(v.len().checked_div(2).unwrap_or(0))
        .copied()
        .unwrap_or(f64::NAN);
    println!("{name}.min_ms={min:.3}");
    println!("{name}.median_ms={median:.3}");
    println!("{name}.runs={}", v.len());
}

fn changed(a: &[u32], b: &[u32]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

fn main() -> ExitCode {
    let dir = std::env::args()
        .nth(1)
        .map_or_else(std::env::temp_dir, PathBuf::from);
    let mut b = build();
    println!("canvas={W}x{H}");

    // The first frame, as main.rs draws it at start.
    let mut now = 1_000_u64;
    b.shell.tick(now);
    let _ = frame(&mut b);

    // ---- full frames ------------------------------------------------------
    let mut times = Vec::with_capacity(RUNS);
    let mut visited = 0;
    for _ in 0..RUNS {
        b.shell.request_full_redraw();
        let t = Instant::now();
        let f = frame(&mut b);
        times.push(elapsed_ms(t));
        visited = f.visited_px;
    }
    println!("full_frame.visited_px={visited}");
    report("full_frame", times);

    // ---- one pointer move over empty desktop ------------------------------
    let mut times = Vec::with_capacity(RUNS);
    let mut pointer_damage = Vec::new();
    for i in 0..RUNS {
        now = now.saturating_add(16);
        let at = if i % 2 == 0 {
            Point::new(640.0, 560.0)
        } else {
            Point::new(648.0, 566.0)
        };
        b.shell.handle(
            &InputEvent::PointerMove {
                at,
                delta: Point::new(8.0, 6.0),
            },
            now,
        );
        b.shell.tick(now);
        let before = b.scene.pixels().to_vec();
        let t = Instant::now();
        let f = frame(&mut b);
        times.push(elapsed_ms(t));
        let n = changed(&before, b.scene.pixels());
        if i == 0 || i == RUNS.saturating_sub(1) {
            println!(
                "pointer_move.run{i}.full={} damage_rects={} restored_px={:.0} changed_px={n} visited_px={}",
                f.full,
                f.damage.len(),
                f.restored_px,
                f.visited_px
            );
        }
        pointer_damage = f.damage;
    }
    report("pointer_move_frame", times);

    // ---- opening a window -------------------------------------------------
    let mut times = Vec::with_capacity(RUNS);
    for i in 0..RUNS {
        now = now.saturating_add(16);
        b.shell.open_app(App::Files);
        b.shell.tick(now);
        let before = b.scene.pixels().to_vec();
        let t = Instant::now();
        let f = frame(&mut b);
        times.push(elapsed_ms(t));
        let n = changed(&before, b.scene.pixels());
        if i == 0 || i == RUNS.saturating_sub(1) {
            println!(
                "open_window.run{i}.full={} damage_rects={} restored_px={:.0} changed_px={n} visited_px={}",
                f.full,
                f.damage.len(),
                f.restored_px,
                f.visited_px
            );
        }
        if let Some(id) = b.shell.wm.focused() {
            b.shell.wm.close(id);
        }
        now = now.saturating_add(16);
        b.shell.tick(now);
        let _ = frame(&mut b);
    }
    report("open_window_frame", times);

    // ---- the box blur on its own -------------------------------------------
    for radius in [3_u32, 10, 30] {
        let mut times = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let mut c = b.wallpaper.clone();
            let t = Instant::now();
            c.blur_region(Rect::new(0.0, 0.0, W as f32, H as f32), radius, 2);
            times.push(elapsed_ms(t));
        }
        report(&format!("blur_full_screen_r{radius}_2passes"), times);
    }

    // ---- presenting -------------------------------------------------------
    let dev = dir.join("render-bench-fb0.bin");
    let stride = W.saturating_mul(4);
    let frame_bytes = (stride as usize).saturating_mul(H as usize);
    if let Err(e) = fs::write(&dev, vec![0u8; frame_bytes]) {
        eprintln!("render_bench: cannot create {}: {e}", dev.display());
        return ExitCode::FAILURE;
    }
    let Some(info) = FbInfo::parse(&format!("{W},{H}"), "32", &stride.to_string(), "bench") else {
        eprintln!("render_bench: framebuffer geometry did not parse");
        return ExitCode::FAILURE;
    };
    let mut fb = match Framebuffer::open(&dev, info) {
        Ok(fb) => fb,
        Err(e) => {
            eprintln!("render_bench: cannot open {}: {e}", dev.display());
            return ExitCode::FAILURE;
        }
    };
    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        if let Err(e) = fb.present(&b.scene) {
            eprintln!("render_bench: present failed: {e}");
            return ExitCode::FAILURE;
        }
        times.push(elapsed_ms(t));
    }
    println!("present_full.bytes={frame_bytes}");
    report("present_full", times);

    let mut times = Vec::with_capacity(RUNS);
    let mut rows = 0u32;
    for _ in 0..RUNS {
        let t = Instant::now();
        match fb.present_damage(&b.scene, &pointer_damage) {
            Ok(n) => rows = n,
            Err(e) => {
                eprintln!("render_bench: present_damage failed: {e}");
                return ExitCode::FAILURE;
            }
        }
        times.push(elapsed_ms(t));
    }
    println!(
        "present_pointer_damage.rows={rows} bytes={}",
        u64::from(rows).saturating_mul(u64::from(stride))
    );
    report("present_pointer_damage", times);

    let _ = fs::remove_file(&dev);
    ExitCode::SUCCESS
}
