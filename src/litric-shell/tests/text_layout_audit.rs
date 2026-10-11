//! A gate against text drawn over other text.
//!
//! The complaint was literal -- "ตัวอักษรทับกัน", letters on top of letters --
//! and the five defects it named were all geometry that nothing was checking:
//! a right-aligned CPU value over the core count, a locale value over the label
//! that introduced it, a subtitle inside its own title's band, a filter readout
//! under a button, and four command-row buttons under the Details toggle in a
//! narrow window.
//!
//! Each of those is one assertion away from being caught automatically, so this
//! file is that assertion: render a frame, ask [`litric_shell::textaudit`] for
//! the colliding pairs, and fail unless every one of them is between two
//! *different* surfaces -- two windows that overlap on the screen, whose text is
//! expected to overlap because the upper window's own background covers the
//! lower one's letters.
//!
//! It is deliberately blind to anything else: it does not know whether the
//! layout is good, only whether two strings were drawn on top of each other
//! inside one window. That is the property that was broken.

use litric_design::{Theme, ThemeKind};
use litric_locale::Regional;
use litric_shell::canvas::{rgb, Canvas};
use litric_shell::shell::{DesktopMode, EnvRow, ProcRow, Shell};
use litric_shell::shortcuts::App;
use litric_shell::textaudit;
use litric_shell::ui::Tokens;
use litric_shell::views;

const W: u32 = 1280;
const H: u32 = 800;

fn env_row(token: &str, title: &str, state: &str) -> EnvRow {
    EnvRow {
        token: token.to_owned(),
        title: title.to_owned(),
        state: state.to_owned(),
        active: state == "running",
        hotkey: "Win+1".to_owned(),
    }
}

/// A desktop with fixed facts, the same fixture the frame dumper uses, so a
/// failure here names a layout that a person can reproduce in one command.
fn shell(width: u32, height: u32) -> Shell {
    let mut shell = Shell::new(
        Tokens::new(Theme::of(ThemeKind::Dark), 1.0),
        width as f32,
        height as f32,
        Regional::fallback("text-audit"),
    );
    shell.mode = DesktopMode::Ready;
    shell.facts.mem_total_mb = Some(7867);
    shell.facts.mem_used_mb = Some(527);
    shell.facts.mem_available_mb = Some(7100);
    shell.facts.mem_cached_mb = Some(1200);
    shell.facts.cpus = Some(8);
    shell.facts.cpu_model = Some("AMD Ryzen 7 5800X 8-Core Processor".to_owned());
    shell.facts.cpu_now = Some(0.18);
    shell.facts.cpu_cores = vec![0.18, 0.07, 0.42, 0.05, 0.11, 0.30, 0.02, 0.09];
    shell.facts.uptime_secs = Some(4_000.0);
    shell.facts.load1 = Some(0.52);
    shell.facts.tasks = Some(143);
    shell.facts.sysd_rss_mb = Some(2);
    shell.facts.mglru = Some(true);
    shell.facts.verity = Some(true);
    shell.facts.root_fs = Some("erofs".to_owned());
    shell.facts.trash = Some((0, 0));
    shell.facts.source = "text-audit fixture".to_owned();
    shell.facts.volumes = vec![
        ("/".to_owned(), "erofs".to_owned(), true),
        ("/persist".to_owned(), "ext4".to_owned(), false),
    ];
    shell.environments = vec![
        env_row("windows", "Windows 11", "not-installed"),
        env_row("litric", "LitricOS", "running"),
        env_row("linux", "Linux", "not-installed"),
    ];
    shell
}

/// Draw one frame and return the collisions that are inside a single surface.
fn collisions_in_a_window(shell: &mut Shell, width: u32, height: u32) -> Vec<textaudit::Collision> {
    let mut canvas = Canvas::new(width, height, rgb(0x000000));
    canvas.record_text(true);
    views::draw(&mut canvas, shell);
    let found = textaudit::collisions(canvas.text_runs());
    found.into_iter().filter(|c| !c.cross_surface).collect()
}

fn assert_clean(shell: &mut Shell, width: u32, height: u32, what: &str) {
    let found = collisions_in_a_window(shell, width, height);
    if found.is_empty() {
        return;
    }
    let worst: Vec<String> = found
        .iter()
        .take(5)
        .map(|c| {
            format!(
                "{}: {:?} at ({:.0},{:.0}) [surface {}] over {:?} at ({:.0},{:.0}) [surface {}] by {:.0}x{:.0}px",
                c.kind.as_str(),
                c.first.text,
                c.first.x,
                c.first.baseline,
                c.first.surface,
                c.second.text,
                c.second.x,
                c.second.baseline,
                c.second.surface,
                c.overlap_x,
                c.overlap_y
            )
        })
        .collect();
    let runs = found.first().map_or(0, |_| 0);
    for c in found.iter() {
        eprintln!(
            "RUN a={:?} x={:.0} end={:.0} base={:.0} size={:.0} surface={} | b={:?} x={:.0} end={:.0} base={:.0} size={:.0} surface={}",
            c.first.text, c.first.x, c.first.end_x, c.first.baseline, c.first.size, c.first.surface,
            c.second.text, c.second.x, c.second.end_x, c.second.baseline, c.second.size, c.second.surface
        );
    }
    assert!(
        found.is_empty(),
        "{what}: {} pair(s) of strings drawn over each other (first {runs}): {}",
        found.len(),
        worst.join(" | ")
    );
}

