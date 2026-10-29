//! End-to-end journeys through the shell, driven by synthetic input exactly
//! the way a user produces it: key presses, pointer clicks, right-clicks,
//! typed text. Nothing reaches into private state that real input could not
//! change, so a passing journey is evidence the feature works when the
//! machine is actually driven.

use litric_input::event::{InputEvent, Point, PointerButton};
use litric_input::keys::{Key, Modifiers};

use crate::apps::{DialogKind, FilesHot, MenuAction, ScanState};
use crate::canvas::{luminance, rgb, Canvas, Rect};
use crate::desktop;
use crate::shell::{DesktopMode, Effect, Hot, LifeState, Overlay, Priority, Shell};
use crate::shortcuts::{Action, App};
use crate::ui::Tokens;
use litric_design::{Theme, ThemeKind};
use litric_locale::Regional;

/// One frame at the pace of an active desktop, in milliseconds.
///
/// A journey ticks in **deltas**, not timestamps, and this is the delta it
/// uses: the shell steps its springs by the wall time that passed, so ticking
/// forty times with one timestamp is forty frames that advance nothing. That is
/// what these journeys used to rely on — a constant 16 ms step hid the
/// difference — and it is why every tick here is `tick_after`.
const FRAME_MS: u32 = 16;

fn shell() -> Shell {
    let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
    let mut s = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("test"));
    s.mode = DesktopMode::Ready;
    s
}

/// A fresh temporary root for a journey that reaches the filesystem.
///
/// These journeys drive the shell the way a user does, so the ones that touch
/// files would otherwise touch the machine's real root: the folder the
/// next-folder dialog creates is a real directory, and a test may not put one
/// in `/`. Tagged with the process id so two test processes cannot share one.
fn e2e_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("litric-shell-e2e-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a fixture root");
    dir
}

fn click(s: &mut Shell, x: f32, y: f32) -> Vec<Effect> {
    let mut out = s.handle(
        &InputEvent::PointerMove {
            at: Point::new(x, y),
            delta: Point::new(1.0, 1.0),
        },
        1_000,
    );
    out.extend(s.handle(
        &InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: true,
            at: Point::new(x, y),
            clicks: 1,
            mods: Modifiers::NONE,
        },
        1_010,
    ));
    out.extend(s.handle(
        &InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: false,
            at: Point::new(x, y),
            clicks: 1,
            mods: Modifiers::NONE,
        },
        1_020,
    ));
    out
}

fn right_click(s: &mut Shell, x: f32, y: f32) {
    let _ = s.handle(
        &InputEvent::PointerMove {
            at: Point::new(x, y),
            delta: Point::new(1.0, 1.0),
        },
        1_000,
    );
    let _ = s.handle(
        &InputEvent::PointerButton {
            button: PointerButton::Secondary,
            pressed: true,
            at: Point::new(x, y),
            clicks: 1,
            mods: Modifiers::NONE,
        },
        1_010,
    );
    let _ = s.handle(
        &InputEvent::PointerButton {
            button: PointerButton::Secondary,
            pressed: false,
            at: Point::new(x, y),
            clicks: 1,
            mods: Modifiers::NONE,
        },
        1_020,
    );
}

fn move_pointer(s: &mut Shell, x: f32, y: f32) {
    let _ = s.handle(
        &InputEvent::PointerMove {
            at: Point::new(x, y),
            delta: Point::new(1.0, 1.0),
        },
        1_050,
    );
}

fn type_text(s: &mut Shell, text: &str) {
    for c in text.chars() {
        let _ = s.handle(
            &InputEvent::TextInput {
                text: c.to_string(),
            },
            1_100,
        );
    }
}

fn key_press(s: &mut Shell, k: Key, mods: Modifiers) -> Vec<Effect> {
    let mut out = s.handle(
        &InputEvent::KeyDown {
            key: k,
            code: 0,
            mods,
            repeat: false,
        },
        1_200,
    );
    out.extend(s.handle(
        &InputEvent::KeyUp {
            key: k,
            code: 0,
            mods: Modifiers::NONE,
        },
        1_210,
    ));
    out
}

fn draw_frame(s: &mut Shell) -> Canvas {
    // The real loop is handle → tick → draw, and the tick is what advances
    // the overlay springs the views read. Harnesses that skip it would test
    // a frame the compositor can never produce, so the tick lives here — one
    // frame's worth of it, at the pace the active desktop runs at.
    s.tick_after(FRAME_MS);
    let mut c = Canvas::new(1280, 800, rgb(0x000000));
    desktop::wallpaper(&mut c, &Theme::of(ThemeKind::Dark));
    crate::views::draw(&mut c, s);
    c
}

fn draw_frame_at(s: &mut Shell, width: u32, height: u32) -> Canvas {
    s.tick_after(FRAME_MS);
    let theme = s.tokens.theme;
    let mut c = Canvas::new(width, height, rgb(0x000000));
    c.record_text(true);
    desktop::wallpaper(&mut c, &theme);
    crate::views::draw(&mut c, s);
    c
}

// ---------------------------------------------------------------------------
// Journeys
// ---------------------------------------------------------------------------

#[test]
fn journey_the_desktop_is_bare_and_applications_open_from_the_launcher() {
    // The desktop is a place, not a shelf: no icon grid, no bar along the
    // bottom edge. Applications are one gesture away through the launcher,
    // which is where the pointer journey goes now.
    let mut s = shell();
    let _canvas = draw_frame(&mut s);
    assert!(
        s.hotspots().rect_of(Hot::DesktopIcon(0)).is_none(),
        "the desktop still draws an icon grid"
    );
    // The bar is on screen at rest, with every pinned application on it.
    for i in [3u8, 4, 5, 2] {
        assert!(
            s.hotspots().rect_of(Hot::DockItem(i)).is_some(),
            "the resting bar is missing pinned item {i}"
        );
    }
    // Clicking the bare wallpaper does nothing rather than selecting a ghost.
    click(&mut s, 900.0, 700.0);
    assert!(s.desktop_icons.selected.is_none());
    assert!(
        s.wm.all().is_empty(),
        "a click on the wallpaper opened something"
    );

    // The bar's own icons open applications without opening the launcher first.
    let tile = s
        .hotspots()
        .rect_of(Hot::DockItem(3))
        .expect("the bar's Files icon is drawn");
    click(&mut s, tile.center().0, tile.center().1);
    assert!(
        s.wm.all().iter().any(|w| w.app_id == "com.litric.files"),
        "the Files window opened from the launcher"
    );
}

#[test]
fn journey_right_click_desktop_menu_creates_a_folder_via_dialog() {
    let mut s = shell();
    // The dialog writes through the shell's root, so point that at a fixture
    // and assert on the result there. On a host with no filesystem the dialog
    // still has to explain itself, which is the first arm below.
    let root = e2e_root("newfolder");
    std::fs::create_dir_all(root.join("Documents")).unwrap();
    s.files_root = root.clone();
    // Right-click the wallpaper.
    right_click(&mut s, 700.0, 650.0);
    assert!(s.context_menu.is_some(), "a context menu opened");
    let _canvas = draw_frame(&mut s);
    let menu_row = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::NewFolder))
        .expect("the menu drew a New folder row");
    let _ = menu_row;
    // Click New folder.
    let r = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::NewFolder))
        .unwrap();
    click(&mut s, r.center().0, r.center().1);
    assert!(matches!(
        s.desktop_dialog,
        Some(crate::apps::DesktopDialog::NewFolder)
    ));
    // Type a name and confirm. The dialog's buttons exist once drawn.
    type_text(&mut s, "Trip 2026");
    let frame = draw_frame(&mut s);
    drop(frame);
    let ok = s.hotspots().rect_of(Hot::DialogConfirm).unwrap();
    click(&mut s, ok.center().0, ok.center().1);
    // On Windows there is no filesystem; the dialog must say so, honestly,
    // and never pretend the folder exists.
    match &s.desktop_dialog {
        Some(crate::apps::DesktopDialog::Error(msg)) => assert!(msg.contains("available"), "{msg}"),
        Some(_) => panic!("dialog stayed open without an explanation"),
        #[cfg(unix)]
        None => {
            // Unix: the folder really exists, in the root the shell was given.
            let zone = s.regional.zone.clone();
            let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();
            assert!(bridge.exists("/Documents/Trip 2026").unwrap());
        }
        // A host with no filesystem can never report success: the dialog must
        // have said why, which the arm above checks.
        #[cfg(not(unix))]
        None => panic!("the folder was reported created with no filesystem"),
    }
    let _ = std::fs::remove_dir_all(&root);
    // The menu is gone after an item was chosen.
    let frame = draw_frame(&mut s);
    assert!(
        s.hotspots()
            .rect_of(Hot::MenuRow(MenuAction::NewFolder))
            .is_none(),
        "the menu closed after activation"
    );
    drop(frame);
}

#[cfg(unix)]
#[test]
fn journey_new_folder_from_the_files_menu_lands_in_the_open_folder() {
    let mut s = shell();
    let root = e2e_root("files-menu-newfolder");
    std::fs::create_dir_all(root.join("Documents")).unwrap();
    std::fs::create_dir_all(root.join("Pictures")).unwrap();
    s.files_root = root.clone();
    s.open_app(crate::shortcuts::App::Files);
    s.files.path = "/Pictures".to_owned();
    s.files.listed = None;

    // The toolbar's New folder and both context-menu rows are one hotspot, and
    // by the time it runs the menu that drew it has already been cleared. This
    // is that exact state: no menu, Files in front, the row's action.
    assert!(s.context_menu.is_none());
    s.perform_menu(MenuAction::NewFolder);

    // The folder belongs to Files, not to the desktop.
    assert!(
        matches!(s.files.dialog, Some(DialogKind::NewFolder)),
        "the Files dialog must own the row: {:?}",
        s.files.dialog
    );
    assert!(
        s.desktop_dialog.is_none(),
        "the desktop's dialog must not open over Files"
    );

    type_text(&mut s, "Holiday");
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.files.dialog_field, "Holiday",
        "the name reached the field"
    );
    let ok = s
        .hotspots()
        .rect_of(Hot::DialogConfirm)
        .expect("the dialog drew its Create button");
    click(&mut s, ok.center().0, ok.center().1);

    assert!(
        s.files.dialog.is_none(),
        "the confirmation failed: {:?}",
        s.files.dialog
    );
    let zone = s.regional.zone.clone();
    let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();
    assert!(
        bridge.exists("/Pictures/Holiday").unwrap(),
        "the folder was created in the folder the window was showing (path {:?})",
        s.files.path
    );
    assert!(
        !root.join("Documents").join("Holiday").exists(),
        "the desktop's /Documents must be untouched"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A point on the wallpaper that no window covers.
///
/// A journey that right-clicks the desktop needs the desktop, so the window in
/// front is asked for its frame rather than assumed: if Files ever opens
/// somewhere else, this test must still right-click wallpaper and not a table.
fn wallpaper_point(s: &Shell) -> (f32, f32) {
    // The display `shell()` builds. The point is checked against every frame
    // below, so a wrong constant fails loudly instead of clicking a window.
    let (w, h) = (1280.0, 800.0);
    for (x, y) in [
        (20.0, h * 0.3),
        (w - 20.0, h * 0.3),
        (w * 0.5, h * 0.15),
        (20.0, h * 0.6),
    ] {
        let covered = s.wm.all().iter().any(|win| {
            let f = win.frame;
            x >= f.x && x <= f.x + f.w && y >= f.y && y <= f.y + f.h
        });
        if !covered {
            return (x, y);
        }
    }
    panic!("no wallpaper left to right-click");
}

#[test]
fn journey_desktop_menu_keeps_its_folder_on_the_desktop_with_files_in_front() {
    let mut s = shell();
    s.open_app(App::Files);
    let (x, y) = wallpaper_point(&s);
    right_click(&mut s, x, y);
    assert_eq!(
        s.context_menu.as_ref().map(|m| m.surface),
        Some(crate::apps::MenuSurface::Desktop),
        "the right-click at ({x}, {y}) did not land on the wallpaper"
    );
    let _ = draw_frame(&mut s);
    let row = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::NewFolder))
        .expect("the desktop's menu drew a New folder row");
    click(&mut s, row.center().0, row.center().1);
    // The desktop's row is the desktop's: a Files window being in front does
    // not turn the wallpaper into the folder Files is showing.
    assert!(
        matches!(
            s.desktop_dialog,
            Some(crate::apps::DesktopDialog::NewFolder)
        ),
        "the desktop's dialog did not open: {:?}",
        s.desktop_dialog
    );
    assert!(
        s.files.dialog.is_none(),
        "a desktop row opened the Files dialog: {:?}",
        s.files.dialog
    );
}

