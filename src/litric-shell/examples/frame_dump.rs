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
//! The scenes are the desktop, the windows overlapping, one per application, the
//! dock grown into the launcher, and the lock surface in three states — as it
//! opens, with the password field out, and with its customize sheet open. The
//! lock is the one surface that replaces the whole composition, so a dumped
//! frame is the only way to see it before the image boots.
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
use litric_input::event::{InputEvent, Point};
use litric_locale::Regional;
use litric_shell::canvas::{rgb, Canvas, Rect};
use litric_shell::desktop;
use litric_shell::lock::Stage;
use litric_shell::shell::{DesktopMode, EnvRow, ProcRow, Shell};
use litric_shell::shortcuts::{Action, App};
use litric_shell::ui::Tokens;
use litric_shell::views;

/// The canvas size, `--size WxH`. 1280x800 unless asked otherwise.
static SIZE: std::sync::OnceLock<(u32, u32)> = std::sync::OnceLock::new();
/// The theme, `--theme light`. Dark unless asked otherwise.
static THEME: std::sync::OnceLock<ThemeKind> = std::sync::OnceLock::new();

fn width() -> u32 {
    SIZE.get().copied().unwrap_or((1280, 800)).0
}

fn height() -> u32 {
    SIZE.get().copied().unwrap_or((1280, 800)).1
}