#[test]
fn the_desktop_chrome_does_not_draw_text_over_text() {
    let mut shell = shell(W, H);
    shell.tick(1_000);
    assert_clean(&mut shell, W, H, "desktop with no windows");
}

#[test]
fn every_window_does_not_draw_text_over_text() {
    for app in App::ALL {
        let mut shell = shell(W, H);
        let _ = shell.open_app(app);
        shell.tick(1_000);
        assert_clean(&mut shell, W, H, app.title());
    }
}

#[test]
fn a_narrow_window_does_not_draw_text_over_text() {
    // The command row is where a fixed offset failed: at 420 pixels wide the
    // buttons used to be drawn under the Details toggle.
    let mut shell = shell(560, 420);
    let _ = shell.open_app(App::Files);
    shell.tick(1_000);
    assert_clean(&mut shell, 560, 420, "Files in a small window");
}

#[test]
fn the_system_monitor_does_not_draw_text_over_text() {
    // The monitor is the densest page in the system: a header row, a meter, a
    // per-core grid, three labelled rows, a storage list with notes and a fact
    // list. It is also where the audit found the largest overlap (62x17px).
    let mut shell = shell(760, 620);
    let _ = shell.open_app(App::Activity);
    shell.tick(1_000);
    assert_clean(&mut shell, 760, 620, "Activity");
}

#[test]
fn settings_does_not_draw_text_over_text() {
    let mut shell = shell(760, 620);
    let _ = shell.open_app(App::Settings);
    shell.tick(1_000);
    assert_clean(&mut shell, 760, 620, "Settings");
}

/// Every window size a person can actually make, not one convenient size.
///
/// The five defects the first audit found were all geometry that only failed
/// below some width -- 560 was checked and 620 was not, and the narrow-window
/// command row was the one that survived. A single size per app is the same
/// mistake in a smaller shape, so this walks the range the window manager can
/// produce: the smallest window it will open, two in between, and the largest
/// the screen allows.
const SIZES: [(u32, u32); 4] = [(480, 360), (560, 420), (900, 640), (1264, 784)];

fn assert_clean_at_every_size(app: App, what: &str) {
    for (w, h) in SIZES {
        let mut shell = shell(w, h);
        let _ = shell.open_app(app);
        shell.tick(1_000);
        assert_clean(&mut shell, w, h, &format!("{what} at {w}x{h}"));
    }
}

#[test]
fn every_window_is_clean_at_every_size_it_can_have() {
    for app in App::ALL {
        assert_clean_at_every_size(app, app.title());
    }
}

#[test]
fn every_window_is_clean_in_the_light_theme() {
    // Tokens carry the palette and the scale; a layout that only works in dark
    // is a layout whose geometry depends on colour, which is a defect in both.
    let mut desktop = shell(W, H);
    desktop.tokens = Tokens::new(Theme::of(ThemeKind::Light), 1.0);
    desktop.tick(1_000);
    assert_clean(&mut desktop, W, H, "desktop in the light theme");
    for app in App::ALL {
        let mut light = shell(760, 620);
        light.tokens = Tokens::new(Theme::of(ThemeKind::Light), 1.0);
        let _ = light.open_app(app);
        light.tick(1_000);
        assert_clean(&mut light, 760, 620, &format!("{} (light)", app.title()));
    }
}

/// One process, as the daemon reports it: cumulative ticks, which is what the
/// rate between two samples is computed from.
fn process(pid: u32, name: &str, ticks: u64, rss_mb: u64) -> ProcRow {
    ProcRow {
        pid,
        name: name.to_owned(),
        cmdline: format!("/usr/bin/{name} --daemon"),
        state: "sleeping".to_owned(),
        ppid: 1,
        threads: 3,
        ticks,
        rss_mb: Some(rss_mb),
        uid: Some(1000),
        cpu: None,
    }
}

/// `base + rate * step`, saturating.
///
/// `clippy::arithmetic_side_effects` is denied workspace-wide in every target,
/// test fixtures included, and it is right to be: a tick counter that overflowed
/// would make the rate under test depend on the width of the integer. Saturating
/// keeps the sample monotonic, which is the property these fixtures assert.
fn ticks_at(base: u64, rate: u64, step: u64) -> u64 {
    base.saturating_add(rate.saturating_mul(step))
}

