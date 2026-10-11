//! Dump the desktop's real frames to PPM files, without a display and without
//! a virtual machine.
//!
//! The governance plan (04-QA-LINT-GOVERNANCE §2) requires ten golden screens
//! and says they must come from the machine, not from a mock renderer. That is
//! the right rule for release evidence, but it has a cost the project has been
//! paying: until someone boots QEMU, nobody can see what a change did to the
//! interface, and a review of a UI diff is a review of an argument.
//!
//! This example is the half of that pipeline that runs anywhere. It drives the
//! *shipping* composition -- [`views::draw`] and [`desktop::wallpaper`], the
//! same functions `main.rs` calls on a booted machine -- into an in-memory
//! [`Canvas`], and writes each frame as a P6 PPM that `tools/ppm2png.py` turns
//! into a PNG. Nothing here reimplements drawing, so a frame dumped here differs
//! from a frame on the machine only by the facts that were fed in.
//!
//! What it is not: a replacement for the VM photographs. A frame produced here
//! proves what the composition does with fixed inputs; it cannot prove that the
//! machine boots, that the framebuffer is the right depth, or that the typeface
//! rasterises on the target. Those still need the image. It removes one of the
//! two excuses for not looking, not the obligation to boot.
//!
//! The facts are fixed numbers set below rather than readings, so two runs of
//! this example produce byte-identical frames and the `fingerprint` line is a
//! usable identity for a golden: same code plus same facts gives the same
//! fingerprint, and any change to the composition changes it.
//!
//! ```text
//! cargo run --release -p litric-shell --example frame_dump -- [DIR]
//! ```
//!
//! `DIR` is where the PPM files are written (default: `build/frames`).

#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use litric_design::{Theme, ThemeKind};
use litric_locale::Regional;
use litric_shell::canvas::{rgb, Canvas, Rect};
use litric_shell::desktop;
use litric_shell::shell::{DesktopMode, EnvRow, ProcRow, Shell};
use litric_shell::shortcuts::App;
use litric_shell::ui::Tokens;
use litric_shell::views;

const W: u32 = 1280;
const H: u32 = 800;

