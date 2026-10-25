//! Render benchmark for the desktop's frame path.
//!
//! Builds the first-run desktop `litric-shell` shows at start (the Activity and
//! Environments windows open, Activity focused) at 1280x800 and times the frame
//! path the binary's loop uses: a full frame, the partial frame after one
//! pointer move, the frame after opening a window, and presenting to a regular
//! file that stands in for `/dev/fb0`.
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
use litric_input::event::{InputEvent, Point, PointerButton};
use litric_input::keys::Modifiers;
use litric_locale::Regional;
use litric_shell::canvas::{rgb, Canvas, Rect, Site};
use litric_shell::desktop;
use litric_shell::fb::{FbInfo, Framebuffer};
use litric_shell::lock::Stage;
use litric_shell::shell::{DesktopMode, EnvRow, Hot, Shell};
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

/// Click a control the last drawn frame published, the way a pointer would.
///
/// The hotspot registry is the only route into the shell's controls from
/// outside the crate, and it is also the honest one: it is the same rectangle a
/// person aims at.
fn click(b: &mut Bench, now: &mut u64, hot: Hot) -> bool {
    let Some(rect) = b.shell.hotspots().rect_of(hot) else {
        eprintln!("render_bench: {hot:?} is not on screen, so the reveal cannot be measured");
        return false;
    };
    let (x, y) = rect.center();
    let at = Point::new(x, y);
    for ev in [
        InputEvent::PointerMove {
            at,
            delta: Point::new(1.0, 1.0),
        },
        InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: true,
            at,
            clicks: 1,
            mods: Modifiers::NONE,
        },
        InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: false,
            at,
            clicks: 1,
            mods: Modifiers::NONE,
        },
    ] {
        *now = (*now).saturating_add(4);
        b.shell.handle(&ev, *now);
    }
    true
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

    // ---- a panel's reveal, through the compositor --------------------------
    //
    // Clicking Quick Settings travels a `FAST` spring while the panel grows its
    // height from its own top edge. Two kinds of frame come out of that, and
    // before the draw side published the rectangle it paints in, every one of
    // them was the first kind:
    //
    // * `panel_reveal_full` -- frames that repainted the whole screen. Now at
    //   most one per open: the frame that first paints the panel, before any
    //   rectangle exists to trust.
    // * `panel_reveal_damaged` -- frames that repainted only what
    //   `Shell::note_panel` said the panel drew in, plus the reach of its
    //   shadow.
    //
    // Measured through `Compositor::repaint` and not through `frame` above,
    // because `frame` calls `views::draw` unclipped: its partial-frame figure is
    // an upper bound on the loop's cost, not the cost. The comparison that
    // matters is these two against each other -- the same reveal, drawn two
    // ways -- and `every_panel_reveal_is_a_partial_frame_that_matches_a_full_
    // redraw` (main.rs) holds the second kind to a byte of the first.
    //
    // The desktop is left to come to rest first: the section above opened and
    // closed a window fifteen times, and a window that closes mid-reveal leaves
    // a focus spring travelling. Measuring a panel against that would be
    // measuring two animations at once.
    for _ in 0..40 {
        now = now.saturating_add(16);
        b.shell.tick(now);
        let _ = b.shell.take_damage();
    }
    let mut comp = desktop::Compositor::new(W, H, &b.shell.tokens.theme);
    let _ = comp.repaint(&mut b.shell, true, &[]);
    let mut full_frames: Vec<f64> = Vec::new();
    let mut panel_frames: Vec<f64> = Vec::new();
    let mut panel_px = 0.0_f64;
    for _ in 0..10_u32 {
        now = now.saturating_add(16);
        if !click(&mut b, &mut now, Hot::MenuEnvironment) {
            return ExitCode::FAILURE;
        }
        for _ in 0..12 {
            now = now.saturating_add(16);
            b.shell.tick(now);
            if !b.shell.is_dirty() {
                continue;
            }
            let full = b.shell.needs_full_redraw();
            let damage = b.shell.take_damage();
            let t = Instant::now();
            let plan = comp.repaint(&mut b.shell, full, &damage);
            let ms = elapsed_ms(t);
            if full {
                full_frames.push(ms);
            } else {
                panel_frames.push(ms);
                let area: f64 = plan
                    .rects(W, H)
                    .iter()
                    .map(|r| f64::from(r.w) * f64::from(r.h))
                    .sum();
                panel_px = panel_px.max(area);
            }
        }
        // Close it again, so the next open starts from the desktop.
        now = now.saturating_add(16);
        if !click(&mut b, &mut now, Hot::MenuEnvironment) {
            return ExitCode::FAILURE;
        }
        b.shell.tick(now);
        let full = b.shell.needs_full_redraw();
        let damage = b.shell.take_damage();
        let _ = comp.repaint(&mut b.shell, full, &damage);
    }
    println!(
        "panel_reveal.panel_area_px={panel_px:.0} screen_px={} full_frames={} panel_frames={}",
        W.saturating_mul(H),
        full_frames.len(),
        panel_frames.len()
    );
    report("panel_reveal_full", full_frames);
    report("panel_reveal_damaged", panel_frames);

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
    locked.shell.facts.session_user = Some("root".to_owned());
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

    ExitCode::SUCCESS
}