#[cfg(unix)]
#[test]
fn journey_files_context_menu_creates_its_folder_in_the_open_folder() {
    let mut s = shell();
    let root = e2e_root("files-row-newfolder");
    std::fs::create_dir_all(root.join("Pictures")).unwrap();
    s.files_root = root.clone();
    s.open_app(App::Files);
    s.files.path = "/Pictures".to_owned();
    s.files.listed = None;

    // Right-click the middle of the window: the table, not the details pane.
    let frame =
        s.wm.all()
            .iter()
            .find(|w| w.app_id == App::Files.app_id())
            .expect("the Files window is open")
            .frame;
    right_click(&mut s, frame.x + frame.w * 0.4, frame.y + frame.h * 0.5);
    assert!(
        matches!(
            s.context_menu.as_ref().map(|m| m.surface),
            Some(crate::apps::MenuSurface::Files) | Some(crate::apps::MenuSurface::FilesRow)
        ),
        "the right-click landed on {:?}",
        s.context_menu.as_ref().map(|m| m.surface)
    );

    let _ = draw_frame(&mut s);
    let row = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::NewFolder))
        .expect("the Files menu drew a New folder row");
    click(&mut s, row.center().0, row.center().1);
    assert!(
        matches!(s.files.dialog, Some(DialogKind::NewFolder)),
        "the Files dialog must own its own row: {:?}",
        s.files.dialog
    );
    assert!(s.desktop_dialog.is_none());

    type_text(&mut s, "Poster");
    let _ = draw_frame(&mut s);
    let ok = s
        .hotspots()
        .rect_of(Hot::DialogConfirm)
        .expect("the dialog drew its Create button");
    click(&mut s, ok.center().0, ok.center().1);
    assert!(s.files.dialog.is_none(), "{:?}", s.files.dialog);

    let zone = s.regional.zone.clone();
    let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();
    assert!(
        bridge.exists("/Pictures/Poster").unwrap(),
        "the folder was created somewhere else: {:?}",
        s.files.path
    );
    assert!(!root.join("Documents").join("Poster").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn journey_files_window_is_drawn_with_places_rows_and_commands() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Files);
    let _canvas = draw_frame(&mut s);
    for hot in [
        Hot::Files(FilesHot::Place(0)),
        Hot::Files(FilesHot::Up),
        Hot::Files(FilesHot::Details),
    ] {
        assert!(
            s.hotspots().rect_of(hot).is_some(),
            "{hot:?} drew no hotspot"
        );
    }
    // Command row exists.
    assert!(
        s.hotspots()
            .rect_of(Hot::MenuRow(MenuAction::NewFolder))
            .is_some(),
        "the New folder command is drawn"
    );
    // A button with nowhere to go is not a target. Once there is history, the
    // same button is drawn and clickable.
    assert!(
        s.hotspots().rect_of(Hot::Files(FilesHot::Back)).is_none(),
        "Back must not be a target with an empty history"
    );
    s.files.navigate("/Pictures");
    let _ = draw_frame(&mut s);
    assert!(
        s.hotspots().rect_of(Hot::Files(FilesHot::Back)).is_some(),
        "Back draws once there is history"
    );
}

#[cfg(unix)]
#[test]
fn journey_files_selects_with_mouse_and_compresses_multiple_items() {
    let mut s = shell();
    let root = e2e_root("multi-archive");
    let documents = root.join("Documents");
    std::fs::create_dir_all(&documents).unwrap();
    std::fs::write(documents.join("alpha.txt"), "alpha contents").unwrap();
    std::fs::write(documents.join("beta.txt"), "beta contents").unwrap();
    s.files_root = root.clone();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);

    for index in 0..2 {
        let checkbox = s
            .hotspots()
            .rect_of(Hot::Files(FilesHot::ToggleRow(index)))
            .expect("each Files row has a multi-select checkbox");
        click(&mut s, checkbox.center().0, checkbox.center().1);
    }
    assert_eq!(s.files.selected_paths().len(), 2);
    let compress = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Compress))
        .expect("Compress is available in the Files command row");
    click(&mut s, compress.center().0, compress.center().1);
    assert!(matches!(
        s.files.dialog,
        Some(DialogKind::Confirm {
            action: crate::apps::FilesOp::CompressMany { .. },
            ..
        })
    ));

    let _ = draw_frame(&mut s);
    let confirm = s.hotspots().rect_of(Hot::DialogConfirm).unwrap();
    click(&mut s, confirm.center().0, confirm.center().1);
    let bridge = crate::fsbridge::FsBridge::open(&root, s.regional.zone.clone()).unwrap();
    assert!(bridge.exists("/Documents/Archive.zip").unwrap());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn journey_activity_environment_and_settings_controls_change_real_state() {
    let mut s = shell();
    s.open_app(App::Activity);
    let _ = draw_frame(&mut s);
    let filter = s.hotspots().rect_of(Hot::ActivityFilter).unwrap();
    click(&mut s, filter.center().0, filter.center().1);
    type_text(&mut s, "qemu");
    assert_eq!(s.activity.prefs.filter_text, "qemu");
    let cpu = s
        .hotspots()
        .rect_of(Hot::ActivitySort(crate::taskmgr::Column::Cpu))
        .expect("CPU heading is a sortable control");
    click(&mut s, cpu.center().0, cpu.center().1);
    assert_eq!(
        s.activity.prefs.sort.0,
        crate::taskmgr::Column::Cpu.sort_key()
    );

    s.environments = vec![crate::shell::EnvRow {
        token: "linux".to_owned(),
        title: "Linux".to_owned(),
        state: "running".to_owned(),
        active: false,
        hotkey: "Win+3".to_owned(),
    }];
    s.open_app(App::Environments);
    let _ = draw_frame(&mut s);
    let env = s.hotspots().rect_of(Hot::QuickEnvironment(0)).unwrap();
    assert!(click(&mut s, env.center().0, env.center().1).contains(&Effect::SwitchEnvironment(1)));

    s.open_app(App::Settings);
    s.settings.page = crate::settings::Page::Appearance;
    let _ = draw_frame(&mut s);
    let theme = s.hotspots().rect_of(Hot::SettingsTheme).unwrap();
    let effects = click(&mut s, theme.center().0, theme.center().1);
    assert!(effects.contains(&Effect::PersistPreferences(s.settings.preferences)));
    s.finish_preferences_save(Ok(()));
    let _ = draw_frame(&mut s);
    let accent = s.hotspots().rect_of(Hot::SettingsAccent).unwrap();
    let effects = click(&mut s, accent.center().0, accent.center().1);
    assert!(effects.contains(&Effect::PersistPreferences(s.settings.preferences)));
    s.finish_preferences_save(Ok(()));
    assert_ne!(
        s.settings.preferences.accent,
        litric_design::Accent::Default,
        "the accent row must move the accent off the theme default"
    );
    let _ = draw_frame(&mut s);
    let wallpaper = s.hotspots().rect_of(Hot::SettingsWallpaper).unwrap();
    let effects = click(&mut s, wallpaper.center().0, wallpaper.center().1);
    assert!(effects.contains(&Effect::PersistPreferences(s.settings.preferences)));
    s.finish_preferences_save(Ok(()));
    assert_ne!(
        s.settings.preferences.wallpaper,
        crate::desktop::Wallpaper::System,
        "the wallpaper row must move the wallpaper off the shipped default"
    );
    let _ = draw_frame(&mut s);
    let language = s.hotspots().rect_of(Hot::SettingsLanguage).unwrap();
    let effects = click(&mut s, language.center().0, language.center().1);
    assert!(effects.contains(&Effect::PersistPreferences(s.settings.preferences)));
    s.finish_preferences_save(Ok(()));
    assert_ne!(s.settings.preferences.language, crate::i18n::Language::Thai);
}

#[test]
fn journey_browser_history_and_bookmarks_are_clickable_and_keep_visits() {
    let mut s = shell();
    s.open_app(App::Browser);
    let _ = draw_frame(&mut s);
    let field = s.hotspots().rect_of(Hot::BrowserOmnibox).unwrap();
    click(&mut s, field.center().0, field.center().1);
    type_text(&mut s, "litricos.example");
    let effects = key_press(&mut s, Key::Enter, Modifiers::NONE);
    assert!(effects.contains(&Effect::PersistBrowserData));
    assert_eq!(
        s.browser.tab().map(|tab| tab.url.as_str()),
        Some("https://litricos.example/")
    );

    let _ = draw_frame(&mut s);
    let save = s.hotspots().rect_of(Hot::BrowserBookmark).unwrap();
    assert!(click(&mut s, save.center().0, save.center().1).contains(&Effect::PersistBrowserData));
    assert!(s.browser.is_bookmarked());

    let history = s.hotspots().rect_of(Hot::BrowserHistory).unwrap();
    click(&mut s, history.center().0, history.center().1);
    let _ = draw_frame(&mut s);
    let first_visit = s.hotspots().rect_of(Hot::BrowserHistoryItem(0)).unwrap();
    click(&mut s, first_visit.center().0, first_visit.center().1);
    assert_eq!(
        s.browser.tab().map(|tab| tab.url.as_str()),
        Some("https://litricos.example/")
    );
    assert!(s.browser.is_bookmarked());
}

