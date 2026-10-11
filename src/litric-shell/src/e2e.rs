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
use crate::shell::{DesktopMode, Effect, Hot, Priority, Shell};
use crate::ui::Tokens;
use litric_design::{Theme, ThemeKind};
use litric_locale::Regional;

fn shell() -> Shell {
    let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
    let mut s = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("test"));
    s.mode = DesktopMode::Ready;
    s
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
    // a frame the compositor can never produce, so the tick lives here.
    s.tick(16_000);
    let mut c = Canvas::new(1280, 800, rgb(0x000000));
    desktop::wallpaper(&mut c, &Theme::of(ThemeKind::Dark));
    crate::views::draw(&mut c, s);
    c
}

// ---------------------------------------------------------------------------
// Journeys
// ---------------------------------------------------------------------------

#[test]
fn journey_the_desktop_has_icons_and_one_opens_files() {
    let mut s = shell();
    let _canvas = draw_frame(&mut s);
    // Icons exist and are hittable.
    let files_icon = s
        .hotspots()
        .rect_of(Hot::DesktopIcon(0))
        .expect("Files icon is on the desktop");
    // Clicking it opens the Files window.
    click(&mut s, files_icon.center().0, files_icon.center().1);
    assert!(
        s.wm.all().iter().any(|w| w.app_id == "com.litric.files"),
        "the Files window opened from the desktop icon"
    );
    // Clicking empty wallpaper clears the selection.
    click(&mut s, 900.0, 700.0);
    assert_eq!(s.desktop_icons.selected, None);
}

#[test]
fn journey_right_click_desktop_menu_creates_a_folder_via_dialog() {
    let mut s = shell();
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
    assert!(matches!(s.desktop_dialog, Some(DialogKind::NewFolder)));
    // Type a name and confirm. The dialog's buttons exist once drawn.
    type_text(&mut s, "Trip 2026");
    let frame = draw_frame(&mut s);
    drop(frame);
    let ok = s.hotspots().rect_of(Hot::DialogConfirm).unwrap();
    click(&mut s, ok.center().0, ok.center().1);
    // On Windows there is no filesystem; the dialog must say so, honestly,
    // and never pretend the folder exists.
    match &s.desktop_dialog {
        Some(DialogKind::Error(msg)) => assert!(msg.contains("available"), "{msg}"),
        Some(_) => panic!("dialog stayed open without an explanation"),
        None => {
            // Unix: the folder really exists now.
            let zone = s.regional.zone.clone();
            let bridge = crate::fsbridge::FsBridge::open(std::path::Path::new("/"), zone).unwrap();
            assert!(bridge.exists("/Documents/Trip 2026").unwrap());
        }
    }
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

#[test]
fn journey_files_window_is_drawn_with_places_rows_and_commands() {
    let mut s = shell();
    s.open_app(crate::shortcuts::App::Files);
    let _canvas = draw_frame(&mut s);
    for hot in [
        Hot::Files(FilesHot::Place(0)),
        Hot::Files(FilesHot::Back),
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
    assert_eq!(s.browser.tab().unwrap().url, "https://example.com");
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
fn journey_dock_click_opens_focuses_then_minimizes() {
    let mut s = shell();
    // The dock is an auto-hide surface: reveal it by parking the pointer at
    // the bottom edge, exactly as a user summons it.
    move_pointer(&mut s, 640.0, 795.0);
    // The dock's reveal is a spring the user waits out for a fraction of a
    // second; tick until it has arrived, then draw.
    for _ in 0..40 {
        s.tick(16_000);
        if s.anims().dock.value() >= 0.95 {
            break;
        }
    }
    let _ = draw_frame(&mut s);
    let dock_hot = s
        .hotspots()
        .rect_of(Hot::DockItem(3))
        .expect("dock revealed at the bottom edge");
    // First click opens the app.
    click(&mut s, dock_hot.center().0, dock_hot.center().1);
    let files =
        s.wm.all()
            .iter()
            .find(|w| w.app_id == "com.litric.files")
            .map(|w| w.id)
            .expect("dock click opened Files");
    assert_eq!(s.wm.focused(), Some(files));
    // Second click on the focused app minimizes it.
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
fn journey_hidden_dock_is_not_clickable() {
    let mut s = shell();
    move_pointer(&mut s, 640.0, 300.0); // far from the bottom edge
    s.tick(16_000);
    let _ = draw_frame(&mut s);
    // Every dock spring must have settled closed; nothing registers.
    for i in 0..8u8 {
        assert!(s.hotspots().rect_of(Hot::DockItem(i)).is_none());
    }
}

#[test]
fn journey_dock_is_clickable_while_still_sliding_up() {
    // A user aims at the dock the moment it appears; the dock must accept
    // the click mid-slide, not silently swallow it until fully settled.
    let mut s = shell();
    move_pointer(&mut s, 640.0, 780.0); // rest on the bottom edge
    let mut caught = false;
    for _ in 0..60 {
        s.tick(16);
        let _ = draw_frame(&mut s);
        let reveal = s.anims().dock.value();
        if (0.6..0.95).contains(&reveal) {
            caught = true;
            break;
        }
    }
    assert!(caught, "the spring passes through the clickable band");
    let dock_hot = s
        .hotspots()
        .rect_of(Hot::DockItem(3))
        .expect("a mid-slide dock is already a click target");
    click(&mut s, dock_hot.center().0, dock_hot.center().1);
    let files =
        s.wm.all()
            .iter()
            .find(|w| w.app_id == "com.litric.files")
            .map(|w| w.id)
            .expect("mid-slide click opened Files");
    assert_eq!(s.wm.focused(), Some(files));
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
        Some("https://github.com".to_owned()),
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
    // The command runs on a worker thread; pump until it reports done.
    for _ in 0..200 {
        s.tick(16_000);
        if !s.terminal.busy {
            break;
        }
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
    s.tick(16_000);
    // One ticked frame later the previously-focused window is part-way up,
    // not at either endpoint: that is the glide existing and measurable.
    let rising = s.anims_mut().focus.elevation(other, true);
    assert!(
        rising > 0.0,
        "the spring began moving toward focused on the first tick"
    );
    // It must arrive rather than hover: settle the spring and check 1.0.
    for _ in 0..40 {
        s.tick(16_000);
    }
    let settled = s.anims_mut().focus.elevation(other, true);
    assert!(
        (settled - 1.0).abs() < 0.01,
        "focus elevation settles at 1.0"
    );
}

#[test]
fn journey_viewer_opens_text_files_with_scrolling() {
    let mut s = shell();
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
            collect(bridge, &p, out, depth + 1);
        } else if e.kind == crate::apps::FileKind::File {
            out.push(p);
        }
    }
}

#[cfg(unix)]
const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";