/// One desktop, its wallpaper layer, and the frame being composed.
struct Rig {
    shell: Shell,
    theme: Theme,
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

/// A desktop with fixed, plausible facts -- the same fixture `render_bench`
/// uses, so the two examples describe the same machine.
fn build() -> Rig {
    let theme = Theme::of(ThemeKind::Dark);
    let mut shell = Shell::new(
        Tokens::new(theme, 1.0),
        W as f32,
        H as f32,
        Regional::fallback("frame-dump"),
    );
    shell.mode = DesktopMode::Ready;
    shell.facts.mem_total_mb = Some(7867);
    shell.facts.mem_used_mb = Some(527);
    shell.facts.cpus = Some(8);
    shell.facts.uptime_secs = Some(4_000.0);
    shell.facts.cpu_fraction = Some(0.07);
    shell.facts.root_fs = Some("erofs".to_owned());
    shell.facts.verity = Some(true);
    shell.facts.source = "frame-dump fixture".to_owned();
    // The system monitor's own data. Fixed numbers, so the frame is
    // reproducible; the point of the fixture is that the monitor shows every
    // fact the machine reports, not two of them.
    shell.facts.cpu_model = Some("AMD Ryzen 7 5800X 8-Core Processor".to_owned());
    shell.facts.cpu_now = Some(0.18);
    shell.facts.cpu_cores = vec![0.18, 0.07, 0.42, 0.05, 0.11, 0.30, 0.02, 0.09];
    shell.facts.core_ticks = vec![(0, 0); 8];
    shell.facts.load1 = Some(0.52);
    shell.facts.tasks = Some(143);
    shell.facts.sysd_rss_mb = Some(2);
    shell.facts.mglru = Some(true);
    shell.facts.mem_available_mb = Some(7100);
    shell.facts.mem_cached_mb = Some(1200);
    shell.facts.swap_used_mb = Some(0);
    shell.facts.volumes = vec![
        ("/".to_owned(), "erofs".to_owned(), true),
        ("/persist".to_owned(), "ext4".to_owned(), false),
        ("/run".to_owned(), "tmpfs".to_owned(), false),
    ];
    // Two samples of a process table. A rate is the difference between two
    // samples, so a frame composed from one sample would show every process as
    // unmeasured -- and the Activity window's whole point is the table. The tick
    // deltas are chosen to reproduce the fixture's own 18% and its per-core
    // pattern exactly, so adding the table changes no other frame.
    let mut first = shell.facts.clone();
    first.cpu_ticks = Some((1_000, 4_000));
    first.core_ticks = CORE_BUSY.iter().map(|_| (100, 500)).collect();
    first.processes = processes(0);
    shell.sample(first);
    let mut second = shell.facts.clone();
    second.cpu_ticks = Some((1_144, 4_800));
    second.core_ticks = CORE_BUSY.iter().map(|b| (100 + b, 600)).collect();
    second.processes = processes(1);
    shell.sample(second);
    shell.environments = vec![
        env_row("windows", "Windows 11", "not-installed", "Win+1"),
        env_row("litric", "LitricOS", "running", "Win+2"),
        env_row("linux", "Linux", "not-installed", "Win+3"),
    ];
    let mut wallpaper = Canvas::new(W, H, rgb(0x000000));
    desktop::wallpaper(&mut wallpaper, &theme);
    let scene = wallpaper.clone();
    Rig {
        shell,
        theme,
        wallpaper,
        scene,
    }
}

/// How busy each of the eight logical CPUs is in the fixture, in percent of the
/// sample interval. It is the same pattern the per-core grid has always drawn.
const CORE_BUSY: [u64; 8] = [18, 7, 42, 5, 11, 30, 2, 9];

/// The process table of the fixture, at sample `step`.
///
/// The cumulative ticks are chosen against the machine's 800-tick sample so the
/// shares read as whole numbers a person can check by eye: 200 of 800 is 25%,
/// and a process whose counter did not move is 0%, which is not the same as one
/// that was not measured.
fn processes(step: u64) -> Vec<ProcRow> {
    let row = |pid: u32,
               name: &str,
               uid: u32,
               ticks: (u64, u64),
               rss: u64,
               threads: u32,
               cmdline: &str| {
        ProcRow {
            pid,
            name: name.to_owned(),
            cmdline: if cmdline.is_empty() {
                String::new()
            } else {
                format!("/usr/bin/{name} --daemon")
            },
            state: "sleeping".to_owned(),
            ppid: 1,
            threads,
            ticks: ticks.0 + (ticks.1 - ticks.0) * step,
            rss_mb: Some(rss),
            uid: Some(uid),
            cpu: None,
        }
    };
    vec![
        row(2, "litric-shell", 1000, (5_000, 5_200), 412, 9, "yes"),
        row(3, "litric-browser", 1000, (1_200, 1_290), 286, 14, "yes"),
        row(4, "litric-sysd", 0, (800, 830), 12, 3, "yes"),
        row(5, "litric-defender", 0, (90, 96), 48, 4, "yes"),
        row(6, "litric-files", 1000, (300, 306), 64, 2, "yes"),
        row(7, "litric-envd", 0, (60, 61), 8, 2, "yes"),
        row(8, "systemd", 0, (200, 202), 9, 1, "yes"),
        row(9, "dbus-daemon", 0, (30, 30), 4, 1, "yes"),
        row(10, "avahi-daemon", 1000, (12, 12), 3, 1, "yes"),
        row(11, "pipewire", 1000, (25, 27), 22, 3, "yes"),
        row(12, "gpu-worker", 1000, (15, 16), 18, 1, "yes"),
        // A kernel thread: no command line, so the default filter hides it. The
        // frame then shows a filter doing something, not a table of everything.
        row(13, "kworker/0:1", 0, (40, 41), 0, 1, ""),
    ]
}

/// Open a window at a fixed rectangle and give it focus.
fn place(rig: &mut Rig, app: App, x: f32, y: f32, w: f32, h: f32) {
    let id = rig
        .shell
        .wm
        .open_at(app.app_id(), app.title(), litric_wm::Rect::new(x, y, w, h));
    rig.shell.wm.focus_window(id);
}

fn scene_idle(_: &mut Rig) {}

/// The desktop `main.rs` shows at start: Activity and Environments open,
/// Activity focused.
fn scene_start(rig: &mut Rig) {
    let work = rig.shell.wm.work_area();
    let pad = 64.0;
    let panel_w = (work.w * 0.34).clamp(320.0, 520.0);
    place(
        rig,
        App::Environments,
        work.right() - pad - panel_w,
        work.y + pad,
        panel_w,
        (work.h * 0.34).max(220.0),
    );
    place(
        rig,
        App::Activity,
        work.x + pad,
        work.y + pad,
        panel_w,
        (work.h * 0.46).max(260.0),
    );
}

/// Three windows overlapping at three focus levels: the case where glass,
/// shadow and the focus ring all have to read at once.
fn scene_overlap(rig: &mut Rig) {
    let work = rig.shell.wm.work_area();
    let w = (work.w * 0.42).clamp(420.0, 640.0);
    let h = (work.h * 0.5).clamp(300.0, 480.0);
    place(rig, App::Activity, work.x + 48.0, work.y + 48.0, w, h);
    place(rig, App::Files, work.x + 180.0, work.y + 140.0, w, h);
    place(
        rig,
        App::Settings,
        work.right() - w - 48.0,
        work.bottom() - h - 120.0,
        w,
        h,
    );
}

fn scene_app(app: App) -> impl Fn(&mut Rig) {
    move |rig: &mut Rig| {
        let _ = rig.shell.open_app(app);
    }
}

/// Compose one full frame and report what drawing it cost.
///
/// Returns the draw time, the pixels painted, and the text-layout audit for the
/// frame. The audit runs here rather than in the shell because it is a property
/// of a composition, and a frame is the only thing that has one.
fn compose(rig: &mut Rig) -> (f64, u64, String) {
    let screen = Rect::new(0.0, 0.0, W as f32, H as f32);
    rig.scene.blit_from(&rig.wallpaper, screen);
    rig.scene.reset_pixels_visited();
    rig.scene.record_text(true);
    let t = Instant::now();
    views::draw(&mut rig.scene, &mut rig.shell);
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let text = litric_shell::textaudit::report(rig.scene.text_runs(), 3);
    rig.scene.record_text(false);
    (ms, rig.scene.pixels_visited(), text)
}

/// FNV-1a over the frame's pixels. A golden is identified by this number, not
/// by a file name that can keep the old picture after the code moves.
fn fingerprint(canvas: &Canvas) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &px in canvas.pixels() {
        for b in px.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// Write the canvas as a P6 PPM -- the one image format `tools/ppm2png.py`
/// already reads, and the one QEMU's `screendump` already writes.
fn write_ppm(path: &Path, canvas: &Canvas) -> std::io::Result<usize> {
    let mut out = Vec::with_capacity(
        (W as usize)
            .saturating_mul(H as usize)
            .saturating_mul(3)
            .saturating_add(32),
    );
    out.extend_from_slice(format!("P6\n{W} {H}\n255\n").as_bytes());
    for &px in canvas.pixels() {
        out.push(((px >> 16) & 0xFF) as u8);
        out.push(((px >> 8) & 0xFF) as u8);
        out.push((px & 0xFF) as u8);
    }
    let mut file = fs::File::create(path)?;
    file.write_all(&out)?;
    Ok(out.len())
}

fn main() -> ExitCode {
    let dir = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("build/frames"), PathBuf::from);
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("frame_dump: cannot create {}: {e}", dir.display());
        return ExitCode::FAILURE;
    }