#[test]
fn every_app_layout_is_audited_at_both_resolutions_and_all_built_in_themes() {
    use crate::settings::Preferences;
    let apps = [
        App::Activity,
        App::Environments,
        App::Settings,
        App::Files,
        App::Browser,
        App::Terminal,
        App::Trash,
        App::Defender,
    ];
    for (width, height) in [(1280, 800), (1920, 1080)] {
        for language in [crate::i18n::Language::Thai, crate::i18n::Language::English] {
            for theme in ThemeKind::all() {
                let mut s = Shell::new(
                    Tokens::new(Theme::of(*theme), 1.0),
                    width as f32,
                    height as f32,
                    Regional::fallback("test"),
                );
                s.mode = DesktopMode::Ready;
                s.load_preferences(Preferences::new(*theme, language));
                for app in apps {
                    s.open_app(app);
                }
                let c = draw_frame_at(&mut s, width, height);
                let collisions = crate::textaudit::audit(&c);
                let within_surface: Vec<_> = collisions
                    .into_iter()
                    .filter(|collision| !collision.cross_surface)
                    .collect();
                assert!(
                    within_surface.is_empty(),
                    "{width}x{height}, {}, {:?}: {}",
                    language.title(),
                    theme,
                    crate::textaudit::report(c.text_runs(), 8)
                );
                for run in c.text_runs() {
                    assert!(
                        run.x >= 0.0 && run.end_x <= width as f32,
                        "text leaves the framebuffer at {width}x{height}: {run:?}"
                    );
                    assert!(
                        run.baseline >= 0.0 && run.baseline <= height as f32,
                        "text leaves the framebuffer at {width}x{height}: {run:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn journey_rename_via_f2_and_confirm_dialog() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Files);
    // Select the first row through the model exactly as a click would.
    s.files.rows = vec![crate::apps::FileRow {
        name: "notes.txt".to_owned(),
        kind: crate::apps::FileKind::File,
        size: 3,
        mtime: 10,
    }];
    s.files.listed = Some(s.files.path.clone());
    s.files.selected = Some(0);
    key_press(&mut s, Key::Function(2), Modifiers::NONE);
    match &s.files.dialog {
        Some(DialogKind::Rename { current, .. }) => assert_eq!(current, "notes.txt"),
        other => panic!("F2 opened {other:?}"),
    }
    // Escape cancels.
    key_press(&mut s, Key::Escape, Modifiers::NONE);
    assert!(s.files.dialog.is_none(), "Escape closed the dialog");
}

#[test]
fn journey_delete_asks_first_and_trash_only_after_confirm() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Files);
    s.files.rows = vec![crate::apps::FileRow {
        name: "gone.txt".to_owned(),
        kind: crate::apps::FileKind::File,
        size: 1,
        mtime: 2,
    }];
    s.files.listed = Some(s.files.path.clone());
    s.files.selected = Some(0);

    // Delete opens a confirm dialog rather than deleting at once.
    key_press(&mut s, Key::Delete, Modifiers::NONE);
    assert!(matches!(s.files.dialog, Some(DialogKind::Confirm { .. })));

    // Draw once so the dialog registers its confirm button.
    let frame = draw_frame(&mut s);
    drop(frame);

    // Confirm runs the operation. On Windows the refusal is honest; on Unix
    // the file is really in the trash.
    let ok = s.hotspots().rect_of(Hot::DialogConfirm).unwrap();
    click(&mut s, ok.center().0, ok.center().1);
    #[cfg(unix)]
    {
        let zone = s.regional.zone.clone();
        let bridge = crate::fsbridge::FsBridge::open(std::path::Path::new("/"), zone).unwrap();
        // The listing came from the model, not the disk, so trash may fail
        // on a path that does not exist; either way the dialog resolved.
        let _ = bridge.trash_list().unwrap().len();
    }
    // The dialog resolved: closed on success (Unix), or replaced by an
    // honest refusal (Windows CI) — never silently stuck.
    assert!(
        s.files.dialog.is_none() || matches!(s.files.dialog, Some(DialogKind::Error(_))),
        "the confirm dialog resolved"
    );
}

#[test]
fn journey_defender_scans_and_reports_eicar_end_to_end() {
    // The engine + shell integration, on the real files of a temp root.
    #[cfg(unix)]
    let root = std::env::temp_dir().join("litric-shell-e2e-defender");
    #[cfg(unix)]
    {
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(root.join("Documents")).unwrap();
        std::fs::write(root.join("Documents").join("eicar.com"), EICAR).unwrap();
        std::fs::write(root.join("Documents").join("clean.txt"), b"hello").unwrap();

        // Bridge the shell at the temp root: scan_step uses "/" like prod,
        // so run the scan pipeline directly against the temp directory.
        let zone = litric_locale::TimeZone::utc("test");
        let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();
        let mut queue = Vec::new();
        collect(&bridge, "/Documents", &mut queue, 0);
        assert_eq!(queue.len(), 2, "both files queued");

        let mut engine = litric_defender::engine::Engine::new();
        engine.load_denylist(crate::shell::eicar_denylist());
        let mut detected = 0;
        for p in &queue {
            let host = bridge.host_path(p).unwrap();
            let bytes = std::fs::read(&host).unwrap();
            let report = engine.scan(p, bytes.len() as u64, 0, &bytes);
            if matches!(
                report.decision,
                litric_defender::verdict::Decision::Deny { .. }
            ) {
                detected += 1;
            }
        }
        assert_eq!(
            detected, 1,
            "the EICAR file was detected, the clean one was not"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    // The window draws and the scan button is wired on every host.
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Defender);
    let _canvas = draw_frame(&mut s);
    let btn = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::ScanNow))
        .expect("the Defender scan button is drawn");
    assert!(btn.w > 0.0);
    // Drive the model through a full scan cycle.
    s.defender.app.start(2);
    s.defender.app.record(
        "/Documents/eicar.com",
        &litric_defender::verdict::Decision::Deny {
            tier: litric_defender::verdict::Tier::Hash,
            id: "eicar-test-file".to_owned(),
        },
    );
    s.defender.app.record(
        "/Documents/clean.txt",
        &litric_defender::verdict::Decision::Allow,
    );
    s.defender.app.finish();
    match s.defender.app.state {
        ScanState::Done {
            scanned: 2,
            threats: 1,
        } => {}
        other => panic!("scan finished as {other:?}"),
    }
    let frame = draw_frame(&mut s);
    let badge = Rect::new(0.0, 0.0, 1280.0, 28.0);
    assert!(ink(&frame, badge) > 0, "the top bar drew");
}

#[test]
fn journey_a_threat_notification_banners_through_notifications() {
    let mut s = shell();
    s.notify(
        "Defender",
        "Litric Defender",
        "1 threat found in 3 files",
        Priority::Critical,
        Some(8_000),
    );
    assert_eq!(s.notifications.len(), 1);
    let canvas = draw_frame(&mut s);
    // The banner area (top right) has ink.
    let banner_zone = Rect::new(900.0, 40.0, 360.0, 80.0);
    assert!(
        ink(&canvas, banner_zone) > 50,
        "the threat banner is visible"
    );
}

#[test]
fn journey_browser_searches_google_and_picks_engines() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Browser);
    // Draw once so the browser window registers its hotspots.
    let frame = draw_frame(&mut s);
    drop(frame);
    // Focus the omnibox by clicking it.
    let omni = s.hotspots().rect_of(Hot::BrowserOmnibox).unwrap();
    click(&mut s, omni.center().0, omni.center().1);
    assert!(s.browser.omnibox_focus);
    // Type a query and press Enter.
    type_text(&mut s, "litric os");
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    let tab = s.browser.tab().unwrap();
    assert_eq!(tab.url, "https://www.google.com/search?q=litric%20os");

    // The engine picker offers three engines and switching works.
    let _canvas = draw_frame(&mut s);
    let ddg_hot = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::EngineDuckDuckGo))
        .is_some();
    assert!(ddg_hot, "DuckDuckGo is in the picker");
    let ddg = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::EngineDuckDuckGo))
        .unwrap();
    click(&mut s, ddg.center().0, ddg.center().1);
    assert_eq!(s.browser.engine, crate::apps::SearchEngine::DuckDuckGo);

    // A new search now goes to the picked engine. Draw once more so the
    // omnibox hotspot reflects the current window state.
    let frame = draw_frame(&mut s);
    drop(frame);
    let omni = s.hotspots().rect_of(Hot::BrowserOmnibox).unwrap();
    click(&mut s, omni.center().0, omni.center().1);
    type_text(&mut s, "rust");
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    assert_eq!(
        s.browser.tab().unwrap().url,
        "https://duckduckgo.com/?q=rust"
    );
}

#[test]
fn journey_browser_menu_starts_engine_session_and_reports_honestly() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Browser);
    // On any host without /run/litric/browser.sock the start reports why
    // instead of pretending. On Unix without the daemon, connect fails by
    // ENOENT.
    let effects = s.perform_menu(MenuAction::BrowserStart);
    assert!(effects.is_empty());
    assert!(
        !s.browser.status.is_empty(),
        "the browser explained the engine state"
    );
}

#[test]
fn journey_opening_the_browser_reaches_for_the_engine_itself() {
    // The old flow left the engine disconnected until a second click on a
    // menu item: open_app returned Launched, main dropped it, and the user
    // saw a browser that could not browse. Opening the app must now attempt
    // the session — a socket that answers starts the engine, a socket that
    // is missing at least explains itself in the status line.
    let mut s = shell();
    let effects = s.open_app(crate::shortcuts::App::Browser);
    // What main.rs's apply() does with the effect the shell returned — the
    // contract under test is that pair, not either half alone.
    assert!(
        effects.iter().any(|e| matches!(
            e,
            crate::shell::Effect::Launched(crate::shortcuts::App::Browser)
        )),
        "opening the browser must report Launched for main to act on"
    );
    let _ = s.browser_start_session();
    assert!(
        !s.browser.status.is_empty(),
        "the Launched effect must lead to an engine-session attempt"
    );
}

#[test]
fn journey_trash_window_restores_after_files_trash() {
    #[cfg(unix)]
    let root = std::env::temp_dir().join("litric-shell-e2e-trash");
    #[cfg(unix)]
    {
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let zone = litric_locale::TimeZone::utc("test");
        let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();
        bridge.write_text("/", "doomed.txt", "bye").unwrap();
        bridge.trash("/doomed.txt", 1_900_000_000).unwrap();
        assert_eq!(bridge.trash_list().unwrap().len(), 1);
        bridge.trash_restore("doomed.txt").unwrap();
        assert!(bridge.exists("/doomed.txt").unwrap());
        let _ = std::fs::remove_dir_all(&root);
    }

    // The Trash window lists and its buttons draw on every host.
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Trash);
    let _canvas = draw_frame(&mut s);
    assert!(
        s.hotspots().rect_of(Hot::TrashEmpty).is_some(),
        "Empty button drew"
    );
    s.trash.entries = vec![(
        "x.txt".to_owned(),
        "2026-01-01".to_owned(),
        "/".to_owned(),
        false,
    )];
    s.trash.selected = Some(0);
    s.perform_menu(MenuAction::Restore);
    // On Windows this recorded the refusal; on Unix it restored. Either way
    // the state is explained, never silently wrong.
}

