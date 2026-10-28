//! Render benchmark for the desktop's frame path.
//!
//! Builds the first-run desktop `litric-shell` shows at start (the Activity and
//! Environments windows open, Activity focused) at 1280x800 and times the frame
//! path the binary's loop uses: a full frame, the partial frame after one
//! pointer move, the frame after opening a window, and presenting to a regular
//! file that stands in for `/dev/fb0`.
//!
//! The scene carries the glass backdrop [`desktop::Compositor::new`] attaches,
//! so every glass panel copies out of the pre-blurred source the shipped desktop
//! samples. One extra scene, `full_frame_live_blur`, times the fallback a canvas
//! with no backdrop takes: that is the live blur the tracker's §8-§11 measured
//! while this benchmark still built its scene without the backdrop.
//!
//! The lock screen is measured last and in four states -- as it opens, with the
//! password field out, with the customize sheet open, and with motion off --
//! because a locked shell composes in full on every tick and is therefore the
//! most expensive steady state the shell has.
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
use litric_shell::canvas::{rgb, with_alpha, Canvas, Rect, Site};
use litric_shell::desktop;
use litric_shell::fb::{FbInfo, Framebuffer};
use litric_shell::font::{self, Style};
use litric_shell::lock::Stage;
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
    /// The blurred wallpaper the scene samples, kept so a scene can be switched
    /// between the shipped path and the no-backdrop fallback and back again.
    backdrop: std::rc::Rc<Canvas>,
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
    // `Compositor::new` blurs and saturates the wallpaper once and attaches it,
    // and every glass panel then copies from it. Building the scene without
    // that step measured the fallback (live blur) instead of the frame the
    // shell runs; `glass_params` is the compositor's own derivation so the two
    // cannot drift apart.
    let (blur, saturation) = desktop::glass_params(&theme);
    let backdrop = std::rc::Rc::new(wallpaper.glass_source(blur, saturation));
    let mut scene = wallpaper.clone();
    let _ = scene.set_backdrop(Some(std::rc::Rc::clone(&backdrop)));
    Bench {
        shell,
        wallpaper,
        scene,
        backdrop,
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
///
/// A locked frame restores nothing, mirroring [`Compositor::repaint`], which
/// skips the same copy for the same reason: the lock paints an opaque
/// background over every pixel it visits, so the wallpaper underneath it could
/// not be seen. A benchmark that kept restoring would be timing work the shell
/// does not do.
///
/// [`Compositor::repaint`]: litric_shell::desktop::Compositor::repaint
fn frame(b: &mut Bench) -> Frame {
    let full = b.shell.needs_full_redraw();
    let damage = b.shell.take_damage();
    let screen = Rect::new(0.0, 0.0, W as f32, H as f32);
    let mut restored_px = 0.0_f64;
    b.scene.reset_pixels_visited();
    if b.shell.lock.locked {
        // Nothing to restore.
    } else if full {
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

/// Time the lock screen in whatever state it is in now, and say what one frame
/// of it did.
///
/// The interval is the one [`litric_shell::shell::LifeState::Locked`] asks the
/// loop for, so the frames are spaced the way the loop spaces them.
fn lock_frames(b: &mut Bench, now: &mut u64, name: &str) {
    let mut times = Vec::with_capacity(RUNS);
    let mut full = false;
    let mut damage = 0_usize;
    let mut restored = 0.0_f64;
    let mut visited = 0_u64;
    // Where the pixels went, summed over the runs. BUG-0018 asks for this
    // split: the total alone does not say whether a locked frame spent its
    // work on the glass blur or on the fields and the glow.
    let mut sites = [0_u64; Site::COUNT];
    for _ in 0..RUNS {
        *now = (*now).saturating_add(33);
        b.shell.tick(*now);
        let t = Instant::now();
        let f = frame(b);
        times.push(elapsed_ms(t));
        full = f.full;
        damage = f.damage.len();
        restored = f.restored_px;
        visited = f.visited_px;
        for (s, n) in b.scene.pixels_visited_by_site() {
            if let Some(c) = sites.get_mut(s.index()) {
                *c = c.saturating_add(n);
            }
        }
    }
    println!(
        "{name}.full={full} damage_rects={damage} restored_px={restored:.0} visited_px={visited}"
    );
    let split = Site::ALL
        .into_iter()
        .filter_map(|s| {
            sites.get(s.index()).copied().filter(|n| *n > 0).map(|n| {
                let per_run = n.checked_div(RUNS as u64).unwrap_or(0);
                format!("{}={per_run}", s.label())
            })
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!("{name}.sites_per_frame={split}");
    report(name, times);
}

fn elapsed_ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

/// Print the statistics a budget line needs.
///
/// `docs/PERF-budget.md` Appendix §3: a target becomes a measurement only with
/// the tool, the environment, `n`, the mean and the standard deviation. The
/// median and the minimum were here first and stay: on a shared development host
/// an outlier inflates the mean, and the median is the number that describes the
/// code rather than the machine's mood.
fn report(name: &str, mut v: Vec<f64>) {
    let n = v.len();
    #[allow(clippy::cast_precision_loss)]
    let count = n as f64;
    let mean = if n > 0 {
        v.iter().sum::<f64>() / count.max(1.0)
    } else {
        f64::NAN
    };
    // The population deviation (divide by n, not n-1): these are all the runs
    // there were, not a sample of a larger set.
    let variance = if n > 0 {
        v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / count.max(1.0)
    } else {
        f64::NAN
    };
    v.sort_by(f64::total_cmp);
    let min = v.first().copied().unwrap_or(f64::NAN);
    let median = v
        .get(v.len().checked_div(2).unwrap_or(0))
        .copied()
        .unwrap_or(f64::NAN);
    println!("{name}.min_ms={min:.3}");
    println!("{name}.median_ms={median:.3}");
    println!("{name}.mean_ms={mean:.3}");
    println!("{name}.stddev_ms={:.3}", variance.sqrt());
    println!("{name}.runs={n}");
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
    // Where a full frame's pixels went, summed over the runs: the number that
    // says which loop to profile rather than which one looks expensive.
    let mut sites = [0_u64; Site::COUNT];
    for _ in 0..RUNS {
        b.shell.request_full_redraw();
        let t = Instant::now();
        let f = frame(&mut b);
        times.push(elapsed_ms(t));
        visited = f.visited_px;
        for (s, n) in b.scene.pixels_visited_by_site() {
            if let Some(c) = sites.get_mut(s.index()) {
                *c = c.saturating_add(n);
            }
        }
    }
    println!("full_frame.visited_px={visited}");
    let split = Site::ALL
        .into_iter()
        .filter_map(|s| {
            sites.get(s.index()).copied().filter(|n| *n > 0).map(|n| {
                let per_run = n.checked_div(RUNS as u64).unwrap_or(0);
                format!("{}={per_run}", s.label())
            })
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!("full_frame.sites_per_frame={split}");
    report("full_frame", times);

    // ---- the same frame on the fallback path ------------------------------
    //
    // A canvas with no backdrop blurs the live pixels around each panel. The
    // shipped desktop does not take this path for glass, but §8-§11 measured it
    // (this benchmark used to build its scene without a backdrop), and a
    // surface that swaps its backdrop for one of its own -- the lock's bloom --
    // still can. Keeping it as its own scene makes the difference between the
    // two rows the measured cost of the live blur.
    let _ = b.scene.set_backdrop(None);
    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        b.shell.request_full_redraw();
        let t = Instant::now();
        let _ = frame(&mut b);
        times.push(elapsed_ms(t));
    }
    report("full_frame_live_blur", times);
    let _ = b.scene.set_backdrop(Some(std::rc::Rc::clone(&b.backdrop)));

    // One frame with the call timers on. Enabled only here, after the timed
    // loop, so the numbers above pay nothing for instrumentation; this pass is
    // about which drawing entry point the frame spends its time in.
    litric_shell::canvas::profile::enable();
    b.shell.request_full_redraw();
    let _ = frame(&mut b);
    litric_shell::canvas::profile::disable();
    let mut rows = litric_shell::canvas::profile::take();
    rows.sort_by_key(|row| std::cmp::Reverse(row.nanos));
    let timed: u64 = rows.iter().map(|row| row.nanos).sum();
    for row in &rows {
        let ms = row.nanos as f64 / 1_000_000.0;
        let share = if timed > 0 {
            row.nanos as f64 * 100.0 / timed as f64
        } else {
            0.0
        };
        let per_px = if row.pixels > 0 {
            row.nanos as f64 / row.pixels as f64
        } else {
            0.0
        };
        println!(
            "profile.{}.calls={} profile.{}.ms={ms:.3} profile.{}.share_pct={share:.1} profile.{}.px={} profile.{}.ns_per_px={per_px:.2}",
            row.name, row.calls, row.name, row.name, row.name, row.pixels, row.name
        );
    }
    for (name, w, h) in litric_shell::canvas::profile::take_dims() {
        println!("profile.{name}.dims={w}x{h}");
    }
    // Every glass rectangle in call order. Two panels of one size class are
    // different rectangles, and whether the frame blurred any pixel twice is a
    // question about these, not about the bucket totals.
    let mut rects = Vec::new();
    for (name, r) in litric_shell::canvas::profile::take_rects() {
        println!(
            "profile.{name}.rect={:.1},{:.1},{:.1}x{:.1}",
            r.x, r.y, r.w, r.h
        );
        rects.push(r);
    }
    let mut overlapping_pairs = 0_usize;
    let mut overlap_px = 0.0_f64;
    for (i, a) in rects.iter().enumerate() {
        for other in rects.iter().skip(i.saturating_add(1)) {
            if let Some(x) = a.intersection(other) {
                overlapping_pairs = overlapping_pairs.saturating_add(1);
                overlap_px += f64::from(x.w * x.h);
            }
        }
    }
    let area_sum: f64 = rects.iter().map(|r| f64::from(r.w * r.h)).sum();
    println!(
        "profile.glass_rects.count={} profile.glass_rects.overlapping_pairs={overlapping_pairs} profile.glass_rects.overlap_px={overlap_px:.0} profile.glass_rects.area_sum_px={area_sum:.0}",
        rects.len()
    );
    println!("profile.timed_ms={:.3}", timed as f64 / 1_000_000.0);

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

    // ---- the lock screen --------------------------------------------------
    //
    // A locked shell composes in full on every tick: its colour fields drift
    // and the clock breathes, so the lock is the most expensive steady state
    // LitricOS has, and this is the number `docs/PERF-budget.md` holds the lock
    // surface to. Each variant is the same drawing path in a different state,
    // so the difference between them is what that state costs.
    let mut locked = build();
    locked.shell.facts.auth_available = Some("unix-chkpwd".to_owned());
    locked.shell.facts.auth_lock = Some(false);
    locked.shell.facts.auth_account = Some("litric".to_owned());
    locked.shell.facts.auth_password = Some("set".to_owned());
    locked.shell.facts.session_user = Some("litric".to_owned());
    if !locked.shell.lock_now() {
        eprintln!("render_bench: the fixture cannot check a password, so it cannot lock");
        return ExitCode::FAILURE;
    }
    println!("lock.interval_ms={}", locked.shell.frame_interval_ms());
    lock_frames(&mut locked, &mut now, "lock_frame");

    // The password field out: a glass sheet over the same drifting frame.
    locked.shell.lock.stage = Stage::Entry;
    lock_frames(&mut locked, &mut now, "lock_frame_entry");

    // And with everything the screen can put on one frame at once: the
    // customize sheet over the drifting fields, password field still out.
    locked.shell.lock.edit = true;
    lock_frames(&mut locked, &mut now, "lock_frame_customize");

    // "Still" stops the drift, and the clock stops breathing with it. The loop
    // still composes one full frame per interval while the session is locked,
    // so this is what a locked screen with no motion costs.
    locked.shell.lock.edit = false;
    locked.shell.lock.tweaks.motion = 0;
    locked.shell.lock.tweaks.breathe = false;
    lock_frames(&mut locked, &mut now, "lock_frame_still");

    // ---- primitives -------------------------------------------------------
    //
    // A scene costs what its loops cost. Each primitive below is the one the
    // desktop actually calls, timed on the same 1280x800 canvas, so the
    // full-frame number can be attributed to a primitive instead of guessed.
    // The panel is a window-sized rectangle: the size the chrome draws.
    let screen = Rect::new(0.0, 0.0, W as f32, H as f32);
    let panel = Rect::new(340.0, 190.0, 600.0, 420.0);

    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        b.scene.blit_from(&b.wallpaper, screen);
        times.push(elapsed_ms(t));
    }
    println!("primitive_blit_full_screen.size={W}x{H}");
    report("primitive_blit_full_screen", times);

    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        b.scene.fill_round_rect(panel, 16.0, rgb(0x202430));
        times.push(elapsed_ms(t));
    }
    println!("primitive_fill_panel.size=600x420");
    report("primitive_fill_panel", times);

    // The same panel in a translucent colour. Its core cannot take the opaque
    // fill path, so this row isolates the per-pixel blend that dominates the
    // residual `fill_round_rect` time; the opaque row above is the control.
    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        b.scene
            .fill_round_rect(panel, 16.0, with_alpha(rgb(0x202430), 0.37));
        times.push(elapsed_ms(t));
    }
    println!("primitive_fill_panel_translucent.size=600x420");
    report("primitive_fill_panel_translucent", times);

    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        b.scene.soft_shadow(panel, 16.0, 36.0, 10.0, 0.5);
        times.push(elapsed_ms(t));
    }
    report("primitive_soft_shadow_panel", times);

    let source = std::rc::Rc::new(b.wallpaper.glass_source(6, 1.2));
    let _ = b.scene.set_backdrop(Some(source));
    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        b.scene.glass_backdrop(panel, 16.0, 6, 1.2);
        times.push(elapsed_ms(t));
    }
    report("primitive_glass_backdrop_panel", times);

    let style = Style::new(24.0, rgb(0xFFFFFF));
    let sample = "The quick brown fox jumps over the lazy dog 0123456789";
    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        for i in 0..10_u32 {
            let baseline = 100.0 + f32::from(u16::try_from(i).unwrap_or(0)) * 30.0;
            font::draw_literal(&mut b.scene, sample, 40.0, baseline, &style);
        }
        times.push(elapsed_ms(t));
    }
    report("primitive_text_10_lines", times);

    ExitCode::SUCCESS
}
