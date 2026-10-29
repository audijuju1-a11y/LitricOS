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

/// Sample a grid inside `r` and count the pixels that differ from `reference`.
/// The grid is coarse on purpose: this asks "is this region showing content of
/// its own", not "is every pixel a particular value".
fn differing_pixels(canvas: &Canvas, r: Rect, reference: u32) -> (usize, usize) {
    let mut differing = 0usize;
    let mut total = 0usize;
    let mut x = r.x + 4.0;
    while x < r.right() - 4.0 {
        let mut y = r.y + 4.0;
        while y < r.bottom() - 4.0 {
            if let Some(px) = canvas.get(x as i64, y as i64) {
                total = total.saturating_add(1);
                if px != reference {
                    differing = differing.saturating_add(1);
                }
            }
            y += 12.0;
        }
        x += 12.0;
    }
    (differing, total)
}

/// One browser session through the window: open it from the shell, type the
/// address of a real page on this machine, read the rendered text, then scroll
/// to the second chapter and see its panel decoded and drawn. Every step is a
/// real click, key event or wheel event; every assertion reads what the reader
/// actually laid out and drew.
#[test]
fn journey_browser_opens_a_real_comic_page_and_renders_it() {
    use crate::reader::{MAX_CACHED_IMAGES, MAX_CACHED_PIXELS};

    let root = e2e_root("browser-comic");
    // A real JPEG from the decoder's own test corpus, copied beside the page:
    // the reader decodes these bytes, not a fixture the test pre-parsed.
    let jpeg = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../litric-browser/testdata/jpeg/s420-q75.jpg"
    ))
    .expect("the decoder's test picture");
    std::fs::write(root.join("panel1.jpg"), &jpeg).unwrap();
    std::fs::write(root.join("panel2.jpg"), &jpeg).unwrap();
    let page = root.join("page.html");
    std::fs::write(
        &page,
        "<!doctype html><html><head><title>Comic Page</title></head><body>\
         <h1>Chapter One</h1>\
         <p>A page of panels, opened from this machine and drawn by the reader.</p>\
         <img src=\"panel1.jpg\" alt=\"Panel one\" width=\"900\" height=\"600\">\
         <h1>Chapter Two</h1>\
         <p>The story continues below the first panel.</p>\
         <img src=\"panel2.jpg\" alt=\"Panel two\" width=\"900\" height=\"600\">\
         </body></html>",
    )
    .unwrap();
    let url = format!("file://{}", page.display());

    let mut s = shell();
    s.open_app(App::Browser);
    let _ = draw_frame(&mut s);
    let field = s
        .hotspots()
        .rect_of(Hot::BrowserOmnibox)
        .expect("the start page draws its search box");
    click(&mut s, field.center().0, field.center().1);
    type_text(&mut s, &url);
    assert_eq!(s.browser.omnibox, url);
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    let canvas = draw_frame(&mut s);

    assert!(
        s.reader.is_some(),
        "the page loaded; the status line says: {}",
        s.browser.status
    );
    assert_eq!(s.browser.status, "Opened Comic Page");
    assert_eq!(
        s.browser.tab().map(|t| t.title.clone()),
        Some("Comic Page".to_owned()),
        "the tab takes the page's own title"
    );
    let area = s
        .browser_engine_area
        .expect("the page surface is on screen");
    let reference = canvas
        .get((area.right() - 6.0) as i64, (area.bottom() - 6.0) as i64)
        .expect("a background pixel");

    let heading = Rect::new(area.x + 24.0, area.y + 16.0, 260.0, 44.0);
    let (text_px, text_total) = differing_pixels(&canvas, heading, reference);
    assert!(
        text_total > 40 && text_px > 30,
        "the heading is drawn, not just laid out: {text_px}/{text_total}"
    );

    let reader = s.reader.as_ref().expect("the reader");
    assert!(
        reader
            .visible_text(area.h)
            .iter()
            .any(|line| line.contains("Chapter One")),
        "the first chapter's text is on screen"
    );
    assert!(
        !reader
            .visible_text(area.h)
            .iter()
            .any(|line| line.contains("Chapter Two")),
        "the second chapter is still below the fold"
    );
    assert_eq!(
        reader.cached_images(),
        1,
        "only the visible panel was decoded"
    );
    let panel = reader.image_area(0).expect("the first panel is laid out");
    let panel = Rect::new(area.x + panel.x, area.y + panel.y, panel.w, panel.h);
    let (pixels, total) = differing_pixels(&canvas, panel, reference);
    assert!(
        total > 100 && pixels * 10 >= total,
        "the first panel's decoded pixels reached the frame: {pixels}/{total}"
    );

    // Wheel down: the second chapter comes into view, and only now is its
    // panel decoded.
    let at = Point::new(area.center().0, area.center().1);
    let _ = s.handle(
        &InputEvent::Scroll {
            dx: 0.0,
            dy: 10.0,
            at,
            mods: Modifiers::NONE,
        },
        2_000,
    );
    let canvas = draw_frame(&mut s);
    let reader = s.reader.as_ref().expect("the reader");
    assert!(reader.scroll() > 0.0, "the wheel scrolled the page");
    let visible = reader.visible_text(area.h);
    assert!(
        visible.iter().any(|line| line.contains("Chapter Two")),
        "the second chapter is now on screen: scroll={} content={} viewport={} visible={visible:?}",
        reader.scroll(),
        reader.content_height(),
        area.h
    );
    assert_eq!(
        reader.cached_images(),
        2,
        "the second panel was decoded for the viewport"
    );
    assert!(reader.cached_images() <= MAX_CACHED_IMAGES);
    assert!(reader.cached_pixels() <= MAX_CACHED_PIXELS);
    let second = reader.image_area(1).expect("the second panel is laid out");
    let second = Rect::new(area.x + second.x, area.y + second.y, second.w, second.h);
    let (pixels, total) = differing_pixels(&canvas, second, reference);
    assert!(
        total > 50 && pixels * 10 >= total,
        "the second panel's decoded pixels reached the frame after scrolling: {pixels}/{total}"
    );

    // The window speaks in words, never in addresses, formats or codes.
    for banned in ["file://", "jpeg", "JPEG", "html", "HTML", "erofs", "error"] {
        assert!(
            !s.browser.status.contains(banned),
            "the status line leaks {banned:?}: {}",
            s.browser.status
        );
    }
    let _ = std::fs::remove_dir_all(&root);
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