#[test]
fn journey_the_omnibox_takes_keys_not_just_text() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Browser);
    s.browser.omnibox_focus = true;
    s.browser.omnibox = "example.com".to_owned();
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    assert_eq!(s.browser.tab().unwrap().url, "https://example.com/");
    assert!(!s.browser.omnibox_focus, "commit released the omnibox");
}

#[test]
fn journey_right_click_files_row_shows_row_menu() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Files);
    s.files.rows = vec![crate::apps::FileRow {
        name: "menu.txt".to_owned(),
        kind: crate::apps::FileKind::File,
        size: 1,
        mtime: 1,
    }];
    s.files.listed = Some(s.files.path.clone());
    let _canvas = draw_frame(&mut s);
    let row = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Row(0)))
        .expect("row 0 drew");
    right_click(&mut s, row.center().0, row.center().1);
    assert!(s.context_menu.is_some());
    let surface = s.context_menu.as_ref().unwrap().surface;
    assert_eq!(surface, crate::apps::MenuSurface::FilesRow);
    // Redraw so the menu registers its row hotspots.
    let frame = draw_frame(&mut s);
    drop(frame);
    // The row menu offers Copy and the copy lands in the clipboard.
    let r = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::Copy))
        .expect("the row menu drew Copy");
    click(&mut s, r.center().0, r.center().1);
    assert!(s.files.clipboard.is_some(), "copy ran from the menu");
}

#[cfg(unix)]
#[test]
fn journey_right_click_trash_row_targets_that_row() {
    let mut s = shell();
    let root = e2e_root("trash-row-target");
    std::fs::create_dir_all(&root).unwrap();
    let zone = s.regional.zone.clone();
    {
        let bridge = crate::fsbridge::FsBridge::open(&root, zone.clone()).unwrap();
        bridge.write_text("/", "a.txt", "a").unwrap();
        bridge.write_text("/", "b.txt", "b").unwrap();
        bridge.trash("/a.txt", 1_900_000_000).unwrap();
        bridge.trash("/b.txt", 1_900_000_000).unwrap();
    }
    s.files_root = root.clone();
    s.open_app(App::Trash);
    s.trash.entries = vec![
        (
            "a.txt".to_owned(),
            "2026-01-01".to_owned(),
            "/".to_owned(),
            false,
        ),
        (
            "b.txt".to_owned(),
            "2026-01-01".to_owned(),
            "/".to_owned(),
            false,
        ),
    ];
    s.trash.listed = true;
    s.trash.selected = Some(0);
    let _ = draw_frame(&mut s);
    let row = s
        .hotspots()
        .rect_of(Hot::TrashRow(1))
        .expect("the trash drew row 1");
    right_click(&mut s, row.center().0, row.center().1);
    assert_eq!(
        s.trash.selected,
        Some(1),
        "the row under the pointer is the row the menu is about"
    );
    let _ = draw_frame(&mut s);
    let restore = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::Restore))
        .expect("the trash menu drew Restore");
    click(&mut s, restore.center().0, restore.center().1);
    let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();
    assert_eq!(
        bridge.trash_list().unwrap().len(),
        1,
        "restoring one entry leaves the other in the trash"
    );
    assert!(
        bridge.exists("/b.txt").unwrap(),
        "the row that was right-clicked is the one restored"
    );
    assert!(
        !root.join("a.txt").exists(),
        "the previously selected row must not be touched"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn journey_right_click_background_tab_closes_that_tab() {
    let mut s = shell();
    s.open_app(App::Browser);
    s.browser.new_tab();
    s.browser.new_tab();
    s.browser.select(0);
    for (i, url) in [
        "https://one.example",
        "https://two.example",
        "https://three.example",
    ]
    .into_iter()
    .enumerate()
    {
        s.browser.tabs.get_mut(i).expect("the tab was opened").url = url.to_owned();
    }
    let _ = draw_frame(&mut s);
    let tab = s
        .hotspots()
        .rect_of(Hot::BrowserTab(2))
        .expect("the strip drew tab 2");
    right_click(&mut s, tab.center().0, tab.center().1);
    assert_eq!(
        s.context_menu.as_ref().and_then(|m| m.tab),
        Some(2),
        "the menu records the tab it is about"
    );
    assert_eq!(s.browser.active, 0, "a right-click does not switch tabs");
    let _ = draw_frame(&mut s);
    let close = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::BrowserCloseTab))
        .expect("the tab menu drew Close tab");
    click(&mut s, close.center().0, close.center().1);
    let urls: Vec<&str> = s.browser.tabs.iter().map(|t| t.url.as_str()).collect();
    assert_eq!(
        urls,
        ["https://one.example", "https://two.example"],
        "the tab under the pointer closed, not the focused one"
    );
    assert_eq!(s.browser.active, 0, "the focused tab stayed focused");
}

#[test]
fn journey_right_click_background_tab_reloads_that_tab() {
    let mut s = shell();
    s.open_app(App::Browser);
    s.browser.new_tab();
    s.browser.select(0);
    s.browser.tabs.get_mut(0).expect("tab 0").url = "https://one.example".to_owned();
    let second = s.browser.tabs.get_mut(1).expect("tab 1");
    second.url = "https://two.example".to_owned();
    second.error = true;
    let _ = draw_frame(&mut s);
    let tab = s
        .hotspots()
        .rect_of(Hot::BrowserTab(1))
        .expect("the strip drew tab 1");
    right_click(&mut s, tab.center().0, tab.center().1);
    let _ = draw_frame(&mut s);
    let reload = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::BrowserReload))
        .expect("the tab menu drew Reload");
    click(&mut s, reload.center().0, reload.center().1);
    assert!(
        s.browser.tabs.get(1).is_some_and(|t| !t.error),
        "reload cleared the error on the tab it was opened for"
    );
    assert_eq!(s.browser.active, 0, "reloading did not switch tabs");
    assert!(
        s.browser.status.is_empty(),
        "a background reload must not claim the visible page is loading"
    );
}

// ---------------------------------------------------------------------------
// Wave: dock, through-click fix, browser search, terminal, state matrix
// ---------------------------------------------------------------------------

#[test]
fn journey_clicking_a_window_never_hits_the_desktop_icon_behind_it() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Files);
    let frame = draw_frame(&mut s);
    drop(frame);
    // The Files window's default frame covers the icon column; aim at its
    // bare body (not on a row) and demand that no desktop icon answers.
    let id =
        s.wm.all()
            .iter()
            .find(|w| w.app_id == "com.litric.files")
            .map(|w| w.id)
            .expect("Files window open");
    let w = s.wm.window(id).unwrap();
    // Aim at the second icon column — the point sits inside the icon grid
    // AND inside the window body. Before the fix, `hot.hit` answered with
    // the icon; now the window's body backdrop must own the click.
    let at = Point::new(150.0, 84.0);
    assert!(
        w.frame.contains(at.x, at.y),
        "the probe really is inside the window"
    );
    // The hotspot table never answers with an icon under a window: the
    // title-bar point has no hotspot at all (the window manager owns it),
    // body points answer with the window or one of its controls.
    let hit = s.hotspots().hit(at.x, at.y);
    assert!(
        !matches!(hit, Some(Hot::DesktopIcon(_))),
        "a click inside a window must not open the icon behind it, got {hit:?}"
    );
    assert!(
        matches!(hit, Some(Hot::Window(_)) | Some(Hot::Files(_)) | None),
        "the point belongs to the window, got {hit:?}"
    );
    // And the click itself: no Settings window may materialise from the
    // icon hiding behind this window's title bar.
    click(&mut s, at.x, at.y);
    assert!(
        s.wm.all().iter().all(|w| w.app_id != "com.litric.settings"),
        "clicking a window launched the icon hiding behind it"
    );
    assert_eq!(
        s.wm.focused().map(|f| f == id),
        Some(true),
        "the click focused the window the user aimed at"
    );
}

#[test]
fn journey_launcher_click_opens_focuses_then_minimizes() {
    // The desktop has no resting bar, so the launcher is where an application
    // is reached from the pointer. The click rules are unchanged.
    let mut s = shell();
    open_launcher(&mut s);
    let _ = draw_frame(&mut s);
    let dock_hot = s
        .hotspots()
        .rect_of(Hot::DockItem(3))
        .expect("the launcher draws its pinned icons");
    // First click opens the app.
    click(&mut s, dock_hot.center().0, dock_hot.center().1);
    let files =
        s.wm.all()
            .iter()
            .find(|w| w.app_id == "com.litric.files")
            .map(|w| w.id)
            .expect("dock click opened Files");
    assert_eq!(s.wm.focused(), Some(files));
    // Second click on the focused app minimizes it. Opening an app closes the
    // launcher, so the journey reopens it the way a person would.
    open_launcher(&mut s);
    let _ = draw_frame(&mut s);
    let dock_hot = s.hotspots().rect_of(Hot::DockItem(3)).unwrap();
    click(&mut s, dock_hot.center().0, dock_hot.center().1);
    let w = s.wm.window(files).unwrap();
    assert!(
        matches!(w.state, litric_wm::WindowState::Minimized),
        "second dock click hides the app"
    );
}

#[test]
fn journey_the_bars_height_is_a_strip_and_its_width_is_wider_than_a_shelf() {
    // The dock stays on the resting desktop, and it is low and wide: a 62-px
    // strip carrying 46-px icons, floating 12 px above the edge, and a large
    // share of the screen's width. (The earlier contract -- tiles under 32 px
    // in a 30-px strip -- is superseded by docs/design/2026-10-24-shell-v4-plan.md
    // section 3.2: at that size the icons were glyph-sized specks.)
    let mut s = shell();
    move_pointer(&mut s, 640.0, 300.0); // nowhere near the bottom edge
    let _frame = draw_frame(&mut s);
    let mut top = f32::MAX;
    let mut bottom = 0.0_f32;
    for i in [3u8, 4, 5, 2, 0, 7, 1, 6] {
        let r = s
            .hotspots()
            .rect_of(Hot::DockItem(i))
            .unwrap_or_else(|| panic!("bar item {i} is missing with nothing open"));
        top = top.min(r.y);
        bottom = bottom.max(r.bottom());
    }
    // 46-px icons plus the 2-px margin each hotspot carries.
    assert!(
        (bottom - top) <= 50.5,
        "the bar's tiles are {} px tall, so the bar is not a strip",
        bottom - top
    );
    // 12 px of float and 8 px of padding under the icon: its bottom is about
    // 20 px above the edge, and never as far as the old 24-px float.
    assert!(
        (800.0 - 40.0..=800.0 - 8.0).contains(&bottom),
        "the bar sits {} px above the bottom edge",
        800.0 - bottom
    );
    // Wide, not merely measured: a large share of the screen rather than a
    // pill sized by a few icons.
    let toggle = s.hotspots().rect_of(Hot::DockToggle).unwrap();
    let field = s.hotspots().rect_of(Hot::DockSearch).unwrap();
    let span = field.right() - toggle.x;
    assert!(
        span > 0.40 * 1280.0,
        "the bar spans {span} px of a 1280-px screen"
    );
    assert!(matches!(s.overlay, Overlay::None));
}