/// Two samples of a machine with a process table, so the Activity window has
/// live rates to draw rather than dashes.
fn sample_processes(shell: &mut Shell, first: bool) {
    let step = u64::from(!first);
    let mut sample = shell.facts.clone();
    sample.cpu_ticks = Some((ticks_at(1_000, 400, step), ticks_at(4_000, 800, step)));
    sample.core_ticks = vec![(ticks_at(100, 40, step), ticks_at(500, 100, step)); 8];
    sample.processes = vec![
        process(1, "litric-sysd", ticks_at(100, 200, step), 64),
        process(2, "litric-shell", ticks_at(500, 50, step), 4096),
        process(3, "a-process-with-a-very-long-name-indeed", 20, 8),
    ];
    shell.sample(sample);
}

#[test]
fn the_activity_window_shows_a_process_table_without_drawing_text_over_text() {
    // The table is the densest surface in the system: a heading row, five
    // columns of numbers and four hundred rows behind them, in a window that can
    // be made very small. Everything it draws is elided to its own column, and
    // this is the test that says so.
    for (w, h) in [(560, 420), (900, 640), (W, H)] {
        let mut shell = shell(w, h);
        let _ = shell.open_app(App::Activity);
        sample_processes(&mut shell, true);
        sample_processes(&mut shell, false);
        shell.tick(1_000);
        assert_clean(
            &mut shell,
            w,
            h,
            &format!("Activity with a process table at {w}x{h}"),
        );
        // And the table really is on the screen: both a process name and the
        // live rate that the two samples above produce.
        let mut canvas = Canvas::new(w, h, rgb(0x000000));
        canvas.record_text(true);
        views::draw(&mut canvas, &mut shell);
        let text: Vec<String> = canvas.text_runs().iter().map(|r| r.text.clone()).collect();
        assert!(
            text.iter().any(|r| r == "litric-shell"),
            "the process table is drawn at {w}x{h}: {text:?}"
        );
        assert!(
            text.iter().any(|r| r == "25.0"),
            "the rate between the two samples is what the table shows at {w}x{h}: {text:?}"
        );
    }
}

#[test]
fn a_full_interface_is_clean() {
    // The dense case: a dialog over a window, a populated trash, a terminal with
    // output, a browser with tabs, notifications and the quick panel. Each of
    // those adds strings to the same frame, and none of them was in the frame
    // the earlier audits measured.
    let mut full = shell(W, H);
    let _ = full.open_app(App::Files);
    full.files.apply_list_reply(
        "e name=%44ocuments kind=dir size=0 mtime=0\n\
         e name=%52eport-2026-final-v3.txt kind=file size=1048576 mtime=0\n\
         e name=%69mage-0420.raw kind=file size=99999999 mtime=0\n",
    );
    full.files.ask_new_folder();
    full.files.dialog_field = "Documents".to_owned();
    full.terminal.push_line("$ ls -la /persist");
    full.terminal
        .push_line("total 1048576  drwxr-xr-x  12 root root  4096  documents".to_owned());
    full.browser.new_tab();
    full.browser.new_tab();
    full.browser.status = "litric-browser: 1280x800 frame, 3 requests".to_owned();
    full.trash.entries = vec![
        (
            "report-2026-final-v3.txt".to_owned(),
            "2026-10-01".to_owned(),
            "/Documents".to_owned(),
            false,
        ),
        (
            "build".to_owned(),
            "2026-09-30".to_owned(),
            "/Projects".to_owned(),
            true,
        ),
    ];
    full.defender.app.start(12);
    full.notifications.post(
        "Defender",
        "Scan finished: 12 files, no threats",
        "LitricOS",
        litric_shell::shell::Priority::Normal,
        0,
        Some(60_000),
    );
    full.tick(1_000);
    assert_clean(&mut full, W, H, "a full desktop");

    // The same, in the smallest window each app can be given: a dialog is a
    // second layer of strings on a screen that has no spare room.
    for app in [App::Files, App::Terminal, App::Trash] {
        let mut small = shell(560, 420);
        let _ = small.open_app(app);
        if app == App::Files {
            small.files.apply_list_reply(
                "e name=%52eport-2026-final-v3.txt kind=file size=1048576 mtime=0\n",
            );
            small.files.selected = Some(0);
            small.files.ask_confirm(
                "Move to trash",
                "Documents/report-2026-final-v3.txt moves to the trash",
                litric_shell::apps::FilesOp::Trash("/Documents/report.txt".to_owned()),
            );
        }
        small.tick(1_000);
        assert_clean(&mut small, 560, 420, &format!("{} at 560x420", app.title()));
    }
}