/// The logical-to-physical scale the shell itself picks for a display of this
/// width (`main.rs::run_png`): the design is drawn for 1280, so a wider screen
/// is the same layout, larger.
fn scale() -> f32 {
    (width() as f32 / 1280.0).clamp(0.6, 3.0)
}

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
    let theme = Theme::of(THEME.get().copied().unwrap_or(ThemeKind::Dark));
    let mut shell = Shell::new(
        Tokens::new(theme, scale()),
        width() as f32,
        height() as f32,
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
    // The same three auth facts the daemon reports on a machine whose image has
    // `/usr/bin/unix_chkpwd`: a password can be checked, the lock is not
    // refused, and root is the account logged in. Without them the lock scenes
    // would photograph a refusal instead of a lock screen.
    shell.facts.auth_available = Some("unix-chkpwd".to_owned());
    shell.facts.auth_lock = Some(false);
    shell.facts.auth_account = Some("litric".to_owned());
    shell.facts.auth_password = Some("set".to_owned());
    shell.facts.session_user = Some("litric".to_owned());
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
    // The Activity window draws graphs from history, and history is what a
    // running machine has: one sample per poll. So the fixture is a *series* of
    // samples, one a second apart, ending at the two that were already here --
    // the earlier ones exist so a frame can show a trend instead of a
    // two-point line, and the last two are unchanged so every other frame in
    // this dump is the frame it was before the graphs existed.
    for step in 0..HISTORY_STEPS {
        let mut sample = shell.facts.clone();
        // A machine that is doing something: a slow wave in the machine-wide
        // busy share, each core offset from its neighbour, memory filling and
        // emptying a little, and an interface carrying a burst. Every operation
        // is saturating because the workspace denies arithmetic that can wrap:
        // these are counters, and a counter that ran past its type should hold
        // at the ceiling rather than produce a number nobody can explain.
        sample.cpu_ticks = Some((
            1_000_u64.saturating_add(step.saturating_mul(40)),
            4_000_u64.saturating_add(step.saturating_mul(800)),
        ));
        sample.core_ticks = CORE_BUSY
            .iter()
            .enumerate()
            .map(|(i, busy)| {
                let wave = step
                    .saturating_mul(3)
                    .saturating_add((i as u64).saturating_mul(5))
                    % 14;
                (
                    100_u64
                        .saturating_add(step.saturating_mul(40))
                        .saturating_add(*busy)
                        .saturating_add(wave),
                    500_u64.saturating_add(step.saturating_mul(800)),
                )
            })
            .collect();
        sample.mem_used_mb = Some(430_u64.saturating_add((step % 12).saturating_mul(8)));
        sample.mem_cached_mb = Some(1_100_u64.saturating_add((step % 25).saturating_mul(6)));
        sample.net_rx_bytes = Some(
            812_000_u64
                .saturating_add(step.saturating_mul(40_000))
                .saturating_add((step % 9).saturating_mul(21_000)),
        );
        sample.net_tx_bytes = Some(
            120_000_u64
                .saturating_add(step.saturating_mul(9_000))
                .saturating_add((step % 5).saturating_mul(3_500)),
        );
        sample.processes = processes(step % 2);
        shell.tick_after(1_000);
        shell.sample(sample);
    }
    let mut first = shell.facts.clone();
    first.cpu_ticks = Some((1_000, 4_000));
    first.core_ticks = CORE_BUSY.iter().map(|_| (100, 500)).collect();
    first.processes = processes(0);
    first.sampled_ms = None;
    shell.tick_after(1_000);
    shell.sample(first);
    let mut second = shell.facts.clone();
    second.cpu_ticks = Some((1_144, 4_800));
    second.core_ticks = CORE_BUSY
        .iter()
        .map(|b| (100_u64.saturating_add(*b), 600))
        .collect();
    second.processes = processes(1);
    second.sampled_ms = None;
    shell.tick_after(1_000);
    shell.sample(second);
    shell.environments = vec![
        env_row("windows", "Windows 11", "not-installed", "Win+1"),
        env_row("litric", "LitricOS", "running", "Win+2"),
        env_row("linux", "Linux", "not-installed", "Win+3"),
    ];
    let mut wallpaper = Canvas::new(width(), height(), rgb(0x000000));
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

/// How many history samples the fixture feeds before the two that set the
/// displayed readings. Forty seconds of a machine that is being used: enough
/// for every graph in the Activity window to have a shape.
const HISTORY_STEPS: u64 = 40;

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
            // Saturating: `clippy::arithmetic_side_effects` is denied workspace
            // wide in every target, and a tick counter that ran past its type is
            // a fixture that should stop at the ceiling rather than wrap.
            ticks: ticks
                .0
                .saturating_add(ticks.1.saturating_sub(ticks.0).saturating_mul(step)),
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

/// The Files window with a listing that has files in it: the grid state of the
/// required golden set (governance §2, screen 3).
///
/// The frames are composed here rather than in a live session, so the listing
/// is the fixture's own -- a folder with a document, a folder, a picture and an
/// archive -- rather than whatever this host happens to have under `/`.
fn scene_files(rig: &mut Rig) {
    let _ = rig.shell.open_app(App::Files);
    rig.shell.files.rows = file_rows();
    rig.shell.files.listed = Some("/Documents".to_owned());
}

/// The same window with nothing in the folder, and nothing selected: the empty
/// and edge state (governance §2, screen 4).
fn scene_files_empty(rig: &mut Rig) {
    scene_files(rig);
    rig.shell.files.rows.clear();
    rig.shell.files.selected_rows.clear();
    rig.shell.files.selected_name = None;
    rig.shell.files.path = "/Documents/Empty".to_owned();
    rig.shell.files.listed = Some("/Documents/Empty".to_owned());
    rig.shell.files.filter.clear();
}

/// Settings, on the Appearance page: theme cards and the accent picker
/// (governance §2, screen 5).
fn scene_settings_appearance(rig: &mut Rig) {
    let _ = rig.shell.open_app(App::Settings);
    rig.shell.settings.page = litric_shell::settings::Page::Appearance;
}

/// Settings, on the Updates page: the updater's generation record and the
/// daemon-owned backup schedule as the shell reads both over its socket.
///
/// The values are the protocol's own shape, not a second model: the page parses
/// the same lines a running daemon writes, so this frame shows what a person
/// sees when both services have a valid record.
fn scene_settings_updates(rig: &mut Rig) {
    let _ = rig.shell.open_app(App::Settings);
    rig.shell.settings.page = litric_shell::settings::Page::Updates;
    rig.shell.update_status = litric_shell::updatestatus::Status {
        schema: Some(3),
        state: "ready".to_owned(),
        active_slot: Some("A".to_owned()),
        active_version: Some("1.0.0".to_owned()),
        rollback_slot: Some("B".to_owned()),
        rollback_version: Some("0.9.0".to_owned()),
        health: Some("healthy".to_owned()),
        health_matches: Some(true),
        ..Default::default()
    };
    rig.shell.backup_status = litric_shell::backupstatus::Status {
        state: "ready".to_owned(),
        interval_secs: Some(86_400),
        snapshots: Some(3),
        last_id: Some("snap-00000000001800000000".to_owned()),
        last_created: Some(1_800_000_000),
        last_entries: Some(7),
        last_bytes: Some(48_213),
        ..Default::default()
    };
}

/// The danger dialog: deleting a file, with the object named and Cancel as the
/// safe default (governance §2, screen 9).
fn scene_danger_dialog(rig: &mut Rig) {
    scene_files(rig);
    rig.shell.files.selected_name = Some("report.pdf".to_owned());
    rig.shell.files.selected_rows.clear();
    rig.shell
        .files
        .selected_rows
        .insert("/Documents/report.pdf".to_owned());
    rig.shell.files.ask_confirm(
        "Delete permanently",
        "Delete 'report.pdf' permanently? This cannot be undone.",
        litric_shell::apps::FilesOp::Delete("/Documents/report.pdf".to_owned()),
    );
}

/// The Control Center, open (governance §2, screen 6).
///
/// The facts are the QEMU machine's: one wired link, a radio that BlueZ says is
/// on, and a sink at 40%. A panel recorded with no interface and no radio would
/// be a picture of an empty state; the screen this frame stands for is the one a
/// person opens, with every row the build can draw doing so.
fn scene_control_centre(rig: &mut Rig) {
    use litric_shell::shell::{NetIface, SystemFacts};
    rig.shell.sample(SystemFacts {
        net_ifaces: vec![NetIface {
            name: "enp0s2".to_owned(),
            state: "up".to_owned(),
            mac: "52:54:00:12:34:56".to_owned(),
            wireless: false,
        }],
        bt_state: Some("ready".to_owned()),
        bt_daemon: Some(true),
        bt_adapters: vec!["hci0 00:1a:7d:da:71:13".to_owned()],
        bt_powered: Some(true),
        audio_volume: Some(40),
        audio_muted: Some(false),
        ..SystemFacts::default()
    });
    let _ = rig.shell.perform(Action::ToggleQuickSettings);
    for _ in 0..40 {
        rig.shell.tick_after(16);
    }
}

/// The notification centre, with a few notifications in it -- including one
/// stop of the Defender, so the list has a severity to draw (screen 8).
fn scene_notifications(rig: &mut Rig) {
    use litric_shell::shell::Priority;
    rig.shell.notify(
        "Defender",
        "Scan complete",
        "1 204 files, no threats",
        Priority::Normal,
        None,
    );
    rig.shell.notify(
        "Files",
        "Moved 12 items",
        "to /Documents/Archive",
        Priority::Normal,
        None,
    );
    rig.shell.notify(
        "Defender",
        "Threat blocked",
        "a download was stopped in Chrome",
        Priority::Critical,
        None,
    );
    let _ = rig.shell.perform(Action::ToggleNotifications);
    for _ in 0..40 {
        rig.shell.tick_after(16);
    }
}

/// The power menu under the power glyph: lock, sleep, shut down, restart.
fn scene_power_menu(rig: &mut Rig) {
    let _ = rig.shell.perform(Action::TogglePowerMenu);
    for _ in 0..40 {
        rig.shell.tick_after(16);
    }
}

/// The LitricOS menu under the wordmark: the session's own settings, and the
/// lock. This is where the quick-settings panel's SESSION section moved to.
fn scene_litro_menu(rig: &mut Rig) {
    let _ = rig.shell.perform(Action::ToggleLitroMenu);
    for _ in 0..40 {
        rig.shell.tick_after(16);
    }
}

/// The boot intro: the mark, the line that says what is happening, and the pen's
/// own progress as a track. Drawn from the first frame of the session.
fn scene_boot_intro(rig: &mut Rig) {
    rig.shell.restart_boot_intro();
    // Half the pen path: the mark is legible and still visibly in progress.
    for _ in 0..60 {
        rig.shell.tick_after(16);
    }
}

/// The listing the Files scenes draw: one of each kind the window renders.
fn file_rows() -> Vec<litric_shell::apps::FileRow> {
    use litric_shell::apps::{FileKind, FileRow};
    vec![
        FileRow {
            name: "archive-2026-10-16.tar.gz".to_owned(),
            kind: FileKind::File,
            size: 12_582_912,
            mtime: 1_761_000_000,
        },
        FileRow {
            name: "notes.txt".to_owned(),
            kind: FileKind::File,
            size: 3_204,
            mtime: 1_760_000_000,
        },
        FileRow {
            name: "report.pdf".to_owned(),
            kind: FileKind::File,
            size: 842_137,
            mtime: 1_759_500_000,
        },
        FileRow {
            name: "Screencast-2026-10-16-14-12-35.webm".to_owned(),
            kind: FileKind::File,
            size: 233_832_448,
            mtime: 1_759_000_000,
        },
        FileRow {
            name: "Screenshots".to_owned(),
            kind: FileKind::Dir,
            size: 0,
            mtime: 1_758_000_000,
        },
    ]
}

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

/// The desktop with the dock grown into the launcher, at rest.
///
/// Two springs have to settle before the frame is the design rather than a frame
/// of an animation: the dock's reveal, which follows the pointer's distance from
/// the bottom edge, and the morph from pill to panel. So the pointer is put where
/// a user puts it to call the dock up, and then the clock is advanced far enough
/// for both to arrive — ninety sixteen-millisecond steps is nearly a second and a
/// half, longer than either takes.
fn scene_launcher(rig: &mut Rig) {
    let _ = rig.shell.handle(
        &InputEvent::PointerMove {
            at: Point::new(width() as f32 / 2.0, height() as f32 - 5.0),
            delta: Point::new(1.0, 1.0),
        },
        1_000,
    );
    let _ = rig.shell.perform(Action::ToggleLauncher);
    // `tick_after` names the elapsed time. A constant timestamp handed to
    // `tick` is a delta of zero after the first call, so no spring ever moved
    // and this frame was the bare desktop with no dock in it.
    for _ in 0..90 {
        rig.shell.tick_after(16);
    }
}

/// The launcher with a query typed: the panel as results and the field.
fn scene_search(rig: &mut Rig) {
    scene_launcher(rig);
    let _ = rig.shell.handle(
        &InputEvent::TextInput {
            text: "fi".to_owned(),
        },
        3_000,
    );
    for _ in 0..30 {
        rig.shell.tick_after(16);
    }
}

/// Mission Control: every window as a card, including one that is minimized
/// and one that lives on another workspace.
fn scene_mission_control(rig: &mut Rig) {
    let _ = rig.shell.open_app(App::Files);
    let _ = rig.shell.open_app(App::Terminal);
    let _ = rig.shell.open_app(App::Settings);
    if let Some(id) = rig
        .shell
        .wm
        .all()
        .iter()
        .find(|w| w.app_id == App::Settings.app_id())
        .map(|w| w.id)
    {
        rig.shell.wm.move_to_workspace(id, 2);
    }
    if let Some(id) = rig
        .shell
        .wm
        .all()
        .iter()
        .find(|w| w.app_id == App::Terminal.app_id())
        .map(|w| w.id)
    {
        rig.shell.wm.minimize(id);
    }
    let _ = rig.shell.perform(Action::ToggleOverview);
    for _ in 0..30 {
        rig.shell.tick_after(16);
    }
}

/// The lock surface as it opens: drifting fields, the glass clock, the widgets.
fn scene_locked(rig: &mut Rig) {
    if !rig.shell.lock_now() {
        eprintln!("frame_dump: the fixture cannot lock, so 11-lock would be the desktop");
    }
}

/// The lock surface with the password field out, which is what a key press
/// shows: the same frame with a glass sheet over it.
fn scene_locked_entry(rig: &mut Rig) {
    scene_locked(rig);
    rig.shell.lock.stage = Stage::Entry;
}

/// The lock surface with the customize sheet open on its first tab.
fn scene_locked_customize(rig: &mut Rig) {
    scene_locked(rig);
    rig.shell.lock.edit = true;
}

/// Compose one full frame and report what drawing it cost.
///
/// Returns the draw time, the pixels painted, the pixel count split by the kind
/// of loop that asked for them, and the text-layout audit for the frame. The
/// audit runs here rather than in the shell because it is a property of a
/// composition, and a frame is the only thing that has one.
///
/// The split is taken from counters [`views::draw`] already keeps, so asking for
/// it adds no drawing and cannot change the frame. BUG-0018 asks for it: the
/// total alone stays silent about whether a locked frame spent its work on the
/// glass blur behind the panels, on the drifting fields, or on the clock's glow.
fn compose(rig: &mut Rig) -> (f64, u64, String, String) {
    let screen = Rect::new(0.0, 0.0, width() as f32, height() as f32);
    rig.scene.blit_from(&rig.wallpaper, screen);
    rig.scene.reset_pixels_visited();
    rig.scene.record_text(true);
    let t = Instant::now();
    views::draw(&mut rig.scene, &mut rig.shell);
    // The pointer belongs to the compositor now (it keeps the pixels under
    // the arrow so a move need not recompose), so a dumped scene draws it the
    // way `Compositor::repaint` does: last, over the finished frame.
    let p = rig.shell.pointer();
    litric_shell::ui::cursor(&mut rig.scene, p.x, p.y, rig.shell.tokens.scale);
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let sites = rig
        .scene
        .pixels_visited_by_site()
        .into_iter()
        .map(|(s, n)| format!("{}={n}", s.label()))
        .collect::<Vec<_>>()
        .join(" ");
    let text = litric_shell::textaudit::report(rig.scene.text_runs(), 3);
    rig.scene.record_text(false);
    (ms, rig.scene.pixels_visited(), text, sites)
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
        (width() as usize)
            .saturating_mul(height() as usize)
            .saturating_mul(3)
            .saturating_add(32),
    );
    out.extend_from_slice(format!("P6\n{} {}\n255\n", width(), height()).as_bytes());
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
    // `--fonts DIR` names a local copy of the faces the image installs, for a
    // host that is not the image. Without it the faces are read from the system
    // font directories, which is what the shipping shell does.
    let mut output = PathBuf::from("build/frames");
    let mut fonts: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fonts" => match args.next() {
                Some(path) => fonts = Some(PathBuf::from(path)),
                None => {
                    eprintln!("frame_dump: --fonts needs a directory");
                    return ExitCode::FAILURE;
                }
            },
            "--size" => {
                let parsed = args.next().and_then(|v| {
                    let (w, h) = v.split_once('x')?;
                    Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))
                });
                match parsed {
                    Some((w, h)) if w >= 640 && h >= 400 => {
                        let _ = SIZE.set((w, h));
                    }
                    _ => {
                        eprintln!("frame_dump: --size needs WxH, at least 640x400");
                        return ExitCode::FAILURE;
                    }
                }
            }
            "--theme" => match args.next().as_deref() {
                Some("light") => {
                    let _ = THEME.set(ThemeKind::Light);
                }
                Some("dark") => {
                    let _ = THEME.set(ThemeKind::Dark);
                }
                _ => {
                    eprintln!("frame_dump: --theme is dark or light");
                    return ExitCode::FAILURE;
                }
            },
            _ => output = PathBuf::from(arg),
        }
    }

    let loaded = match &fonts {
        Some(path) => litric_shell::font::initialize_from_dir(path),
        None => litric_shell::font::initialize(),
    };
    if !loaded {
        eprintln!(
            "frame_dump: no Latin text face is installed, so no frame can be drawn.\n\
             Install the image's fonts (noto-fonts, ttf-jetbrains-mono-nerd), or pass\n\
             --fonts DIR pointing at a local copy of NotoSans-Regular.ttf and the rest.\n\
             Frame dump usage: frame_dump [--fonts DIR] [--size WxH] [--theme dark|light] [OUTPUT_DIR]"
        );
        return ExitCode::FAILURE;
    }
    let faces = litric_shell::font::face_report();
    if !faces.complete() {
        // Substituted faces change word widths, so the frames are still worth
        // looking at but are not identical to the machine's. Say which ones,
        // rather than letting the reviewer assume.
        eprintln!(
            "frame_dump: substituted face(s): {} — frames are usable for layout, not for typography",
            faces.missing().join(", ")
        );
    }
    let dir = output;
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("frame_dump: cannot create {}: {e}", dir.display());
        return ExitCode::FAILURE;
    }

    type Scene = (&'static str, fn(&mut Rig));
    // The lock and overlay scenes are dumped here rather than in the app list
    // because they are not applications: they replace the composition or sit
    // over it, which makes this example the only way to look at them without
    // booting the image.
    //
    // The set is the governance document's ten required golden screens
    // (04-QA-LINT-GOVERNANCE §2) plus the frames this dump has always written:
    // the lock in three states, the launcher and its search, and the remaining
    // applications. `ci/check-golden-frames.py` names the required ten.
    let scenes: [Scene; 17] = [
        ("01-desktop-idle", scene_idle),
        ("02-desktop-start", scene_start),
        ("04-files", scene_files),
        ("16-files-empty", scene_files_empty),
        ("05-settings", scene_settings_appearance),
        ("25-settings-updates", scene_settings_updates),
        ("17-control-centre", scene_control_centre),
        ("18-notifications", scene_notifications),
        ("19-dialog-danger", scene_danger_dialog),
        ("11-lock", scene_locked),
        ("12-lock-password", scene_locked_entry),
        ("13-lock-customize", scene_locked_customize),
        ("14-launcher", scene_launcher),
        ("21-mission-control", scene_mission_control),
        ("22-power-menu", scene_power_menu),
        ("23-litro-menu", scene_litro_menu),
        ("24-boot-intro", scene_boot_intro),
    ];
    // The seek frame is written by the search scene, and the rest of the
    // applications by the app list.
    let app_scenes: [(App, &str); 6] = [
        (App::Browser, "06-browser"),
        (App::Terminal, "07-terminal"),
        (App::Trash, "08-trash"),
        (App::Defender, "09-defender"),
        (App::Activity, "10-activity"),
        (App::Environments, "20-environments"),
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
    {
        let mut rig = build();
        scene_search(&mut rig);
        if let Err(e) = dump(&dir, "15-search", &mut rig) {
            eprintln!("frame_dump: 15-search: {e}");
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
    let mut comp =
        desktop::Compositor::new(width(), height(), &rig.theme, desktop::Wallpaper::System);
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
    let (ms, visited, text, full_sites) = compose(rig);
    let path = dir.join(format!("{name}.ppm"));
    let bytes = write_ppm(&path, &rig.scene)?;
    let pixels = f64::from(width()) * f64::from(height());
    let (partial_ms, partial_visited, partial_area, sites) = partial_frame(rig);
    println!(
        "scene={name} ppm={} bytes={bytes} px={pixels:.0} visited_px={visited} coverage={:.3} draw_ms={ms:.2} fingerprint=0x{:016x}",
        path.display(),
        visited as f64 / pixels,
        fingerprint(&rig.scene)
    );
    println!("sites={name} {full_sites}");
    println!(
        "partial={name} damage_area_px={partial_area} partial_visited_px={partial_visited} partial_ms={partial_ms:.2} overdraw={:.1}x",
        partial_visited as f64 / partial_area as f64
    );
    println!("partial_sites={name} {sites}");
    println!("text={name} overlaps={text}");
    Ok(())
}