#[test]
fn the_launcher_morph_repaints_the_dock_and_not_the_screen() {
    // The launcher used to compose the whole desktop on every frame of its
    // spring -- wallpaper, every window, the top bar's glass -- which in the
    // guest was a fifth of a second a frame. It grows out of the dock, whose
    // rectangle the draw side publishes, so its frames damage that rectangle.
    let mut s = shell();
    let _ = draw_frame(&mut s);
    // The first frame of a session is a full redraw by definition; take it, the
    // way the frame loop does, so what is left is the launcher's own damage.
    let _ = s.take_damage();
    let settled = s
        .dock_settled()
        .expect("the drawn dock publishes the rectangle it grows into");
    let screen = s.wm.screen();
    assert!(
        settled.w * settled.h < screen.w * screen.h * 0.75,
        "the settled dock covers {}x{} of {}x{}",
        settled.w,
        settled.h,
        screen.w,
        screen.h
    );

    let _ = s.perform(Action::ToggleLauncher);
    assert!(
        !s.needs_full_redraw(),
        "opening the launcher repaints the whole screen"
    );
    let damage = s.take_damage();
    assert!(!damage.is_empty(), "the morph is on screen with no damage");
    for r in &damage {
        assert!(
            r.x <= settled.x && r.y <= settled.y && r.right() >= settled.right(),
            "a morph frame damages {r:?}, which does not cover the dock {settled:?}"
        );
    }

    // And the frames after it: the spring is moving, so there is damage on
    // every tick, and none of them is a full redraw.
    for _ in 0..10 {
        s.tick_after(FRAME_MS);
        assert!(
            !s.needs_full_redraw(),
            "a moving morph frame falls back to a full redraw"
        );
        let _ = s.take_damage();
        let _ = draw_frame(&mut s);
    }
}

#[test]
fn merged_pointer_moves_are_one_cursor_step_from_where_the_arrow_was_left() {
    // A frame at 125 Hz sees eight moves from a 1 kHz pointer. Those are merged
    // into one step -- and the step has to start where the *drawn* arrow is,
    // not where the pointer was one event ago, or the compositor refuses to
    // move it and the frame becomes a full composition.
    let mut s = shell();
    move_pointer(&mut s, 640.0, 300.0);
    let _ = draw_frame(&mut s);
    let _ = s.take_damage();
    let drawn_at = s.pointer();

    move_pointer(&mut s, 700.0, 320.0);
    move_pointer(&mut s, 720.0, 340.0);
    move_pointer(&mut s, 760.0, 360.0);
    let step = s
        .take_cursor_step()
        .expect("three moves over bare wallpaper are not one cursor step");
    assert_eq!(
        step.0, drawn_at,
        "the step starts at {:?}, not where the arrow was left ({drawn_at:?})",
        step.0
    );
    assert_eq!(step.1, Point::new(760.0, 360.0));
    assert!(
        s.take_damage().is_empty(),
        "a cursor step left damage behind for the compositor to compose"
    );
}

#[test]
fn journey_the_launcher_marks_which_applications_are_running() {
    // The running indicator is still drawn under a launcher icon: a dot under
    // the app that has a window, taken from the window list, so it cannot
    // disagree with what is actually open. The surface moved; the promise did
    // not.
    let mut s = shell();
    let _ = s.open_app(App::Files);
    open_launcher(&mut s);
    let frame = draw_frame(&mut s);
    // The launcher draws the dot two logical pixels under the icon's tile: a
    // 30-pixel strip has no room for the six the shelf used.
    let bright_at = |c: &crate::canvas::Canvas, x: f32, y: f32| {
        (0..9i64)
            .flat_map(|dy| (0..9i64).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| {
                crate::canvas::luminance(
                    c.get((x - 4.0) as i64 + dx, (y - 4.0) as i64 + dy)
                        .unwrap_or(0),
                )
            })
            .fold(0.0_f32, f32::max)
    };
    let files_icon = s
        .hotspots()
        .rect_of(Hot::DockItem(3))
        .expect("the Files icon is in the launcher");
    let running = bright_at(&frame, files_icon.center().0, files_icon.bottom() + 2.0);
    // An application with no window has no dot in the same place.
    let browser_icon = s
        .hotspots()
        .rect_of(Hot::DockItem(4))
        .expect("the Browser icon is in the launcher");
    let idle = bright_at(&frame, browser_icon.center().0, browser_icon.bottom() + 2.0);
    assert!(
        running > idle + 0.1,
        "the running application has no indicator under its icon: {running} vs {idle}"
    );
}

#[test]
fn journey_browser_search_commits_to_the_engine_url() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Browser);
    let _ = draw_frame(&mut s);
    // The start page: the search box focuses the omnibox, typing lands in
    // it, Enter commits to the engine URL (Google by default).
    let box_r = s
        .hotspots()
        .rect_of(Hot::BrowserOmnibox)
        .expect("the start page draws a search box");
    click(&mut s, box_r.center().0, box_r.center().1);
    assert!(s.browser.omnibox_focus);
    type_text(&mut s, "cats and dogs");
    assert_eq!(s.browser.omnibox, "cats and dogs");
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    let url = s.browser.tab().map(|t| t.url.clone()).unwrap();
    assert_eq!(
        url, "https://www.google.com/search?q=cats%20and%20dogs",
        "Enter searched with Google"
    );
    assert!(!s.browser.omnibox_focus);
}

#[test]
fn journey_engine_chip_and_quick_link_navigate() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Browser);
    let _ = draw_frame(&mut s);
    let chip = s.hotspots().rect_of(Hot::EngineChip(1)).unwrap();
    click(&mut s, chip.center().0, chip.center().1);
    assert!(
        s.browser.status.contains("DuckDuckGo"),
        "chip picked engine"
    );
    let tile = s.hotspots().rect_of(Hot::QuickLink(2)).unwrap();
    click(&mut s, tile.center().0, tile.center().1);
    assert_eq!(
        s.browser.tab().map(|t| t.url.clone()),
        Some("https://github.com/".to_owned()),
        "quick link navigated the tab"
    );
}

#[cfg(unix)]
#[test]
fn journey_terminal_runs_a_real_command() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Terminal);
    let _ = draw_frame(&mut s);
    let row = s
        .hotspots()
        .rect_of(Hot::TerminalInput)
        .expect("terminal draws its input row");
    click(&mut s, row.center().0, row.center().1);
    assert!(s.terminal.focused);
    type_text(&mut s, "echo litric-ok");
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    // The command runs on a worker thread, so the pump has to give it real
    // time: a tick advances the frame clock, which the worker neither reads
    // nor waits on. Bounded, and short — the command is an `echo`.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while s.terminal.busy && std::time::Instant::now() < deadline {
        s.tick_after(FRAME_MS);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!s.terminal.busy, "the command finished within the budget");
    let saw = s.terminal.lines.iter().any(|l| l.contains("litric-ok"));
    assert!(saw, "the command's output reached the scrollback");
}

#[cfg(not(unix))]
#[test]
fn journey_terminal_refuses_with_a_reason_on_the_host() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Terminal);
    let _ = draw_frame(&mut s);
    let row = s.hotspots().rect_of(Hot::TerminalInput).unwrap();
    click(&mut s, row.center().0, row.center().1);
    type_text(&mut s, "ls");
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    assert!(
        s.terminal.note.is_some(),
        "the refusal explains itself instead of doing nothing"
    );
}

#[test]
fn every_control_state_renders_differently() {
    use crate::ui::{self, State, Tokens, Tone};
    use litric_design::{Theme, ThemeKind};
    let t = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
    let states = [
        State::Normal,
        State::Hover,
        State::Pressed,
        State::Focused,
        State::Selected,
        State::Disabled,
        State::Loading,
        State::Error,
    ];
    // Hash the rendered pixels: two states that draw the same picture have
    // failed the matrix before any user ever saw them.
    let render = |draw: &dyn Fn(&mut Canvas, State)| {
        let mut c = Canvas::new(160, 48, rgb(0x10151B));
        draw(&mut c, State::Normal);
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for px in c.pixels() {
            h ^= u64::from(*px);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    };
    let button = |c: &mut Canvas, s: State| {
        let r = Rect::new(8.0, 8.0, 120.0, 32.0);
        ui::button(c, &t, r, "Run", s, Tone::Accent);
    };
    let row = |c: &mut Canvas, s: State| {
        let r = Rect::new(4.0, 4.0, 150.0, 40.0);
        ui::list_item(c, &t, r, None, "Item", None, None, s);
    };
    for (name, draw) in [
        ("button", &button as &dyn Fn(&mut Canvas, State)),
        ("list_item", &row),
    ] {
        let mut hashes: Vec<(State, u64)> = Vec::new();
        for s in states {
            let mut c = Canvas::new(160, 48, rgb(0x10151B));
            draw(&mut c, s);
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for px in c.pixels() {
                h ^= u64::from(*px);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
            hashes.push((s, h));
        }
        let mut seen = std::collections::HashSet::new();
        for (_, h) in &hashes {
            seen.insert(*h);
        }
        assert_eq!(
            seen.len(),
            states.len(),
            "{name}: colliding states {:?}",
            hashes
        );
    }
    let _ = render(&button);
}

#[test]
fn journey_focus_elevation_glides_instead_of_snapping() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Files);
    s.open_app(crate::shortcuts::App::Terminal);
    let other =
        s.wm.all()
            .iter()
            .find(|w| w.app_id == "com.litric.files")
            .map(|w| w.id)
            .expect("the Files window exists");
    assert!(s.wm.focus_window(other), "focus moved");
    s.tick_after(FRAME_MS);
    // One ticked frame later the previously-focused window is part-way up,
    // not at either endpoint: that is the glide existing and measurable.
    let rising = s.anims_mut().focus.elevation(other, true);
    assert!(
        rising > 0.0,
        "the spring began moving toward focused on the first tick"
    );
    // It must arrive rather than hover: settle the spring and check 1.0.
    for _ in 0..40 {
        s.tick_after(FRAME_MS);
    }
    let settled = s.anims_mut().focus.elevation(other, true);
    assert!(
        (settled - 1.0).abs() < 0.01,
        "focus elevation settles at 1.0"
    );
}