    type Scene = (&'static str, fn(&mut Rig));
    let scenes: [Scene; 2] = [
        ("01-desktop-idle", scene_idle),
        ("02-desktop-start", scene_start),
    ];
    let app_scenes: [(App, &str); 7] = [
        (App::Files, "04-files"),
        (App::Settings, "05-settings"),
        (App::Browser, "06-browser"),
        (App::Terminal, "07-terminal"),
        (App::Trash, "08-trash"),
        (App::Defender, "09-defender"),
        (App::Activity, "10-activity"),
    ];

    let mut failures = 0usize;
    for (name, setup) in scenes {
        let mut rig = build();
        setup(&mut rig);
        if let Err(e) = dump(&dir, name, &mut rig) {
            eprintln!("frame_dump: {name}: {e}");
            failures = failures.saturating_add(1);
        }
    }
    {
        let mut rig = build();
        scene_overlap(&mut rig);
        if let Err(e) = dump(&dir, "03-windows-overlap", &mut rig) {
            eprintln!("frame_dump: 03-windows-overlap: {e}");
            failures = failures.saturating_add(1);
        }
    }
    for (app, name) in app_scenes {
        let mut rig = build();
        scene_app(app)(&mut rig);
        if let Err(e) = dump(&dir, name, &mut rig) {
            eprintln!("frame_dump: {name}: {e}");
            failures = failures.saturating_add(1);
        }
    }

    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        eprintln!("frame_dump: {failures} scene(s) failed");
        ExitCode::FAILURE
    }
}