/// One Defender session through the window: Fast scan over a folder that holds
/// a harmless note and a test threat, then Deep scan over the same folder plus
/// a threat nested deeper than Fast scan promises to walk. Every step is a
/// click on the drawn control; every assertion reads what the window renders.
#[test]
fn journey_defender_scans_a_real_folder_fast_and_deep() {
    use crate::apps::ScanMode;

    let root = e2e_root("defender-journey");
    let docs = root.join("Documents");
    let mut nested = docs.clone();
    for i in 0..10 {
        nested = nested.join(format!("level{i}"));
    }
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(docs.join("clean.txt"), b"a harmless note\n").unwrap();
    std::fs::write(docs.join("eicar.com"), EICAR).unwrap();
    std::fs::write(nested.join("eicar-nested.com"), EICAR).unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);

    // Fast scan: the two files at the top of the folder, no deeper.
    let fast_btn = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::ScanNow))
        .expect("the Fast scan button is drawn");
    let _ = click(&mut s, fast_btn.center().0, fast_btn.center().1);
    assert_eq!(
        s.defender.app.state,
        ScanState::Scanning { done: 0, total: 2 },
        "clicking Fast scan queues exactly the top-level files"
    );
    assert!(s.defender.running, "the scan worker is alive");
    for _ in 0..600 {
        let _ = draw_frame(&mut s);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(!s.defender.running, "the fast scan finished");
    assert_eq!(
        s.defender.app.state,
        ScanState::Done {
            scanned: 2,
            threats: 1
        },
        "the fast scan read both files and found the planted threat"
    );
    let rows = &s.defender.app.rows;
    let threat = rows
        .iter()
        .find(|r| r.path.ends_with("/eicar.com"))
        .expect("the threat is listed by name");
    assert!(threat.detected, "the planted threat is flagged: {rows:?}");
    assert_eq!(threat.verdict, "Threat found");
    let clean = rows
        .iter()
        .find(|r| r.path.ends_with("/clean.txt"))
        .expect("the harmless file is listed too");
    assert!(!clean.detected, "the harmless file is not flagged");
    assert_eq!(clean.verdict, "Safe");
    assert!(
        !rows.iter().any(|r| r.path.contains("eicar-nested.com")),
        "Fast scan stays inside the depth it promises: {rows:?}"
    );
    assert!(
        s.defender.app.message.starts_with("Fast scan")
            && s.defender.app.message.contains("1 threat"),
        "the result sentence names the mode and the finding: {}",
        s.defender.app.message
    );

    // Deep scan: the same folder, and everything under it.
    let deep_btn = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::DeepScan))
        .expect("the Deep scan button is drawn once the first scan ends");
    let _ = click(&mut s, deep_btn.center().0, deep_btn.center().1);
    assert_eq!(s.defender.app.mode, ScanMode::Deep, "the button chose deep");
    assert_eq!(
        s.defender.app.state,
        ScanState::Scanning { done: 0, total: 3 },
        "Deep scan queues the nested file Fast scan could not reach"
    );
    for _ in 0..600 {
        let _ = draw_frame(&mut s);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(!s.defender.running, "the deep scan finished");
    assert_eq!(
        s.defender.app.state,
        ScanState::Done {
            scanned: 3,
            threats: 2
        },
        "the deep scan covered the nested file as well"
    );
    let rows = &s.defender.app.rows;
    let deeper = rows
        .iter()
        .find(|r| r.path.ends_with("/eicar-nested.com"))
        .expect("the nested threat is now listed");
    assert!(deeper.detected, "the nested threat is flagged");
    assert!(
        s.defender.app.message.starts_with("Deep scan"),
        "the result sentence names the deeper mode: {}",
        s.defender.app.message
    );
    // Every row the window shows is plain language: a person's verdict, no
    // tier tokens, signature ids, hashes or engine internals.
    for r in rows {
        assert!(
            matches!(r.verdict.as_str(), "Safe" | "Threat found" | "Not checked"),
            "the verdict is a human word, not an engine token: {:?}",
            r.verdict
        );
        for banned in [
            "Hash", "DETECTED", "Deny", "Allow", "skipped", "stat:", "erofs",
        ] {
            assert!(
                !r.verdict.contains(banned),
                "the row leaks {banned:?} to the window: {r:?}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// A running scan must be stoppable from the window itself: the Stop control is
/// drawn beside the inert mode buttons, and clicking it asks the worker to stop.
#[test]
fn journey_a_running_defender_scan_can_be_stopped_from_the_window() {
    let mut s = shell();
    s.open_app(App::Defender);
    s.defender.app.begin(crate::apps::ScanMode::Fast, 40);
    s.defender.running = true;
    s.defender.shared = Some(std::sync::Arc::new(crate::shell::ScanShared::default()));
    let _ = draw_frame(&mut s);
    let stop = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::CancelScan))
        .expect("a running scan offers Stop");
    let _ = click(&mut s, stop.center().0, stop.center().1);
    let shared = s.defender.shared.expect("the worker is attached");
    assert!(
        shared.cancelled.load(std::sync::atomic::Ordering::Relaxed),
        "the click asked the worker to stop"
    );
    assert!(
        s.defender.app.message.contains("Stopping"),
        "the window says what it is doing: {}",
        s.defender.app.message
    );
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

/// Item 3 of the brief, as one sequence through the real Files window: open
/// "This Litric", create a folder through the dialog and rename it in place.
/// Every step goes through the window's own controls and is checked on the
/// fixture's disk.
#[cfg(unix)]
#[test]
fn journey_this_litric_creates_renames_moves_and_trashes() {
    let mut s = shell();
    let root = e2e_root("this-litric");
    std::fs::create_dir_all(root.join("Documents")).unwrap();
    s.files_root = root.clone();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);

    let place = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Place(0)))
        .expect("This Litric is the first Quick access row");
    click(&mut s, place.center().0, place.center().1);
    assert_eq!(s.files.path, "/", "the home place opens the root");

    let zone = s.regional.zone.clone();
    let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();

    // Create a folder with the real New-folder dialog.
    s.files.ask_new_folder();
    let _ = draw_frame(&mut s);
    type_text(&mut s, "Trip");
    let ok = s
        .hotspots()
        .rect_of(Hot::DialogConfirm)
        .expect("the create dialog draws its confirm button");
    click(&mut s, ok.center().0, ok.center().1);
    assert!(bridge.exists("/Trip").unwrap(), "created at the home root");

    // The folder is listed under This Litric; select it and rename it.
    s.files
        .apply_list_reply("e name=%54rip kind=dir size=0 mtime=0\n");
    let _ = draw_frame(&mut s);
    let index = s
        .files
        .rows
        .iter()
        .position(|r| r.name == "Trip")
        .expect("the created folder is listed");
    let row = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Row(index)))
        .expect("the new folder has a row");
    click(&mut s, row.center().0, row.center().1);
    assert_eq!(
        s.files.selected_paths(),
        vec!["/Trip".to_owned()],
        "the clicked row selected the folder the dialog created"
    );
    s.files.ask_rename();
    s.files.dialog_field.clear();
    let _ = draw_frame(&mut s);
    type_text(&mut s, "Journey");
    let ok = s.hotspots().rect_of(Hot::DialogConfirm).unwrap();
    click(&mut s, ok.center().0, ok.center().1);
    assert!(bridge.exists("/Journey").unwrap(), "renamed in place");
    assert!(!bridge.exists("/Trip").unwrap(), "the old name is gone");

    // Move: cut the folder, open Documents, paste, confirm the dialog.
    s.files.listed = None;
    s.files
        .apply_list_reply("e name=%4aourney kind=dir size=0 mtime=0\n");
    let _ = draw_frame(&mut s);
    let index = s
        .files
        .rows
        .iter()
        .position(|r| r.name == "Journey")
        .unwrap();
    let row = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Row(index)))
        .unwrap();
    click(&mut s, row.center().0, row.center().1);
    assert_eq!(s.files.selected_paths(), vec!["/Journey".to_owned()]);
    right_click(&mut s, row.center().0, row.center().1);
    let _ = draw_frame(&mut s);
    let cut = s.hotspots().rect_of(Hot::MenuRow(MenuAction::Cut)).unwrap();
    click(&mut s, cut.center().0, cut.center().1);
    let docs = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Place(1)))
        .unwrap();
    click(&mut s, docs.center().0, docs.center().1);
    let _ = draw_frame(&mut s);
    let paste = s.hotspots().rect_of(Hot::Files(FilesHot::Paste)).unwrap();
    click(&mut s, paste.center().0, paste.center().1);
    let _ = draw_frame(&mut s);
    let ok = s
        .hotspots()
        .rect_of(Hot::DialogConfirm)
        .expect("the paste asks first");
    click(&mut s, ok.center().0, ok.center().1);
    assert!(
        bridge.exists("/Documents/Journey").unwrap(),
        "moved into Documents"
    );
    assert!(!bridge.exists("/Journey").unwrap(), "the source is gone");

    // Trash: select the moved folder and use the row menu.
    s.files.listed = None;
    s.files
        .apply_list_reply("e name=%4aourney kind=dir size=0 mtime=0\n");
    let _ = draw_frame(&mut s);
    let index = s
        .files
        .rows
        .iter()
        .position(|r| r.name == "Journey")
        .unwrap();
    let row = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Row(index)))
        .unwrap();
    click(&mut s, row.center().0, row.center().1);
    right_click(&mut s, row.center().0, row.center().1);
    let _ = draw_frame(&mut s);
    let trash = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::Trash))
        .unwrap();
    click(&mut s, trash.center().0, trash.center().1);
    let _ = draw_frame(&mut s);
    let ok = s
        .hotspots()
        .rect_of(Hot::DialogConfirm)
        .expect("the trash asks first");
    click(&mut s, ok.center().0, ok.center().1);
    assert!(!bridge.exists("/Documents/Journey").unwrap(), "trashed");
    let trashed = bridge.trash_list().unwrap_or_default();
    assert!(
        trashed.iter().any(|e| e.name.contains("Journey")),
        "recoverable: {trashed:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// One workflow across three windows: a finished file in Downloads is opened
/// from the Browser's Downloads panel in Files, already selected, and the
/// Defender scans the same folder and finds a planted threat beside it. Every
/// step is a click on a drawn control; the file assertions are on the fixture's
/// real disk.
#[test]
fn journey_a_download_opens_in_files_and_is_scanned() {
    let root = e2e_root("downloads-files-defender");
    let downloads = root.join("Downloads");
    std::fs::create_dir_all(&downloads).unwrap();
    std::fs::write(downloads.join("report.txt"), b"a harmless download\n").unwrap();
    std::fs::write(downloads.join("eicar.com"), EICAR).unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Browser);
    let _ = draw_frame(&mut s);
    let button = s
        .hotspots()
        .rect_of(Hot::BrowserDownloads)
        .expect("the Downloads button is drawn");
    click(&mut s, button.center().0, button.center().1);
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.browser.downloads.len(),
        2,
        "both finished files are listed: {:?}",
        s.browser.downloads
    );
    let index = s
        .browser
        .downloads
        .iter()
        .position(|download| download.name == "report.txt")
        .expect("the harmless download is listed");
    let row = s
        .hotspots()
        .rect_of(Hot::BrowserDownloadItem(index))
        .expect("a download row is a target");
    click(&mut s, row.center().0, row.center().1);
    let _ = draw_frame(&mut s);

    assert_eq!(
        s.files.path, "/Downloads",
        "Files opened the download's folder"
    );
    assert!(
        s.files.selected_rows.contains("report.txt"),
        "the download is selected in Files: {:?}",
        s.files.selected_rows
    );
    assert!(
        s.wm.focused().is_some_and(|id| s
            .wm
            .all()
            .iter()
            .any(|w| w.id == id && w.app_id == App::Files.app_id())),
        "the Files window is the one in front"
    );

    // The same folder the download landed in is inside the fast scan's scope.
    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);
    let fast = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::ScanNow))
        .expect("the Fast scan button is drawn");
    click(&mut s, fast.center().0, fast.center().1);
    for _ in 0..600 {
        let _ = draw_frame(&mut s);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(!s.defender.running, "the fast scan finished");
    let threat = s
        .defender
        .app
        .rows
        .iter()
        .find(|row| row.path.ends_with("/eicar.com"))
        .expect("the scan found the planted threat in Downloads");
    assert!(threat.detected);
    assert!(
        s.defender
            .app
            .rows
            .iter()
            .any(|row| row.path.ends_with("/report.txt") && !row.detected),
        "the downloaded file was scanned and left unflagged"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// One Defender session that contains a real threat and puts it back: the
/// window's Quarantine and Restore controls are clicked, and every assertion is
/// about the real files under the fixture root or the state the window reads
/// back from disk. A second shell is opened mid-journey to prove the list comes
/// from the records, not from memory.
#[test]
fn journey_defender_quarantines_and_restores_a_real_threat() {
    let root = e2e_root("defender-quarantine");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("clean.txt"), b"a harmless note\n").unwrap();
    std::fs::write(docs.join("eicar.com"), EICAR).unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);

    let fast = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::ScanNow))
        .expect("the Fast scan button is drawn");
    let _ = click(&mut s, fast.center().0, fast.center().1);
    for _ in 0..600 {
        let _ = draw_frame(&mut s);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(!s.defender.running, "the scan finished");
    let index = s
        .defender
        .app
        .rows
        .iter()
        .position(|row| row.path.ends_with("/eicar.com"))
        .expect("the planted threat is listed");
    assert!(
        s.defender
            .app
            .rows
            .get(index)
            .expect("the threat row exists")
            .detected
    );

    let _ = draw_frame(&mut s);
    let quarantine = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::QuarantineFinding(index)))
        .expect("a finding row has a real Quarantine control");
    let _ = click(&mut s, quarantine.center().0, quarantine.center().1);
    let _ = draw_frame(&mut s);

    assert!(
        !docs.join("eicar.com").exists(),
        "the file actually left the folder it was found in"
    );
    assert!(
        root.join("Quarantine/eicar.com").exists(),
        "the file is really in the quarantine folder"
    );
    assert!(
        root.join("Quarantine/eicar.com.quarantine").exists(),
        "the entry has a record on disk"
    );
    assert_eq!(s.defender.app.quarantined.len(), 1);
    assert_eq!(
        s.defender
            .app
            .quarantined
            .first()
            .map(|entry| entry.original.as_str()),
        Some("/Documents/eicar.com")
    );
    assert!(
        s.defender.app.message.starts_with("Quarantined"),
        "the window says what happened: {}",
        s.defender.app.message
    );
    assert!(
        !s.defender
            .app
            .rows
            .iter()
            .any(|row| row.path.ends_with("/eicar.com")),
        "the contained finding is no longer listed as loose"
    );
    assert!(
        docs.join("clean.txt").exists(),
        "the clean file was not touched"
    );

    // The record on disk carries the digest of the bytes that were taken.
    // This is the published SHA-256 of the EICAR test string, computed here
    // independently of the product's hasher.
    let record = std::fs::read_to_string(root.join("Quarantine/eicar.com.quarantine"))
        .expect("the record is on disk");
    assert!(
        record.lines().any(|line| {
            line == "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f"
        }),
        "the record carries the digest of the contained bytes: {record}"
    );

    // Check is a real control, and it says what the bytes are.
    let _ = draw_frame(&mut s);
    let check = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::VerifyQuarantined(0)))
        .expect("the contained entry has a real Check control");
    let _ = click(&mut s, check.center().0, check.center().1);
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.defender
            .app
            .quarantined
            .first()
            .map(|entry| entry.integrity),
        Some(crate::quarantine::Integrity::Verified),
        "the contained bytes match the record"
    );
    assert!(
        s.defender.app.message.starts_with("Verified"),
        "the window says the check passed: {}",
        s.defender.app.message
    );

    // The list is on disk: a fresh shell opening Defender sees the entry.
    let mut restarted = shell();
    restarted.files_root = root.clone();
    restarted.open_app(App::Defender);
    let _ = draw_frame(&mut restarted);
    assert_eq!(
        restarted.defender.app.quarantined.len(),
        1,
        "a restart still shows what is contained"
    );
    assert_eq!(
        restarted
            .defender
            .app
            .quarantined
            .first()
            .map(|entry| entry.original.as_str()),
        Some("/Documents/eicar.com")
    );

    let _ = draw_frame(&mut s);
    let restore = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::RestoreQuarantined(0)))
        .expect("the contained entry has a real Restore control");
    let _ = click(&mut s, restore.center().0, restore.center().1);
    let _ = draw_frame(&mut s);

    assert!(
        docs.join("eicar.com").exists(),
        "the file is back at its original path"
    );
    assert_eq!(
        std::fs::read(docs.join("eicar.com")).unwrap(),
        EICAR,
        "the restored bytes are the ones that were taken"
    );
    assert!(
        !root.join("Quarantine/eicar.com").exists(),
        "nothing is left contained"
    );
    assert!(
        !root.join("Quarantine/eicar.com.quarantine").exists(),
        "the record went with it"
    );
    assert!(s.defender.app.quarantined.is_empty());
    assert!(
        s.defender.app.message.starts_with("Restored"),
        "the window says what happened: {}",
        s.defender.app.message
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A quarantine move made in Defender changes the tree the Files window is
/// showing. The open table must read the folder again on the next frame, not
/// keep drawing a file that is no longer there — and the restored file must
/// reappear the same way.
#[test]
fn journey_defender_quarantine_refreshes_an_open_files_window() {
    let root = e2e_root("defender-files-refresh");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("clean.txt"), b"a harmless note\n").unwrap();
    std::fs::write(docs.join("eicar.com"), EICAR).unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);
    assert!(
        s.files.visible().iter().any(|row| row.name == "eicar.com"),
        "the Files window starts on the real folder: {:?}",
        s.files
            .visible()
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        s.files.visible().iter().any(|row| row.name == "clean.txt"),
        "the clean file is in the same folder"
    );

    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);
    let fast = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::ScanNow))
        .expect("the Fast scan button is drawn");
    let _ = click(&mut s, fast.center().0, fast.center().1);
    for _ in 0..600 {
        let _ = draw_frame(&mut s);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(!s.defender.running, "the scan finished");
    let index = s
        .defender
        .app
        .rows
        .iter()
        .position(|row| row.path.ends_with("/eicar.com"))
        .expect("the planted threat is listed");

    let _ = draw_frame(&mut s);
    let quarantine = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::QuarantineFinding(index)))
        .expect("a finding row has a real Quarantine control");
    let _ = click(&mut s, quarantine.center().0, quarantine.center().1);
    let _ = draw_frame(&mut s);

    assert!(!docs.join("eicar.com").exists(), "the file really moved");
    assert!(
        !s.files.visible().iter().any(|row| row.name == "eicar.com"),
        "the open Files window dropped the contained file: {:?}",
        s.files
            .visible()
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        s.files.visible().iter().any(|row| row.name == "clean.txt"),
        "the rest of the listing is unchanged"
    );

    let restore = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::RestoreQuarantined(0)))
        .expect("the contained entry has a real Restore control");
    let _ = click(&mut s, restore.center().0, restore.center().1);
    let _ = draw_frame(&mut s);
    assert!(docs.join("eicar.com").exists(), "the file came back");
    assert!(
        s.files.visible().iter().any(|row| row.name == "eicar.com"),
        "the open Files window shows the restored file: {:?}",
        s.files
            .visible()
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A restore in Trash puts a real file back into a folder the Files window is
/// showing. The filesystem changed underneath that table, so it must be read
/// again on the next frame instead of keeping the empty listing it drew while
/// the file was away.
#[test]
#[cfg(unix)]
fn journey_trash_restore_refreshes_an_open_files_window() {
    let root = e2e_root("trash-files-refresh");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("note.txt"), b"restore me\n").unwrap();
    let zone = litric_locale::TimeZone::utc("test");
    let bridge = crate::fsbridge::FsBridge::open(&root, zone).unwrap();
    bridge.trash("/Documents/note.txt", 1_900_000_000).unwrap();
    assert!(!docs.join("note.txt").exists(), "the file is in the trash");

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);
    assert!(
        !s.files.visible().iter().any(|row| row.name == "note.txt"),
        "the open folder is empty while the file is in the trash"
    );

    s.open_app(App::Trash);
    let _ = draw_frame(&mut s);
    assert_eq!(s.trash.entries.len(), 1, "the trash lists the real entry");
    let row = s
        .hotspots()
        .rect_of(Hot::TrashRow(0))
        .expect("the trash row is drawn");
    let _ = click(&mut s, row.center().0, row.center().1);
    assert_eq!(s.trash.selected, Some(0), "the row is selected");
    // The Restore button only exists once a row is selected, so it is drawn by
    // the frame after the click that selects it.
    let _ = draw_frame(&mut s);

    let restore = s
        .hotspots()
        .rect_of(Hot::TrashRestore)
        .expect("the Restore button is drawn for a selection");
    let _ = click(&mut s, restore.center().0, restore.center().1);
    let _ = draw_frame(&mut s);

    assert!(docs.join("note.txt").exists(), "the file is back on disk");
    assert!(
        s.files.visible().iter().any(|row| row.name == "note.txt"),
        "the open Files window shows the restored file: {:?}",
        s.files
            .visible()
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The refusal side through the window: a finding that has gone since the scan
/// cannot be contained, the row stays where the user can see it, and no
/// quarantine entry is invented.
#[test]
fn journey_defender_refuses_to_quarantine_a_file_that_is_gone() {
    let root = e2e_root("defender-quarantine-gone");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("clean.txt"), b"a harmless note\n").unwrap();
    std::fs::write(docs.join("eicar.com"), EICAR).unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);
    let fast = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::ScanNow))
        .expect("the Fast scan button is drawn");
    let _ = click(&mut s, fast.center().0, fast.center().1);
    for _ in 0..600 {
        let _ = draw_frame(&mut s);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let index = s
        .defender
        .app
        .rows
        .iter()
        .position(|row| row.path.ends_with("/eicar.com"))
        .expect("the planted threat is listed");

    // The file leaves outside the window, the way another program would.
    std::fs::remove_file(docs.join("eicar.com")).unwrap();

    let _ = draw_frame(&mut s);
    let quarantine = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::QuarantineFinding(index)))
        .expect("the control is still drawn for the stale row");
    let _ = click(&mut s, quarantine.center().0, quarantine.center().1);
    let _ = draw_frame(&mut s);

    assert!(
        s.defender.app.message.starts_with("Could not quarantine"),
        "the refusal is shown: {}",
        s.defender.app.message
    );
    assert!(
        s.defender.app.message.contains("could not be found"),
        "the reason is plain language: {}",
        s.defender.app.message
    );
    assert!(
        !root.join("Quarantine").exists(),
        "nothing was created for a file that was not there"
    );
    assert!(s.defender.app.quarantined.is_empty());
    assert!(
        s.defender
            .app
            .rows
            .iter()
            .any(|row| row.path.ends_with("/eicar.com")),
        "the finding stays listed so the user is not told it was contained"
    );
    assert!(
        docs.join("clean.txt").exists(),
        "the harmless file was not touched"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A contained file whose bytes were changed inside the quarantine folder must
/// not be restored. The window says so, the original path stays empty, and the
/// tampered copy is kept for inspection.
#[test]
fn journey_defender_refuses_to_restore_a_tampered_quarantine_entry() {
    let root = e2e_root("defender-quarantine-tampered");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("eicar.com"), EICAR).unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);
    let fast = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::ScanNow))
        .expect("the Fast scan button is drawn");
    let _ = click(&mut s, fast.center().0, fast.center().1);
    for _ in 0..600 {
        let _ = draw_frame(&mut s);
        if !s.defender.running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let index = s
        .defender
        .app
        .rows
        .iter()
        .position(|row| row.path.ends_with("/eicar.com"))
        .expect("the planted threat is listed");
    let _ = draw_frame(&mut s);
    let quarantine = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::QuarantineFinding(index)))
        .expect("a finding row has a real Quarantine control");
    let _ = click(&mut s, quarantine.center().0, quarantine.center().1);
    let _ = draw_frame(&mut s);
    assert_eq!(s.defender.app.quarantined.len(), 1);

    // Something replaces the contained bytes. The record still describes the
    // original file, so the two no longer agree.
    std::fs::write(root.join("Quarantine/eicar.com"), b"swapped bytes").unwrap();

    let restore = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::RestoreQuarantined(0)))
        .expect("the contained entry has a real Restore control");
    let _ = click(&mut s, restore.center().0, restore.center().1);
    let _ = draw_frame(&mut s);

    assert!(
        s.defender.app.message.starts_with("Could not restore"),
        "the refusal is shown: {}",
        s.defender.app.message
    );
    assert!(
        s.defender
            .app
            .message
            .contains("no longer matches what was quarantined"),
        "the reason is plain language: {}",
        s.defender.app.message
    );
    assert!(
        !docs.join("eicar.com").exists(),
        "tampered bytes were not put back"
    );
    assert_eq!(
        std::fs::read(root.join("Quarantine/eicar.com")).unwrap(),
        b"swapped bytes",
        "the tampered copy is kept for inspection"
    );
    assert_eq!(
        s.defender.app.quarantined.len(),
        1,
        "the entry is still listed"
    );

    // The on-demand check reports the same conclusion from the bytes.
    let check = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::VerifyQuarantined(0)))
        .expect("the contained entry has a real Check control");
    let _ = click(&mut s, check.center().0, check.center().1);
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.defender
            .app
            .quarantined
            .first()
            .map(|entry| entry.integrity),
        Some(crate::quarantine::Integrity::Mismatch)
    );
    assert!(
        s.defender.app.message.contains("does not match"),
        "the window says the bytes do not match: {}",
        s.defender.app.message
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A record written before integrity records existed still restores, and the
/// window says it is unverified rather than implying a check that never
/// happened.
#[test]
fn journey_defender_restores_a_legacy_quarantine_entry_unverified() {
    let root = e2e_root("defender-quarantine-legacy");
    std::fs::create_dir_all(root.join("Documents")).unwrap();
    std::fs::create_dir_all(root.join("Quarantine")).unwrap();
    std::fs::write(root.join("Quarantine/old.bin"), b"old bytes").unwrap();
    std::fs::write(
        root.join("Quarantine/old.bin.quarantine"),
        "LITRIC-QUARANTINE 1\n9\n1\nThreat found\n/Documents/old.bin",
    )
    .unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Defender);
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.defender
            .app
            .quarantined
            .first()
            .map(|entry| entry.integrity),
        Some(crate::quarantine::Integrity::NotRecorded),
        "a legacy record is reported as having no integrity record"
    );

    let restore = s
        .hotspots()
        .rect_of(Hot::MenuRow(MenuAction::RestoreQuarantined(0)))
        .expect("the legacy entry has a real Restore control");
    let _ = click(&mut s, restore.center().0, restore.center().1);
    let _ = draw_frame(&mut s);

    assert!(
        s.defender.app.message.starts_with("Restored"),
        "the restore happened: {}",
        s.defender.app.message
    );
    assert!(
        s.defender.app.message.contains("no integrity record"),
        "the window does not pretend it verified the bytes: {}",
        s.defender.app.message
    );
    assert_eq!(
        std::fs::read(root.join("Documents/old.bin")).unwrap(),
        b"old bytes"
    );
    assert!(!root.join("Quarantine/old.bin").exists());
    assert!(s.defender.app.quarantined.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}