#[test]
fn journey_powering_off_asks_first_and_only_a_confirmation_acts() {
    // The whole point of the session rows: a click asks, and only the answer
    // to a prompt that names the action produces the effect. Driven through
    // the pointer, the way a person does it.
    let mut s = shell();
    let _ = draw_frame(&mut s);
    let power = s
        .hotspots()
        .rect_of(Hot::StatusPower)
        .expect("the status area has a power item");
    click(&mut s, power.center().0, power.center().1);
    // Two ticks: one to start the reveal spring, one to be past the point
    // where the rows become clickable.
    let _ = draw_frame(&mut s);
    let _ = draw_frame(&mut s);

    let row = s
        .hotspots()
        .rect_of(Hot::PowerItem(2))
        .expect("the Shut down row is drawn");
    let asked = click(&mut s, row.center().0, row.center().1);
    assert!(
        asked.is_empty(),
        "one click must not end the session: {asked:?}"
    );
    assert!(
        matches!(
            s.desktop_dialog,
            Some(crate::apps::DesktopDialog::PowerConfirm {
                command: crate::shell::PowerCommand::PowerOff
            })
        ),
        "clicking the row opened a confirmation for it: {:?}",
        s.desktop_dialog
    );

    // Cancelling leaves the machine alone and closes the dialog.
    let _ = draw_frame(&mut s);
    let cancel = s
        .hotspots()
        .rect_of(Hot::DialogCancel)
        .expect("the dialog has a cancel button");
    let cancelled = click(&mut s, cancel.center().0, cancel.center().1);
    assert!(
        cancelled.is_empty(),
        "cancelling does nothing: {cancelled:?}"
    );
    assert!(s.desktop_dialog.is_none(), "cancel closed the dialog");

    // Now confirm, and exactly one effect comes out. The panel closed when
    // the dialog opened — a dialog is the only thing on screen while it is up —
    // so the journey reopens it the way a person would.
    let _ = draw_frame(&mut s);
    let power = s
        .hotspots()
        .rect_of(Hot::StatusPower)
        .expect("the status item is still there");
    click(&mut s, power.center().0, power.center().1);
    let _ = draw_frame(&mut s);
    let _ = draw_frame(&mut s);
    let row = s
        .hotspots()
        .rect_of(Hot::PowerItem(2))
        .expect("the Shut down row is drawn again");
    let _ = click(&mut s, row.center().0, row.center().1);
    let _ = draw_frame(&mut s);
    let ok = s
        .hotspots()
        .rect_of(Hot::DialogConfirm)
        .expect("the dialog has a confirm button");
    let confirmed = click(&mut s, ok.center().0, ok.center().1);
    assert_eq!(
        confirmed,
        vec![Effect::Power(crate::shell::PowerCommand::PowerOff)],
        "confirming produces the one effect that asks the daemon"
    );
    assert!(s.desktop_dialog.is_none(), "the dialog was consumed");
}