/// Time one damage-clipped frame through the path `main.rs` actually uses.
///
/// `render_bench` reproduces the loop by calling `views::draw` unclipped, so its
/// partial-frame figure is an upper bound, not the loop's cost. The real loop
/// goes through `Compositor::repaint`, which clips each damage region before
/// drawing. BUG-0015 turns on the difference, so BUG-0015 is measured here,
/// through `repaint` itself.
///
/// The pixels are also split by the kind of loop that asked for them, because
/// the total alone does not say whether the work was drawing the damaged pixels
/// or blurring the whole window behind them.
fn partial_frame(rig: &mut Rig) -> (f64, u64, u64, String) {
    let region = Rect::new(600.0, 560.0, 160.0, 48.0);
    let area = (region.w * region.h) as u64;
    let mut comp = desktop::Compositor::new(W, H, &rig.theme);
    let _ = comp.repaint(&mut rig.shell, true, &[]);
    comp.scene.reset_pixels_visited();
    let t = Instant::now();
    let _ = comp.repaint(&mut rig.shell, false, &[region]);
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let sites = comp
        .scene
        .pixels_visited_by_site()
        .into_iter()
        .map(|(s, n)| format!("{}={n}", s.label()))
        .collect::<Vec<_>>()
        .join(" ");
    (ms, comp.scene.pixels_visited(), area, sites)
}

fn dump(dir: &Path, name: &str, rig: &mut Rig) -> std::io::Result<()> {
    let (ms, visited, text) = compose(rig);
    let path = dir.join(format!("{name}.ppm"));
    let bytes = write_ppm(&path, &rig.scene)?;
    let pixels = f64::from(W) * f64::from(H);
    let (partial_ms, partial_visited, partial_area, sites) = partial_frame(rig);
    println!(
        "scene={name} ppm={} bytes={bytes} px={pixels:.0} visited_px={visited} coverage={:.3} draw_ms={ms:.2} fingerprint=0x{:016x}",
        path.display(),
        visited as f64 / pixels,
        fingerprint(&rig.scene)
    );
    println!(
        "partial={name} damage_area_px={partial_area} partial_visited_px={partial_visited} partial_ms={partial_ms:.2} overdraw={:.1}x",
        partial_visited as f64 / partial_area as f64
    );
    println!("partial_sites={name} {sites}");
    println!("text={name} overlaps={text}");
    Ok(())
}