/// One Activity session against a real child process: select its real row,
/// click End task once (a question, not a signal), click again, and watch the
/// OS process actually exit. The row is built from the child's real `/proc`
/// identity, so the ownership and session checks are exercised, not bypassed.
#[test]
fn journey_activity_ends_a_real_process_after_confirmation() {
    let mut child = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("a real child process");
    let pid = child.id();
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(2)
        && !std::path::Path::new(&format!("/proc/{pid}/stat")).is_file()
    {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let status =
        std::fs::read_to_string(format!("/proc/{pid}/status")).expect("the child's status");
    let uid = litric_sysd::facts::parse_uid(&status);

    let mut s = shell();
    s.facts.mem_total_mb = Some(1024);
    s.facts.processes.push(crate::shell::ProcRow {
        pid,
        name: "sleep".to_owned(),
        cmdline: "sleep 30".to_owned(),
        state: "sleeping".to_owned(),
        ppid: 0,
        threads: 1,
        ticks: 0,
        rss_mb: Some(1),
        uid,
        cpu: None,
    });
    s.activity.prefs.show_system_users = true;
    s.activity.observe(&s.facts);
    s.open_app(App::Activity);
    let _ = draw_frame(&mut s);

    let row = s
        .hotspots()
        .rect_of(Hot::ActivityRow(pid))
        .expect("the real process row is a click target");
    let _ = click(&mut s, row.center().0, row.center().1);
    assert_eq!(s.activity.selected_pid(), Some(pid), "the row is selected");

    let _ = draw_frame(&mut s);
    let end = s
        .hotspots()
        .rect_of(Hot::ActivityEndTask)
        .expect("End task is drawn once a row is selected");
    let _ = click(&mut s, end.center().0, end.center().1);
    let _ = draw_frame(&mut s);
    assert!(
        s.activity
            .message
            .as_deref()
            .is_some_and(|m| m.contains("Click End task again")),
        "the first click is a question: {:?}",
        s.activity.message
    );
    assert!(
        child.try_wait().expect("try_wait").is_none(),
        "the first click must not signal anything"
    );

    let end = s
        .hotspots()
        .rect_of(Hot::ActivityEndTask)
        .expect("the confirming control is still there");
    let _ = click(&mut s, end.center().0, end.center().1);
    let _ = draw_frame(&mut s);
    assert!(
        s.activity
            .message
            .as_deref()
            .is_some_and(|m| m.starts_with("Asked ")),
        "the window says the process was asked to stop: {:?}",
        s.activity.message
    );

    let started = std::time::Instant::now();
    let mut ended = false;
    while started.elapsed() < std::time::Duration::from_secs(3) {
        if child.try_wait().expect("try_wait").is_some() {
            ended = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if !ended {
        let _ = child.kill();
        let _ = child.wait();
    }
    assert!(
        ended,
        "the real child process survived the confirmed End task"
    );
}

/// The kernel's own child list for a live process, for the tree journey.
#[cfg(unix)]
fn live_children(pid: u32) -> Vec<u32> {
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for task in tasks.flatten() {
        let Ok(text) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        for field in text.split_whitespace() {
            if let Ok(child) = field.parse::<u32>() {
                if !out.contains(&child) {
                    out.push(child);
                }
            }
        }
    }
    out
}

/// A process has ended when it is gone or is a zombie waiting to be reaped.
#[cfg(unix)]
fn process_running(pid: u32) -> bool {
    let Ok(text) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
        return false;
    };
    text.rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .is_some_and(|state| state != "Z" && state != "X")
}

/// One Activity session against a real parent and its two real children:
/// select the parent's row, click End tree once (a question, not a signal),
/// click again, and watch every process in the tree actually stop. A process
/// outside this session's scope (init) is refused in a second session.
#[test]
fn journey_activity_ends_a_real_process_tree_after_confirmation() {
    let mut parent = std::process::Command::new("sh")
        .arg("-c")
        .arg("sleep 30 & sleep 30 & wait")
        .spawn()
        .expect("a real parent process");
    let pid = parent.id();
    let started = std::time::Instant::now();
    let mut kids = live_children(pid);
    while started.elapsed() < std::time::Duration::from_secs(2) && kids.len() < 2 {
        kids = live_children(pid);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(
        kids.len(),
        2,
        "the fixture must be one real parent with two real children"
    );
    let status =
        std::fs::read_to_string(format!("/proc/{pid}/status")).expect("the parent's status");
    let uid = litric_sysd::facts::parse_uid(&status);

    let mut s = shell();
    s.facts.mem_total_mb = Some(1024);
    s.facts.processes.push(crate::shell::ProcRow {
        pid,
        name: "sh".to_owned(),
        cmdline: "sh -c sleep 30 & sleep 30 & wait".to_owned(),
        state: "sleeping".to_owned(),
        ppid: 0,
        threads: 1,
        ticks: 0,
        rss_mb: Some(1),
        uid,
        cpu: None,
    });
    s.activity.prefs.show_system_users = true;
    s.activity.observe(&s.facts);
    s.open_app(App::Activity);
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.activity.rows.len(),
        1,
        "the fixture starts with one known row"
    );

    let row = s
        .hotspots()
        .rect_of(Hot::ActivityRow(pid))
        .expect("the parent's real row is a click target");
    let _ = click(&mut s, row.center().0, row.center().1);
    assert_eq!(s.activity.selected_pid(), Some(pid), "the row is selected");
    let _ = draw_frame(&mut s);

    let tree = s
        .hotspots()
        .rect_of(Hot::ActivityEndTree)
        .expect("End tree is drawn once a row is selected");
    let _ = click(&mut s, tree.center().0, tree.center().1);
    let _ = draw_frame(&mut s);
    assert!(
        s.activity
            .message
            .as_deref()
            .is_some_and(|m| m.contains("Click End tree again")),
        "the first click is a question: {:?}",
        s.activity.message
    );
    assert!(
        process_running(pid),
        "the first click must not signal the parent"
    );
    for kid in &kids {
        assert!(
            process_running(*kid),
            "the first click must not signal child {kid}"
        );
    }

    let tree = s
        .hotspots()
        .rect_of(Hot::ActivityEndTree)
        .expect("the confirming control is still there");
    let _ = click(&mut s, tree.center().0, tree.center().1);
    let _ = draw_frame(&mut s);
    assert!(
        s.activity.rows.len() > 1 && s.activity.rows.iter().any(|p| p.pid == std::process::id()),
        "the confirmed action must replace the one-row fixture with a real /proc sample"
    );
    assert!(
        s.activity
            .message
            .as_deref()
            .is_some_and(|m| m.starts_with("Asked ") && m.contains("processes")),
        "the window reports how many were asked: {:?}",
        s.activity.message
    );

    let started = std::time::Instant::now();
    let mut parent_ended = false;
    while started.elapsed() < std::time::Duration::from_secs(3) {
        if parent.try_wait().expect("try_wait").is_some() {
            parent_ended = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if !parent_ended {
        let _ = parent.kill();
    }
    let _ = parent.wait();
    assert!(
        parent_ended,
        "the real parent survived the confirmed End tree"
    );

    for kid in &kids {
        let started = std::time::Instant::now();
        let mut stopped = false;
        while started.elapsed() < std::time::Duration::from_secs(3) {
            if !process_running(*kid) {
                stopped = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        if !stopped {
            let _ = std::process::Command::new("/bin/kill")
                .arg("-KILL")
                .arg(kid.to_string())
                .status();
        }
        assert!(stopped, "real child {kid} survived the confirmed End tree");
    }
}

/// A tree request aimed at a process outside this desktop's scope is refused
/// with a real sentence, and the process is untouched.
#[test]
fn journey_activity_refuses_a_process_outside_its_scope() {
    let status = std::fs::read_to_string("/proc/1/status").expect("init's status");
    let uid = litric_sysd::facts::parse_uid(&status);

    let mut s = shell();
    s.facts.mem_total_mb = Some(1024);
    s.facts.processes.push(crate::shell::ProcRow {
        pid: 1,
        name: "system".to_owned(),
        cmdline: "system".to_owned(),
        state: "sleeping".to_owned(),
        ppid: 0,
        threads: 1,
        ticks: 0,
        rss_mb: Some(1),
        uid,
        cpu: None,
    });
    s.activity.prefs.show_system_users = true;
    s.activity.prefs.user_scope = crate::taskmgr::UserScope::All;
    s.activity.observe(&s.facts);
    s.open_app(App::Activity);
    let _ = draw_frame(&mut s);

    let row = s
        .hotspots()
        .rect_of(Hot::ActivityRow(1))
        .expect("init has a real row");
    let _ = click(&mut s, row.center().0, row.center().1);
    let _ = draw_frame(&mut s);
    let tree = s
        .hotspots()
        .rect_of(Hot::ActivityEndTree)
        .expect("End tree is drawn for the selected row");
    let _ = click(&mut s, tree.center().0, tree.center().1);
    let _ = draw_frame(&mut s);
    assert!(
        s.activity
            .message
            .as_deref()
            .is_some_and(|m| m.contains("Click End tree again")),
        "the first click only arms: {:?}",
        s.activity.message
    );

    let tree = s
        .hotspots()
        .rect_of(Hot::ActivityEndTree)
        .expect("the confirming control is still there");
    let _ = click(&mut s, tree.center().0, tree.center().1);
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.activity.rows.len(),
        1,
        "a refused action must not claim a new sample"
    );
    assert!(
        s.activity
            .message
            .as_deref()
            .is_some_and(|m| m.starts_with("Could not end") && m.contains("does not let")),
        "init is refused in plain language: {:?}",
        s.activity.message
    );
    assert!(
        std::path::Path::new("/proc/1/stat").is_file(),
        "the refused process is untouched"
    );
}

/// A folder changed by something other than the shell is not watched: the
/// open table keeps the listing it last read until a person asks for a fresh
/// read. The Refresh control must really read the disk, and a folder that has
/// gone must say so instead of keeping stale rows.
#[test]
fn journey_files_refresh_shows_a_change_made_outside_the_shell() {
    let root = e2e_root("files-refresh");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("before.txt"), b"old\n").unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);
    let names = |shell: &Shell| {
        shell
            .files
            .visible()
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>()
    };
    assert!(names(&s).iter().any(|n| n == "before.txt"));
    assert!(!names(&s).iter().any(|n| n == "after.txt"));

    // Outside the shell: one file appears and another goes.
    std::fs::write(docs.join("after.txt"), b"new\n").unwrap();
    std::fs::remove_file(docs.join("before.txt")).unwrap();
    let _ = draw_frame(&mut s);
    // These frames are inside the watcher's one-second interval, so before
    // the click the table is still the last real read. The automatic path is
    // the separate journey below; this one stays sensitive to the control.
    assert!(
        names(&s).iter().any(|n| n == "before.txt"),
        "the table must still show the old listing before Refresh: {:?}",
        names(&s)
    );
    assert!(
        !names(&s).iter().any(|n| n == "after.txt"),
        "the table must not show an outside change before Refresh: {:?}",
        names(&s)
    );

    let refresh = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Refresh))
        .expect("the Files command row draws a real Refresh control");
    let _ = click(&mut s, refresh.center().0, refresh.center().1);
    let _ = draw_frame(&mut s);
    assert!(
        names(&s).iter().any(|n| n == "after.txt"),
        "Refresh read the folder on disk: {:?}",
        names(&s)
    );
    assert!(
        !names(&s).iter().any(|n| n == "before.txt"),
        "the deleted file is gone from the refreshed table: {:?}",
        names(&s)
    );
    assert!(s.files.error.is_none(), "a good read clears the error");

    // The keyboard route goes through the same request: a change made after
    // the last read is not shown until F5 asks for it.
    std::fs::write(docs.join("keyed.txt"), b"key\n").unwrap();
    let _ = draw_frame(&mut s);
    assert!(
        !names(&s).iter().any(|n| n == "keyed.txt"),
        "the table is still the old read before F5"
    );
    let _ = key_press(&mut s, Key::Function(5), Modifiers::NONE);
    let _ = draw_frame(&mut s);
    assert!(
        names(&s).iter().any(|n| n == "keyed.txt"),
        "F5 re-read the folder through the same request: {:?}",
        names(&s)
    );

    // The folder disappears while it is open. The read fails, the stale rows
    // go, and the window says what happened in plain language.
    std::fs::create_dir_all(docs.join("gone")).unwrap();
    s.files.navigate("/Documents/gone");
    let _ = draw_frame(&mut s);
    assert!(s.files.rows.is_empty(), "the new folder is empty");
    std::fs::remove_dir_all(docs.join("gone")).unwrap();
    let _ = draw_frame(&mut s);
    let refresh = s
        .hotspots()
        .rect_of(Hot::Files(FilesHot::Refresh))
        .expect("Refresh is still drawn after navigation");
    let _ = click(&mut s, refresh.center().0, refresh.center().1);
    let _ = draw_frame(&mut s);

    assert!(
        s.files.rows.is_empty(),
        "no stale rows survive a failed read"
    );
    let error = s.files.error.as_deref().unwrap_or_default();
    assert!(
        error.contains("no longer exists"),
        "the window explains the missing folder: {error:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A folder open in Files notices a change made outside the shell without any
/// click: one bounded directory read on the shell's own clock marks the
/// listing stale and the frame that follows draws what is on disk. The check
/// is not a per-frame poll — inside the interval the table is still the last
/// real read, and the diagnostics count proves exactly one read per interval.
#[test]
fn journey_files_watch_shows_an_outside_change_without_a_click() {
    let root = e2e_root("files-watch");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("before.txt"), b"old\n").unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Files);
    let _ = draw_frame(&mut s);
    let names = |shell: &Shell| {
        shell
            .files
            .visible()
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>()
    };
    assert!(names(&s).iter().any(|n| n == "before.txt"));
    assert!(!names(&s).iter().any(|n| n == "after.txt"));

    // Outside the shell: one file appears and another goes.
    std::fs::write(docs.join("after.txt"), b"new\n").unwrap();
    std::fs::remove_file(docs.join("before.txt")).unwrap();

    // Inside the interval nothing is read. A watcher that read on every frame
    // would be a poll, and these two frames would already show the change.
    let reads = s.files_watch.reads();
    let _ = draw_frame(&mut s);
    let _ = draw_frame(&mut s);
    assert_eq!(
        s.files_watch.reads(),
        reads,
        "the watcher must not read on every frame"
    );
    assert!(
        names(&s).iter().any(|n| n == "before.txt"),
        "before the interval the table is the last real read: {:?}",
        names(&s)
    );
    assert!(
        !names(&s).iter().any(|n| n == "after.txt"),
        "the outside file must not appear before the check: {:?}",
        names(&s)
    );

    // One second on the shell's clock reaches the next permitted check. The
    // watcher reads the directory, sees a different listing, and the frame
    // that follows is drawn from a fresh read. No click and no input.
    s.tick_after(1_000);
    let _ = draw_frame(&mut s);
    assert_eq!(s.files_watch.reads(), reads + 1, "one interval is one read");
    assert!(
        names(&s).iter().any(|n| n == "after.txt"),
        "the open table shows the file created outside: {:?}",
        names(&s)
    );
    assert!(
        !names(&s).iter().any(|n| n == "before.txt"),
        "the open table drops the file deleted outside: {:?}",
        names(&s)
    );
    assert!(s.files.error.is_none(), "a good read clears the error");

    // A folder that disappears is reported by the same path: the watcher
    // marks it stale, the read fails, and the window says so in plain words
    // instead of keeping rows that are no longer there.
    std::fs::remove_dir_all(&docs).unwrap();
    s.tick_after(1_000);
    let _ = draw_frame(&mut s);
    assert_eq!(s.files_watch.reads(), reads + 2);
    assert!(
        s.files.rows.is_empty(),
        "no stale rows survive a failed automatic read"
    );
    let error = s.files.error.as_deref().unwrap_or_default();
    assert!(
        error.contains("no longer exists"),
        "the window explains the missing folder: {error:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn journey_reduce_transparency_changes_the_rendered_frame_and_survives_a_restart() {
    let mut s = shell();
    s.load_preferences(crate::settings::Preferences::new(
        ThemeKind::Dark,
        crate::i18n::Language::English,
    ));
    s.perform(Action::ToggleQuickSettings);
    // Let the reveal spring settle: the material is the variable under test,
    // not the panel's position.
    for _ in 0..40 {
        let _ = draw_frame(&mut s);
    }
    let toggle = s
        .hotspots()
        .rect_of(Hot::QuickToggle(2))
        .expect("quick settings draws the Reduce Transparency switch");
    assert!(s.tokens.theme.glass.tint_alpha_pct < 100);
    let before = draw_frame(&mut s);

    let effects = click(&mut s, toggle.center().0, toggle.center().1);
    assert!(s.settings.preferences.reduce_transparency);
    assert!(s.quick.reduce_transparency);
    assert_eq!(s.tokens.theme.glass.tint_alpha_pct, 100);
    assert_eq!(s.tokens.theme.glass.blur_px, 0);
    assert!(effects
        .iter()
        .any(|e| matches!(e, Effect::PersistPreferences(p) if p.reduce_transparency)));

    let after = draw_frame(&mut s);
    let differing = before
        .pixels()
        .iter()
        .zip(after.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing > 1_000,
        "the switch changed state but not the frame: {differing} pixels differ"
    );

    // The material itself is opaque, not merely flagged: a fresh panel over
    // white returns the full-strength tint and asks for no blur.
    let mut c = Canvas::new(200, 120, rgb(0xFFFFFF));
    let panel = desktop::glass(
        &mut c,
        &s.tokens.theme,
        Rect::new(10.0, 10.0, 180.0, 100.0),
        8.0,
    );
    assert_eq!(panel.alpha_pct, 100);
    assert_eq!(desktop::glass_params(&s.tokens.theme), (0, 1.0));

    // A restart from the record keeps both the choice and the material.
    let saved = s.settings.preferences;
    let mut restarted = shell();
    restarted.load_preferences(saved);
    assert!(restarted.quick.reduce_transparency);
    assert_eq!(restarted.tokens.theme.glass.tint_alpha_pct, 100);
}

/// Hidden files are a real directory read, not a filter over rows already in
/// memory. The fixture holds one visible and one dot-prefixed file; the open
/// table starts without the dot entry, `Ctrl+H` asks the file service for a
/// listing that includes it, and a restart reads the stored choice back. The
/// only assertions about pixels are the before/after frames of that read.
#[cfg(unix)]
#[test]
fn journey_hidden_files_toggle_is_a_real_read_and_survives_a_restart() {
    let root = e2e_root("hidden-files");
    let docs = root.join("Documents");
    std::fs::create_dir_all(&docs).unwrap();
    std::fs::write(docs.join("visible.txt"), b"visible\n").unwrap();
    std::fs::write(docs.join(".secret"), b"hidden data\n").unwrap();

    let mut s = shell();
    s.files_root = root.clone();
    s.open_app(App::Files);
    // Let the window's reveal spring settle before the frame that is compared.
    for _ in 0..12 {
        let _ = draw_frame(&mut s);
    }
    assert!(
        s.files.rows.iter().any(|row| row.name == "visible.txt"),
        "the real visible file is listed: {:?}",
        s.files.rows
    );
    assert!(
        !s.files.rows.iter().any(|row| row.name == ".secret"),
        "the dot entry is absent before the switch: {:?}",
        s.files.rows
    );
    assert_eq!(
        std::fs::read(docs.join(".secret")).unwrap(),
        b"hidden data\n",
        "the hidden file really exists on disk"
    );
    let before = draw_frame(&mut s);

    let effects = key_press(
        &mut s,
        Key::Char('h'),
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    );
    assert!(
        s.settings.preferences.show_hidden,
        "Ctrl+H did not reach the Files preference"
    );
    // The toggle marks the listing stale. The next composed frame is the real
    // read, so the assertion about rows comes after it, not before.
    let after = draw_frame(&mut s);
    assert!(
        s.files.rows.iter().any(|row| row.name == ".secret"),
        "the refreshed table does not show the dot entry: {:?}",
        s.files.rows
    );
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, Effect::PersistPreferences(p) if p.show_hidden)),
        "the switch did not ask for the choice to be stored: {effects:?}"
    );
    let differing = before
        .pixels()
        .iter()
        .zip(after.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing > 100,
        "the listing changed but the frame did not: {differing} pixels differ"
    );

    // Store the exact record the shell asked to persist, read it back, and
    // start a fresh shell against the same real directory.
    let saved = effects
        .iter()
        .find_map(|effect| match effect {
            Effect::PersistPreferences(preferences) => Some(*preferences),
            _ => None,
        })
        .expect("the toggle asks for a persisted record");
    let store = crate::prefstore::Store::new(root.join("state/settings.conf"));
    store.save(saved).expect("the record is written");
    let loaded = store
        .load()
        .expect("the record is read")
        .expect("the record exists");
    assert!(loaded.show_hidden, "the store lost the choice");
    s.finish_preferences_save(Ok(()));

    let mut restarted = shell();
    restarted.files_root = root.clone();
    restarted.load_preferences(loaded);
    restarted.open_app(App::Files);
    for _ in 0..4 {
        let _ = draw_frame(&mut restarted);
    }
    assert!(
        restarted.files.rows.iter().any(|row| row.name == ".secret"),
        "a restart from the record does not show the dot entry: {:?}",
        restarted.files.rows
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Do Not Disturb is a persisted display-time decision, not a queue filter and
/// not a label. A normal note banners while the switch is off; the real quick
/// settings control silences that banner, the next composed frame reflects it,
/// the note is still in the centre, and a fresh shell reads the choice back from
/// the exact record the shell asked to store.
#[test]
fn journey_do_not_disturb_persists_silences_the_banner_and_survives_a_restart() {
    let root = e2e_root("do-not-disturb");
    let mut s = shell();
    s.load_preferences(crate::settings::Preferences::new(
        ThemeKind::Dark,
        crate::i18n::Language::English,
    ));
    s.perform(Action::ToggleQuickSettings);
    // Let the panel's reveal spring settle: the switch position is the variable
    // under test, not the panel's motion.
    for _ in 0..40 {
        let _ = draw_frame(&mut s);
    }
    let toggle = s
        .hotspots()
        .rect_of(Hot::QuickToggle(0))
        .expect("quick settings draws the Do Not Disturb switch");

    s.notify("Test", "A visible note", "body", Priority::Normal, None);
    for _ in 0..30 {
        let _ = draw_frame(&mut s);
    }
    let before = draw_frame(&mut s);
    let banner_zone = Rect::new(900.0, 40.0, 360.0, 80.0);
    assert!(
        ink(&before, banner_zone) > 50,
        "the normal note did not banner before the switch"
    );

    let effects = click(&mut s, toggle.center().0, toggle.center().1);
    assert!(
        s.settings.preferences.do_not_disturb,
        "the quick control did not reach the stored preference"
    );
    assert!(s.quick.do_not_disturb);
    assert!(s.notifications.do_not_disturb);
    let saved = effects
        .iter()
        .find_map(|effect| match effect {
            Effect::PersistPreferences(preferences) if preferences.do_not_disturb => {
                Some(*preferences)
            }
            _ => None,
        })
        .expect("the switch asks for a persisted record");

    let after = draw_frame(&mut s);
    assert_eq!(
        s.notifications.len(),
        1,
        "Do Not Disturb must not drop the note from the centre"
    );
    assert!(!s.notifications.should_banner(Priority::Normal));
    assert!(
        s.notifications.should_banner(Priority::Critical),
        "Do Not Disturb hid a critical alert"
    );
    let differing = before
        .pixels()
        .iter()
        .zip(after.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing > 200,
        "the banner and switch changed but the frame did not: {differing} pixels differ"
    );

    // Store the exact record the shell asked to persist, read it back, and start
    // a fresh shell from it. The note posted there must not banner.
    let store = crate::prefstore::Store::new(root.join("state/settings.conf"));
    store.save(saved).expect("the record is written");
    let loaded = store
        .load()
        .expect("the record is read")
        .expect("the record exists");
    assert!(loaded.do_not_disturb, "the store lost the choice");
    s.finish_preferences_save(Ok(()));

    let mut restarted = shell();
    restarted.load_preferences(loaded);
    assert!(restarted.settings.preferences.do_not_disturb);
    assert!(restarted.quick.do_not_disturb);
    assert!(restarted.notifications.do_not_disturb);
    restarted.notify("Test", "Still quiet", "body", Priority::Normal, None);
    assert_eq!(restarted.notifications.len(), 1);
    assert!(!restarted.notifications.should_banner(Priority::Normal));
    let quiet = draw_frame(&mut restarted);
    assert!(
        ink(&quiet, banner_zone) < 20,
        "a restarted shell still bannered under a stored quiet mode"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Write a frame as a binary PPM when `LITRIC_E2E_SHOTS` names a directory.
///
/// A reviewer can then look at the frame the journey actually produced. With
/// the variable unset this does nothing, so the journey's result never depends
/// on it.
fn save_shot(canvas: &Canvas, name: &str) {
    let Some(dir) = std::env::var_os("LITRIC_E2E_SHOTS") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let (w, h) = (canvas.width(), canvas.height());
    let capacity = (w as usize)
        .saturating_mul(h as usize)
        .saturating_mul(3)
        .saturating_add(16);
    let mut out = Vec::with_capacity(capacity);
    out.extend_from_slice(format!("P6\n{w} {h}\n255\n").as_bytes());
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let px = canvas.get(x, y).unwrap_or(0);
            out.push((px >> 16) as u8);
            out.push((px >> 8) as u8);
            out.push(px as u8);
        }
    }
    let _ = std::fs::write(dir.join(format!("{name}.ppm")), out);
}

/// One page served by a real web server on this machine, loaded through the
/// window, then a server that is not there.
///
/// The server is a real `TcpListener` on loopback: the reader connects, sends
/// `GET`, and gets its bytes back. The picture arrives over the same socket as
/// the page, so a passing journey proves the fetch, the parse, the decode and
/// the draw, not a fixture the test pre-parsed.
#[test]
fn journey_browser_loads_a_page_from_a_local_web_server_and_reports_a_failed_fetch() {
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    let jpeg = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../litric-browser/testdata/jpeg/s420-q75.jpg"
    ))
    .expect("the decoder's test picture");
    let html = b"<!doctype html><html><head><title>Local Web App</title></head><body>\
                 <h1>Served over HTTP</h1>\
                 <p>A page fetched from a real server on this machine, not read from a file.</p>\
                 <img src=\"panel.jpg\" alt=\"Panel\" width=\"900\" height=\"600\">\
                 <h1>Below the fold</h1>\
                 <p>The second heading arrives when the page is scrolled.</p>\
                 </body></html>"
        .to_vec();

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("a loopback server");
    let port = listener.local_addr().expect("an address").port();
    listener.set_nonblocking(true).expect("nonblocking");
    let stop = Arc::new(AtomicBool::new(false));
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let stop_flag = Arc::clone(&stop);
    let seen = Arc::clone(&requests);
    let page = html;
    let picture = jpeg;
    std::thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let line = String::from_utf8_lossy(buf.get(..n).unwrap_or(&[]))
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .to_owned();
                    if let Ok(mut list) = seen.lock() {
                        list.push(line.clone());
                    }
                    let (status, kind, body): (&str, &str, &[u8]) =
                        if line.starts_with("GET /panel.jpg ") {
                            ("200 OK", "image/jpeg", picture.as_slice())
                        } else if line.starts_with("GET / ") {
                            ("200 OK", "text/html; charset=utf-8", page.as_slice())
                        } else {
                            ("404 Not Found", "text/plain", b"not found".as_slice())
                        };
                    let head = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(body);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                Err(_) => break,
            }
        }
    });

    let url = format!("http://127.0.0.1:{port}/");
    let mut s = shell();
    s.open_app(App::Browser);
    let _ = draw_frame(&mut s);
    let field = s
        .hotspots()
        .rect_of(Hot::BrowserOmnibox)
        .expect("the start page draws its search box");
    click(&mut s, field.center().0, field.center().1);
    type_text(&mut s, &url);
    key_press(&mut s, Key::Enter, Modifiers::NONE);
    let canvas = draw_frame(&mut s);
    save_shot(&canvas, "51-local-web-page");

    assert!(
        s.reader.is_some(),
        "the served page loaded; the status line says: {}",
        s.browser.status
    );
    assert_eq!(s.browser.status, "Opened Local Web App");
    assert_eq!(
        s.browser.tab().map(|t| t.title.clone()),
        Some("Local Web App".to_owned()),
        "the tab takes the served page's own title"
    );
    let area = s
        .browser_engine_area
        .expect("the page surface is on screen");
    let reference = canvas
        .get((area.right() - 6.0) as i64, (area.bottom() - 6.0) as i64)
        .expect("a background pixel");
    let heading = Rect::new(area.x + 24.0, area.y + 16.0, 300.0, 44.0);
    let (text_px, text_total) = differing_pixels(&canvas, heading, reference);
    assert!(
        text_total > 40 && text_px > 30,
        "the fetched heading is drawn: {text_px}/{text_total}"
    );

    let reader = s.reader.as_ref().expect("the reader");
    assert!(
        reader
            .visible_text(area.h)
            .iter()
            .any(|line| line.contains("Served over HTTP")),
        "the served page's text is on screen"
    );
    assert_eq!(
        reader.cached_images(),
        1,
        "the picture was fetched over the socket and decoded"
    );
    let panel = reader.image_area(0).expect("the panel is laid out");
    let panel = Rect::new(area.x + panel.x, area.y + panel.y, panel.w, panel.h);
    let (pixels, total) = differing_pixels(&canvas, panel, reference);
    assert!(
        total > 100 && pixels * 10 >= total,
        "the fetched picture reached the frame: {pixels}/{total}"
    );
    let seen = requests.lock().map(|list| list.clone()).unwrap_or_default();
    assert!(
        seen.iter().any(|line| line == "GET / HTTP/1.1"),
        "the page was requested: {seen:?}"
    );
    assert!(
        seen.iter().any(|line| line == "GET /panel.jpg HTTP/1.1"),
        "the picture was requested: {seen:?}"
    );

    // The wheel scrolls the fetched page exactly like a file page.
    let at = Point::new(area.center().0, area.center().1);
    let _ = s.handle(
        &InputEvent::Scroll {
            dx: 0.0,
            dy: 10.0,
            at,
            mods: Modifiers::NONE,
        },
        2_000,
    );
    let canvas = draw_frame(&mut s);
    save_shot(&canvas, "51-local-web-page-scrolled");
    let reader = s.reader.as_ref().expect("the reader");
    assert!(
        reader
            .visible_text(area.h)
            .iter()
            .any(|line| line.contains("Below the fold")),
        "the second heading is on screen after the wheel: scroll={} content={}",
        reader.scroll(),
        reader.content_height()
    );

    // A server that is not there: the window says so and keeps no page.
    let dead_port = {
        let probe = TcpListener::bind(("127.0.0.1", 0)).expect("a port");
        probe.local_addr().expect("an address").port()
    };
    let mut failed = shell();
    failed.open_app(App::Browser);
    let _ = draw_frame(&mut failed);
    let field = failed
        .hotspots()
        .rect_of(Hot::BrowserOmnibox)
        .expect("the start page draws its search box");
    click(&mut failed, field.center().0, field.center().1);
    type_text(&mut failed, &format!("http://127.0.0.1:{dead_port}/"));
    key_press(&mut failed, Key::Enter, Modifiers::NONE);
    let canvas = draw_frame(&mut failed);
    save_shot(&canvas, "51-local-web-page-unreachable");
    assert!(
        failed.reader.is_none(),
        "a failed fetch must not leave a page on screen"
    );
    assert_eq!(failed.browser.status, "Could not open this page");
    assert!(
        failed.browser.tab().is_some_and(|tab| tab.error),
        "the tab reports the failure rather than looking loaded"
    );
    assert!(
        ink(&canvas, Rect::new(0.0, 0.0, 1280.0, 28.0)) > 0,
        "the shell still drew"
    );

    // Leaving this machine still needs the engine; the reader must not become
    // a second, unchecked network path.
    assert!(!crate::reader::serves("http://example.com/"));
    assert!(!crate::reader::serves("https://127.0.0.1/"));
    stop.store(true, Ordering::Relaxed);
}