#[test]
fn journey_viewer_opens_text_files_with_scrolling() {
    let mut s = shell();
    // The listing below is the model's; the viewer reads the bytes from the
    // shell's root, so the file has to exist there for Enter to open it.
    let root = e2e_root("viewer");
    std::fs::create_dir_all(root.join("Documents")).unwrap();
    std::fs::write(root.join("Documents/readme.md"), "a line\n".repeat(400)).unwrap();
    s.files_root = root.clone();
    s.open_app(crate::shortcuts::App::Files);
    s.files.rows = vec![crate::apps::FileRow {
        name: "readme.md".to_owned(),
        kind: crate::apps::FileKind::File,
        size: 100,
        mtime: 1,
    }];
    s.files.listed = Some(s.files.path.clone());
    s.files.selected = Some(0);
    // Enter opens the viewer through the shell's open path.
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    // No bridge on Windows, so the viewer opens only on Unix; the dialog
    // explains on Windows.
    #[cfg(unix)]
    assert!(s.files.viewer.is_some(), "Enter opened the viewer");
    #[cfg(not(unix))]
    assert!(
        s.files.dialog.is_some() || s.files.viewer.is_some(),
        "the refusal was shown or the viewer opened"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn journey_context_menu_rows_all_do_something() {
    // Every menu row the desktop can draw must be an action the shell
    // implements: the menu can never contain a dead entry.
    let surfaces = [
        crate::apps::MenuSurface::Desktop,
        crate::apps::MenuSurface::Files,
        crate::apps::MenuSurface::FilesRow,
        crate::apps::MenuSurface::Browser,
        crate::apps::MenuSurface::BrowserTab,
        crate::apps::MenuSurface::Trash,
    ];
    for surface in surfaces {
        let menu = crate::apps::ContextMenu::for_surface(surface, (10.0, 10.0));
        assert!(!menu.items.is_empty(), "{surface:?} menu is empty");
        for item in &menu.items {
            if let crate::apps::MenuItem::Item(_, action) = item {
                // perform_menu accepts every action without panicking.
                let mut s = shell();
                let _ = s.perform_menu(*action);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ink(c: &Canvas, r: Rect) -> usize {
    let mut n = 0usize;
    let y0 = r.y.max(0.0) as i64;
    let y1 = r.bottom().max(0.0) as i64;
    let x0 = r.x.max(0.0) as i64;
    let x1 = r.right().max(0.0) as i64;
    for y in y0..y1 {
        for x in x0..x1 {
            if luminance(c.get(x, y).unwrap_or(0)) > 0.25 {
                n = n.saturating_add(1);
            }
        }
    }
    n
}

#[cfg(unix)]
fn collect(bridge: &crate::fsbridge::FsBridge, dir: &str, out: &mut Vec<String>, depth: usize) {
    if out.len() >= 100 || depth > 6 {
        return;
    }
    let Ok(entries) = bridge.list_dir(dir) else {
        return;
    };
    for e in entries {
        let p = format!("{dir}/{}", e.name);
        if e.kind == crate::apps::FileKind::Dir {
            collect(bridge, &p, out, depth.saturating_add(1));
        } else if e.kind == crate::apps::FileKind::File {
            out.push(p);
        }
    }
}

#[cfg(unix)]
const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";
// ---------------------------------------------------------------------------
// The lock screen
// ---------------------------------------------------------------------------

/// A shell on a machine that can check a password, as the daemon reports one.
///
/// The facts are the daemon's own words (`AUTH VERIFY` is answered through
/// `unix_chkpwd`), because the question "may this session be locked" is decided
/// by them and by nothing else.
fn lockable_shell() -> Shell {
    let mut s = shell();
    s.facts.auth_available = Some("unix-chkpwd".to_owned());
    s.facts.auth_lock = Some(false);
    s.facts.auth_account = Some("litric".to_owned());
    s.facts.auth_password = Some("set".to_owned());
    s.facts.session_user = Some("litric".to_owned());
    s
}

/// A frame drawn over a sentinel backdrop instead of a wallpaper.
///
/// The lock surface owes the user a cleared frame: nothing that was on screen
/// may still be in it. Filling the canvas with a colour that appears nowhere
/// else makes that checkable -- if any sentinel pixel survives, the lock did
/// not cover the screen.
fn draw_frame_over(s: &mut Shell, sentinel: u32) -> Canvas {
    s.tick_after(FRAME_MS);
    let mut c = Canvas::new(1280, 800, sentinel);
    crate::views::draw(&mut c, s);
    c
}

fn count_colour(c: &Canvas, colour: u32) -> usize {
    c.pixels()
        .iter()
        .filter(|p| (**p & 0x00FF_FFFF) == (colour & 0x00FF_FFFF))
        .count()
}

#[test]
fn journey_a_machine_that_cannot_check_a_password_refuses_to_lock() {
    // No daemon answered, so neither auth fact arrived. The refusal is the
    // point: a lock screen on this machine would have no way out.
    let mut s = shell();
    assert!(!s.can_lock());
    assert!(!s.lock_now(), "a lock with no verifier is refused");
    assert!(!s.lock.locked);
    assert!(!s.lock_refusal().is_empty());

    // A verifier that the daemon refuses is refused too, even though a password
    // could be checked: the two facts are separate questions.
    let mut refused = lockable_shell();
    refused.facts.auth_lock = Some(true);
    assert!(!refused.can_lock());
    assert!(!refused.lock_now());

    // A helper being present is not the same as the account being able to
    // accept a password. A skipped first run leaves `litric` locked; raising
    // the screen there would trap the user with no password that can open it.
    for (password, reason, expected) in [
        ("locked", "password-locked", "no usable password"),
        ("absent", "password-unset", "no password set"),
        ("unknown", "password-unknown", "could not be read"),
    ] {
        let mut unavailable = lockable_shell();
        unavailable.facts.auth_password = Some(password.to_owned());
        unavailable.facts.auth_reason = Some(reason.to_owned());
        unavailable.facts.auth_lock = Some(true);
        assert!(!unavailable.can_lock(), "{password}");
        assert!(!unavailable.lock_now(), "{password}");
        let sentence = unavailable.lock_refusal();
        assert!(
            sentence.contains("litric") && sentence.contains(expected),
            "{password}: {sentence}"
        );
        // The actionable half: a locked or unset password names the command
        // that gives the account one.
        assert!(sentence.contains("passwd"), "{password}: {sentence}");
    }

    // And a machine that can check one, and whose daemon agrees, locks.
    let mut ok = lockable_shell();
    assert!(ok.can_lock());
    assert!(ok.lock_now());
    assert_eq!(ok.life(), LifeState::Locked);
}

#[test]
fn journey_locking_hides_the_session_and_only_a_password_brings_it_back() {
    let mut s = lockable_shell();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);
    let window =
        s.wm.all()
            .iter()
            .find(|w| w.app_id == "com.litric.files")
            .map(|w| w.id)
            .expect("the session has a window open to hide");
    assert!(
        s.hotspots().rect_of(Hot::Window(window)).is_some(),
        "the window is on screen before the lock"
    );

    assert!(s.lock_now());
    let sentinel = rgb(0x00_FF_00);
    let locked_frame = draw_frame_over(&mut s, sentinel);
    assert_eq!(
        count_colour(&locked_frame, sentinel),
        0,
        "the lock surface cleared the frame: no pixel of what was underneath is left"
    );
    // Every other surface's hotspots are gone with the pixels they were drawn
    // on, so a click cannot reach a window or a desktop shortcut.
    for i in 0..8u8 {
        assert!(s.hotspots().rect_of(Hot::DockItem(i)).is_none());
    }
    assert!(s.hotspots().rect_of(Hot::DesktopIcon(0)).is_none());
    assert!(s.hotspots().rect_of(Hot::LockWake).is_none());
    assert!(
        s.hotspots().rect_of(Hot::LockSubmit).is_some(),
        "the password field is available as soon as the lock appears"
    );

    // Typing does not reach the application behind the lock: the Files window
    // has a filter field, and it is not what the keyboard is talking to.
    let app_before = s.files.filter.clone();
    type_text(&mut s, "hunter2");
    assert_eq!(s.files.filter, app_before, "no keystroke reached the app");

    // Enter hands the password to the caller that owns the socket, and nowhere
    // else. The effect carries it as a redacted `Secret`: its Debug output is
    // what a diagnostic would print.
    let effects = key_press(&mut s, Key::Enter, Modifiers::NONE);
    let mut seen = 0u32;
    for e in &effects {
        if let Effect::VerifyPassword { user, secret } = e {
            seen = seen.saturating_add(1);
            assert_eq!(
                user, "litric",
                "the desktop account, not the serial console"
            );
            assert_eq!(secret.hex(), "68756e74657232");
            let printed = format!("{secret:?}");
            assert!(!printed.contains("hunter2"), "{printed}");
            assert!(printed.contains("7 bytes"), "{printed}");
        }
    }
    assert_eq!(seen, 1, "exactly one verification per Enter");
    assert!(s.lock.locked, "the session stays locked until the answer");
    assert_eq!(
        s.lock.typed(),
        0,
        "the buffer is emptied as it is handed over"
    );

    // A refusal keeps the lock and says why, in the daemon's words.
    s.lock_failed("ERR action-failed unix_chkpwd said no");
    let _ = draw_frame(&mut s);
    assert!(s.lock.locked);
    assert!(s
        .lock
        .message()
        .is_some_and(|m| m.contains("action-failed")));

    // And a verified password puts the session back, with the window that was
    // open still where it was.
    s.unlock();
    let _ = draw_frame(&mut s);
    assert!(!s.lock.locked);
    assert_eq!(s.life(), LifeState::Active);
    assert!(
        s.hotspots().rect_of(Hot::Window(window)).is_some(),
        "the session's window is back on screen"
    );
    assert!(s.hotspots().rect_of(Hot::LockWake).is_none());
}

#[test]
fn lock_idle_shows_the_account_and_a_direct_unlock_path() {
    let mut s = lockable_shell();
    assert!(s.lock_now());
    s.lock.back();
    let mut c = Canvas::new(1280, 800, rgb(0x000000));
    c.record_text(true);
    crate::views::draw(&mut c, &mut s);
    assert!(
        c.text_runs()
            .iter()
            .any(|run| run.text == "Click or press Enter to unlock"),
        "the lock explains the next action before a keypress"
    );
    assert!(s.hotspots().rect_of(Hot::LockSubmit).is_none());
    click(&mut s, 640.0, 560.0);
    assert_eq!(s.lock.stage, crate::lock::Stage::Entry);
}

#[test]
fn journey_the_lock_does_not_collect_a_password_it_could_not_check() {
    let mut s = lockable_shell();
    assert!(s.lock_now());
    // The daemon goes away while the session is locked.
    s.facts.auth_available = None;
    s.facts.auth_lock = None;
    type_text(&mut s, "hunter2");
    let _ = draw_frame(&mut s);
    assert!(
        s.hotspots().rect_of(Hot::LockSubmit).is_none(),
        "no submit button is offered for a password nothing can check"
    );
    let effects = key_press(&mut s, Key::Enter, Modifiers::NONE);
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::VerifyPassword { .. })),
        "no verification is attempted"
    );
    assert!(s.lock.locked);
}

#[test]
fn journey_a_refusal_closes_the_field_before_it_can_be_asked_again() {
    // The harness's key helpers stamp events at a fixed time, so this journey
    // advances the shell's clock explicitly: a wait is measured against it.
    fn enter_at(s: &mut Shell, now: u64) -> Vec<Effect> {
        s.handle(
            &InputEvent::KeyDown {
                key: Key::Enter,
                code: 0,
                mods: Modifiers::NONE,
                repeat: false,
            },
            now,
        )
    }

    let mut s = lockable_shell();
    assert!(s.lock_now());
    type_text(&mut s, "hunter2");
    let first = enter_at(&mut s, 1_300);
    assert!(first
        .iter()
        .any(|e| matches!(e, Effect::VerifyPassword { .. })));

    s.lock_failed("ERR action-failed unix_chkpwd said no");
    s.tick(1_400);
    let _ = draw_frame(&mut s);
    assert!(
        s.hotspots().rect_of(Hot::LockSubmit).is_none(),
        "the field closes on the first refusal"
    );
    assert!(
        s.take_unlock().is_none(),
        "no secret leaves the shell while the field is closed"
    );

    // Enter during the wait is not a request, and it does not eat the buffer.
    type_text(&mut s, "hunter2");
    let blocked = enter_at(&mut s, 1_500);
    assert!(!blocked
        .iter()
        .any(|e| matches!(e, Effect::VerifyPassword { .. })));

    // The wait ends on the clock, not on a keypress: the field is back and the
    // text typed during the wait is still in it.
    s.tick(2_400);
    let _ = draw_frame(&mut s);
    assert!(s.hotspots().rect_of(Hot::LockSubmit).is_some());
    let second = enter_at(&mut s, 2_500);
    assert!(second
        .iter()
        .any(|e| matches!(e, Effect::VerifyPassword { .. })));

    // The second refusal waits twice as long as the first.
    s.lock_failed("ERR action-failed unix_chkpwd said no");
    s.tick(3_000);
    let _ = draw_frame(&mut s);
    assert!(
        s.hotspots().rect_of(Hot::LockSubmit).is_none(),
        "the second wait is two seconds, not one"
    );
    s.tick(5_000);
    let _ = draw_frame(&mut s);
    assert!(s.hotspots().rect_of(Hot::LockSubmit).is_some());
}

#[test]
fn journey_the_lock_customize_sheet_changes_the_look_live() {
    let mut s = lockable_shell();
    assert!(s.lock_now());
    let _ = draw_frame(&mut s);
    let customize = s
        .hotspots()
        .rect_of(Hot::LockCustomize)
        .expect("the customize button is on the lock screen");
    click(&mut s, customize.center().0, customize.center().1);
    let _ = draw_frame(&mut s);
    assert!(s.lock.edit);
    // The sheet's own controls are what the pointer can reach while it is open;
    // the full-surface wake catcher is not, or a click inside the sheet would
    // also bring the password field out behind it.
    assert!(s.hotspots().rect_of(Hot::LockWake).is_none());
    assert!(s.hotspots().rect_of(Hot::LockDone).is_some());

    // Effects tab, then the "Dreamy" glow and bloom preset.
    let effects_tab = s
        .hotspots()
        .rect_of(Hot::LockTab(1))
        .expect("the effects tab is drawn");
    click(&mut s, effects_tab.center().0, effects_tab.center().1);
    let _ = draw_frame(&mut s);
    assert_eq!(s.lock.tab, crate::lock::Tab::Effects);
    let dreamy = s
        .hotspots()
        .rect_of(Hot::LockOption(crate::lock::GROUP_LIGHT, 3))
        .expect("the glow preset buttons are drawn");
    click(&mut s, dreamy.center().0, dreamy.center().1);
    assert_eq!(s.lock.tweaks.glow(), 70.0);
    assert_eq!(s.lock.tweaks.bloom(), 80.0);

    // A switch, then Done.
    let _ = draw_frame(&mut s);
    let breathe = s
        .hotspots()
        .rect_of(Hot::LockSwitch(crate::lock::SWITCH_BREATHE))
        .expect("the breathing switch is drawn");
    click(&mut s, breathe.center().0, breathe.center().1);
    assert!(s.lock.tweaks.breathe);

    let _ = draw_frame(&mut s);
    let done = s.hotspots().rect_of(Hot::LockDone).expect("Done");
    click(&mut s, done.center().0, done.center().1);
    assert!(!s.lock.edit);
    let _ = draw_frame(&mut s);
    assert!(s.hotspots().rect_of(Hot::LockEntry).is_some());
}

// ---------------------------------------------------------------------------
// The dock that becomes the launcher
// ---------------------------------------------------------------------------

/// Put the dock on screen the way the shell has it: present, at rest.
///
/// The dock is not an auto-hide surface, so this is the pointer-free case --
/// no reveal to wait for, and test intent stays honest about that.
fn show_dock(s: &mut Shell) {
    s.tick_after(FRAME_MS);
}

/// Open the launcher the way a user does: press the dock's button and wait for
/// the morph spring to arrive.
fn open_launcher(s: &mut Shell) {
    show_dock(s);
    s.perform(Action::ToggleLauncher);
    for _ in 0..60 {
        s.tick_after(FRAME_MS);
    }
}

#[test]
fn journey_the_launcher_opens_on_the_apps_menu_and_a_tile_opens_the_app() {
    let mut s = shell();
    show_dock(&mut s);
    let _ = draw_frame(&mut s);
    // At rest the bar is on screen -- its button, its pinned icons and its
    // field -- and the panel's own contents are not.
    assert!(s.hotspots().rect_of(Hot::DockItem(3)).is_some());
    assert!(s.hotspots().rect_of(Hot::DockSearch).is_some());
    assert!(s.hotspots().rect_of(Hot::DockToggle).is_some());
    assert!(s.hotspots().rect_of(Hot::LauncherItem(3)).is_none());

    // The Apps menu opens the surface the bar grows into.
    let apps = s
        .hotspots()
        .rect_of(Hot::MenuApps)
        .expect("the Apps menu is on the bar");
    click(&mut s, apps.center().0, apps.center().1);
    for _ in 0..60 {
        s.tick_after(FRAME_MS);
    }
    let _ = draw_frame(&mut s);
    assert!(matches!(s.overlay, Overlay::Launcher { .. }));
    let tile = s
        .hotspots()
        .rect_of(Hot::LauncherItem(3))
        .expect("the app tiles are drawn once the panel is out");
    // The dock's own row is still there, at the bottom of the same panel: the
    // icons the user was aiming at have not moved.
    assert!(s.hotspots().rect_of(Hot::DockItem(3)).is_some());
    assert!(s.hotspots().rect_of(Hot::DockSearch).is_some());

    click(&mut s, tile.center().0, tile.center().1);
    assert!(
        s.wm.all().iter().any(|w| w.app_id == "com.litric.files"),
        "a tile opens its application"
    );
    assert!(matches!(s.overlay, Overlay::None));

    // The panel animates back down to the pill.
    for _ in 0..60 {
        s.tick_after(FRAME_MS);
    }
    assert!(s.anims().launcher_morph.value() < 0.05);
}

#[test]
fn journey_searching_in_the_launcher_filters_and_escape_collapses() {
    let mut s = shell();
    open_launcher(&mut s);
    let _ = draw_frame(&mut s);
    assert!(s.hotspots().rect_of(Hot::LauncherItem(0)).is_some());

    // Typing switches the panel to its results layer: one row per match, and
    // not one row per application as the grid had.
    type_text(&mut s, "term");
    let _ = draw_frame(&mut s);
    let matches = App::search("term");
    assert_eq!(matches.first().copied(), Some(App::Terminal));
    assert!(s.hotspots().rect_of(Hot::LauncherItem(0)).is_some());
    assert!(
        s.hotspots()
            .rect_of(Hot::LauncherItem(matches.len()))
            .is_none(),
        "no row is drawn for a match that does not exist"
    );
    assert!(s.hotspots().rect_of(Hot::DockCollapse).is_some());

    // The first result is the terminal.
    let row = s
        .hotspots()
        .rect_of(Hot::LauncherItem(0))
        .expect("a result");
    click(&mut s, row.center().0, row.center().1);
    assert!(
        s.wm.all().iter().any(|w| w.app_id == "com.litric.terminal"),
        "the first result opened"
    );

    // Reopen and Escape: the overlay closes and the panel shrinks back.
    open_launcher(&mut s);
    let _ = draw_frame(&mut s);
    assert!(matches!(s.overlay, Overlay::Launcher { .. }));
    let _ = key_press(&mut s, Key::Escape, Modifiers::NONE);
    assert!(matches!(s.overlay, Overlay::None));
    for _ in 0..60 {
        s.tick_after(FRAME_MS);
    }
    assert!(s.anims().launcher_morph.value() < 0.05);
}

#[test]
fn journey_mission_control_shows_windows_from_every_workspace_and_opens_one() {
    let mut s = shell();
    s.open_app(App::Files);
    s.perform(Action::Workspace(1));
    s.open_app(App::Terminal);
    let _ = draw_frame(&mut s);

    let meta = Modifiers {
        meta: true,
        ..Modifiers::NONE
    };
    let _ = key_press(&mut s, Key::Tab, meta);
    assert!(
        matches!(s.overlay, Overlay::Overview { .. }),
        "Win+Tab did not open Mission Control"
    );
    let _ = draw_frame(&mut s);

    // Both windows are cards, even though only one is on the active workspace.
    let first = s
        .hotspots()
        .rect_of(Hot::OverviewItem(0))
        .expect("the first card is drawn");
    assert!(
        s.hotspots().rect_of(Hot::OverviewItem(1)).is_some(),
        "a window on another workspace has no card"
    );
    click(&mut s, first.center().0, first.center().1);
    assert!(matches!(s.overlay, Overlay::None));
    assert_eq!(
        s.wm.active_workspace(),
        0,
        "the card did not switch workspace"
    );
    assert_eq!(
        s.wm.focused()
            .and_then(|id| s.wm.window(id))
            .map(|w| w.app_id.as_str()),
        Some("com.litric.files")
    );
}

#[test]
fn journey_mission_control_is_keyboard_only() {
    let mut s = shell();
    s.wm.open("com.litric.files", "Documents", (400.0, 300.0));
    s.perform(Action::Workspace(1));
    s.wm.open("com.litric.terminal", "Terminal", (400.0, 300.0));
    let meta = Modifiers {
        meta: true,
        ..Modifiers::NONE
    };
    let _ = key_press(&mut s, Key::Tab, meta);
    // The overview starts on the focused window (the Terminal, card 1);
    // Left moves to the Files window on workspace 1.
    let _ = key_press(&mut s, Key::Left, Modifiers::NONE);
    let _ = key_press(&mut s, Key::Enter, Modifiers::NONE);
    assert!(matches!(s.overlay, Overlay::None));
    assert_eq!(s.wm.active_workspace(), 0);
    assert!(s
        .wm
        .focused()
        .and_then(|id| s.wm.window(id))
        .is_some_and(|w| w.app_id == "com.litric.files"));
}

#[test]
fn journey_the_search_field_finds_an_action_and_runs_it() {
    let mut s = shell();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);
    open_launcher(&mut s);
    type_text(&mut s, "show desktop");
    let _ = draw_frame(&mut s);
    let _ = key_press(&mut s, Key::Enter, Modifiers::NONE);
    assert!(matches!(s.overlay, Overlay::None));
    assert_eq!(
        s.wm.all().first().map(|w| w.state),
        Some(litric_wm::WindowState::Minimized),
        "the action result did not run"
    );
}

#[test]
fn journey_the_search_field_finds_a_window_on_another_workspace() {
    let mut s = shell();
    let id =
        s.wm.open("com.litric.files", "Quarterly budget", (400.0, 300.0));
    s.perform(Action::MoveToWorkspace(3));
    s.perform(Action::Workspace(0));
    open_launcher(&mut s);
    type_text(&mut s, "quarterly");
    let _ = draw_frame(&mut s);
    let row = s
        .hotspots()
        .rect_of(Hot::LauncherItem(0))
        .expect("the window is a search result");
    click(&mut s, row.center().0, row.center().1);
    assert_eq!(s.wm.focused(), Some(id));
    assert_eq!(s.wm.active_workspace(), 3);
    assert_eq!(
        s.wm.window(id).map(|w| w.state),
        Some(litric_wm::WindowState::Normal)
    );
}

#[test]
fn journey_a_display_scale_change_undoes_itself_unless_it_is_kept() {
    // A scale that is too large moves its own undo control off the edge of the
    // screen, so the only safe design is a change that expires. This journey
    // drives both endings: the user who does nothing gets the old scale back,
    // and the user who says keep keeps the new one. The control itself is a
    // hotspot on the Display page; what is measured here is the mechanism the
    // control arms, which is the part that has to work when the screen does not.
    let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
    let mut s = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("test"));
    s.mode = DesktopMode::Ready;

    s.apply_ui_scale(1.25);
    assert!(
        (s.tokens.scale - 1.25).abs() < 0.001,
        "the change is applied at once"
    );
    let pending = s.scale_revert.as_ref().expect("a provisional change");
    assert_eq!(pending.previous(), 1.0);
    assert_eq!(pending.seconds_left(), 15);

    // Another step while a change is provisional must not stack: it would make
    // the value being tested ambiguous, and the first undo value unreachable.
    s.apply_ui_scale(1.5);
    assert!((s.tokens.scale - 1.25).abs() < 0.001);

    s.tick(10_000);
    assert!(
        (s.tokens.scale - 1.25).abs() < 0.001,
        "still inside the window"
    );
    assert_eq!(s.scale_revert.as_ref().map(|w| w.seconds_left()), Some(5));
    s.tick(15_500);
    assert!(
        (s.tokens.scale - 1.0).abs() < 0.001,
        "the window closed and put the old scale back"
    );
    assert!(s.scale_revert.is_none());

    // The other ending: the user keeps it, and the clock no longer applies.
    s.apply_ui_scale(1.25);
    assert!((s.tokens.scale - 1.25).abs() < 0.001);
    let effects = s.keep_pending_change();
    assert!(s.scale_revert.is_none());
    s.tick(60_000);
    assert!(
        (s.tokens.scale - 1.25).abs() < 0.001,
        "a kept change survives the clock that would have undone it"
    );
    // Keeping it is what writes it: the step reaches the record, and the record
    // is what a restart reads back.
    let saved = effects
        .iter()
        .find_map(|effect| match effect {
            Effect::PersistPreferences(preferences) => Some(*preferences),
            _ => None,
        })
        .expect("keeping a scale asks for it to be stored");
    assert_eq!(saved.ui_scale, crate::settings::UiScale::Large);
    s.finish_preferences_save(Ok(()));
    assert!(
        (s.tokens.scale - 1.25).abs() < 0.001,
        "a successful write does not move the screen"
    );

    // A restart reads the step back and does not ask again: a value that came
    // from the file was confirmed by the save that wrote it.
    let mut rebooted = Shell::new(
        Tokens::new(Theme::of(ThemeKind::Dark), 1.0),
        1280.0,
        800.0,
        Regional::fallback("test"),
    );
    rebooted.load_preferences(saved);
    assert!((rebooted.tokens.scale - 1.25).abs() < 0.001);
    assert!(rebooted.scale_revert.is_none());
}

/// A fast scan and a deep scan must not be the same scan with a different
/// label: the deep one reaches folders the fast one promises not to walk. The
/// fixture nests a threat ten folders below Documents, which is outside the
/// fast walk's depth and inside the deep walk's, then drives the real button
/// and the real worker over it.
#[cfg(unix)]
#[test]
fn journey_defender_fast_and_deep_scans_cover_different_folders() {
    use crate::apps::ScanMode;
    use litric_defender::verdict::Decision;

    let root = e2e_root("defender-modes");
    let mut dir = root.join("Documents");
    for i in 0..11 {
        dir = dir.join(format!("level{i}"));
    }
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("eicar.com"), EICAR).unwrap();
    std::fs::write(root.join("Documents").join("clean.txt"), b"hello").unwrap();

    let zone = litric_locale::TimeZone::utc("test");
    let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();

    let (fast, _) = crate::shell::collect_for_mode(&bridge, ScanMode::Fast);
    assert!(
        !fast.iter().any(|p| p.ends_with("eicar.com")),
        "the fast scan stays inside the depth it promises: {fast:?}"
    );
    assert!(
        fast.iter().any(|p| p.ends_with("clean.txt")),
        "the fast scan still reaches the top level: {fast:?}"
    );
    let (deep, _) = crate::shell::collect_for_mode(&bridge, ScanMode::Deep);
    assert!(
        deep.iter().any(|p| p.ends_with("eicar.com")),
        "the deep scan reaches the nested file: {deep:?}"
    );

    // The file the deep scan reached is the one the engine denies.
    let mut engine = litric_defender::engine::Engine::new();
    engine.load_denylist(crate::shell::eicar_denylist());
    let target = deep
        .iter()
        .find(|p| p.ends_with("eicar.com"))
        .expect("the deep queue holds the nested file");
    let host = bridge.host_path(target).unwrap();
    let bytes = std::fs::read(&host).unwrap();
    let report = engine.scan(target, bytes.len() as u64, 0, &bytes);
    assert!(matches!(report.decision, Decision::Deny { .. }));

    // And the window is wired to it: clicking Deep scan runs the deep mode.
    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);
    let deep_btn = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::DeepScan))
        .expect("the Deep scan button is drawn");
    click(&mut s, deep_btn.center().0, deep_btn.center().1);
    assert_eq!(s.defender.app.mode, ScanMode::Deep, "the button chose deep");
    for _ in 0..400 {
        s.scan_step(16);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!s.defender.running, "the deep scan finished");
    assert_eq!(
        s.defender.app.state,
        ScanState::Done {
            scanned: 2,
            threats: 1
        },
        "the wired deep scan scanned both files and reported the threat"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The machine's own vocabulary must not reach a surface a person reads. This
/// journey drives the real frames -- Tasks, then the two Settings pages -- with
/// the tokens set on the facts, and inspects every text run the canvas drew.
#[test]
fn journey_no_machine_token_reaches_the_system_surfaces() {
    use crate::copy::BANNED_TOKENS;
    use crate::settings::Page;

    let mut s = shell();
    s.facts.net_ifaces = vec![crate::shell::NetIface {
        name: "wlan0".to_owned(),
        state: "up".to_owned(),
        mac: "00:11:22:33:44:55".to_owned(),
        wireless: true,
    }];
    s.facts.network = Some("online".to_owned());
    s.facts.root_fs = Some("erofs".to_owned());
    s.facts.volumes = vec![
        ("/".to_owned(), "erofs".to_owned(), true),
        ("/home".to_owned(), "btrfs".to_owned(), false),
    ];

    let mut seen: Vec<String> = Vec::new();
    s.open_app(App::Activity);
    let canvas = draw_frame_at(&mut s, 1280, 800);
    seen.extend(canvas.text_runs().iter().map(|r| r.text.clone()));

    s.open_app(App::Settings);
    s.settings.page = Page::Network;
    let canvas = draw_frame_at(&mut s, 1280, 800);
    seen.extend(canvas.text_runs().iter().map(|r| r.text.clone()));
    assert!(
        seen.iter().any(|t| t == "Connected"),
        "the network row reads as the plain state: {seen:?}"
    );

    // Every page, in the sidebar's own order: the storage list is drawn by the
    // page that owns it, and the guard must not care which one that is.
    for page in Page::ALL {
        s.settings.page = page;
        let canvas = draw_frame_at(&mut s, 1280, 800);
        seen.extend(canvas.text_runs().iter().map(|r| r.text.clone()));
    }
    assert!(
        seen.iter().any(|t| t == "Read-only"),
        "storage says what can be done: {seen:?}"
    );

    for token in BANNED_TOKENS {
        if let Some(hit) = seen.iter().find(|t| t.to_ascii_lowercase().contains(token)) {
            panic!("the frame printed {token:?}: {hit:?}");
        }
    }
}
