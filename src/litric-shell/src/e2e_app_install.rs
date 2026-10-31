//! Real-socket proof for the application install, launch and uninstall path.
//!
//! This lives beside the shell's other end-to-end journeys so it can use the
//! real desktop discovery code. The daemon is a real `litric-sysd` on a Unix
//! socket, the installed program is a real executable copy, and the marker is
//! written by that copy after launch.
//!
//! The second journey is the S82 proof: the installed program is the Weston
//! reference client `weston-simple-shm`, a real third-party Wayland
//! application. It is installed by `APP INSTALL`, launched by the shell through
//! the daemon with `LAUNCHENV`, maps a real `xdg_toplevel`, and its pixels are
//! composed by `views::draw`. Nothing in this file makes a client window itself.

#[cfg(unix)]
#[test]
fn journey_app_install_launch_uninstall_through_sysd() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    let root = std::env::temp_dir().join(format!(
        "litric-shell-e2e-app-install-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a fixture root");

    let data = root.join("data");
    let incoming = data.join("litric/incoming/fixture");
    std::fs::create_dir_all(&incoming).expect("incoming bundle");
    let marker = root.join("installed-marker");
    std::fs::write(
        incoming.join("app"),
        format!(
            "#!/bin/sh\nprintf '%s' 'installed-app-ran' > '{}'\n",
            marker.display()
        ),
    )
    .expect("the fixture program");
    std::fs::set_permissions(incoming.join("app"), std::fs::Permissions::from_mode(0o755))
        .expect("chmod");
    std::fs::write(
        incoming.join("app.desktop"),
        "[Desktop Entry]\nType=Application\nName=Installed Fixture\nExec=app\n",
    )
    .expect("the bundle description");

    let socket = root.join("sysd.sock");
    let listener = litric_sysd::server::bind(&socket).expect("bind sysd");
    let state = litric_sysd::state::Sysd::new(2, 512, "/tmp", std::path::Path::new("/"))
        .with_application_root(&data);
    std::thread::spawn(move || {
        let _ = litric_sysd::server::serve(&listener, state);
    });

    let installed =
        litric_sysd::server::query(&socket, "APP INSTALL com.example.installed fixture")
            .expect("install over the socket");
    assert_eq!(
        installed, "OK 2\nid=com.example.installed\nstate=installed\n",
        "the daemon reports the install"
    );
    let installed_app = data.join("litric/apps/com.example.installed/app");
    assert!(installed_app.is_file(), "the executable was written");

    let apps = crate::desktop_entry::discover_in(&[data.join("applications")]);
    assert_eq!(apps.len(), 1, "the installed entry is discovered: {apps:?}");
    let app = apps.first().expect("the discovered application");
    assert_eq!(app.name, "Installed Fixture");
    assert_eq!(app.exec.len(), 1);
    let executable = app.exec.first().expect("the discovered executable");
    assert_eq!(std::path::Path::new(executable), installed_app);

    let launch = format!(
        "LAUNCH {}",
        litric_sysd::launch::hex_encode(executable.as_bytes())
    );
    let started = litric_sysd::server::query(&socket, &launch).expect("launch over the socket");
    assert!(started.starts_with("OK 2\npid="), "launch reply: {started}");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut written: Option<String> = None;
    while Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(&marker) {
            written = Some(text);
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        written.as_deref(),
        Some("installed-app-ran"),
        "the installed copy ran and wrote its marker"
    );

    let removed = litric_sysd::server::query(&socket, "APP UNINSTALL com.example.installed")
        .expect("uninstall over the socket");
    assert_eq!(removed, "OK 2\nid=com.example.installed\nstate=removed\n");
    assert!(!installed_app.exists(), "the executable is gone");
    assert!(
        !data
            .join("applications/com.example.installed.desktop")
            .exists(),
        "the desktop entry is gone"
    );
    assert!(
        crate::desktop_entry::discover_in(&[data.join("applications")]).is_empty(),
        "discovery no longer lists the uninstalled app"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Failure paths through the same real socket: a refused install, an app that
/// exits before mapping, and a launch whose executable disappeared after
/// discovery. The first is a daemon error, the second a shell liveness result,
/// and the third the existing `Could not start` path.
#[cfg(unix)]
#[test]
fn journey_bad_install_and_an_app_that_exits_are_reported() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::Duration;

    let root = std::env::temp_dir().join(format!(
        "litric-shell-e2e-app-failure-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a fixture root");

    let data = root.join("data");
    let incoming = data.join("litric/incoming/exiting");
    std::fs::create_dir_all(&incoming).expect("incoming bundle");
    let app = incoming.join("app");
    std::fs::write(&app, "#!/bin/sh\nexit 0\n").expect("the early-exit program");
    std::fs::set_permissions(&app, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    std::fs::write(
        incoming.join("app.desktop"),
        "[Desktop Entry]\nType=Application\nName=Exiting Fixture\nExec=app\n",
    )
    .expect("the bundle description");

    let socket = root.join("sysd.sock");
    let listener = litric_sysd::server::bind(&socket).expect("bind sysd");
    let state = litric_sysd::state::Sysd::new(2, 512, "/tmp", std::path::Path::new("/"))
        .with_application_root(&data);
    std::thread::spawn(move || {
        let _ = litric_sysd::server::serve(&listener, state);
    });

    let refused = litric_sysd::server::query(&socket, "APP INSTALL com.example.bad ../escape")
        .expect("install refusal over the socket");
    assert!(
        refused.starts_with("ERR "),
        "an unsafe bundle is refused, not installed: {refused}"
    );
    assert!(
        !data.join("applications/com.example.bad.desktop").exists(),
        "a refused install writes no desktop entry"
    );
    assert!(
        !data.join("litric/apps/com.example.bad").exists(),
        "a refused install writes no application directory"
    );

    let installed = litric_sysd::server::query(&socket, "APP INSTALL com.example.exiting exiting")
        .expect("install over the socket");
    assert_eq!(installed, "OK 2\nid=com.example.exiting\nstate=installed\n");
    let installed_app = data.join("litric/apps/com.example.exiting/app");
    assert!(installed_app.is_file(), "the installed executable exists");

    let apps = crate::desktop_entry::discover_in(&[data.join("applications")]);
    assert_eq!(apps.len(), 1, "the installed entry is discovered: {apps:?}");
    let mut shell = crate::shell::Shell::new(
        crate::ui::Tokens::new(
            litric_design::Theme::of(litric_design::ThemeKind::Dark),
            1.0,
        ),
        1280.0,
        800.0,
        litric_locale::Regional::fallback("test"),
    );
    shell.mode = crate::shell::DesktopMode::Ready;
    shell.sysd_socket = socket.clone();
    shell.load_desktop_apps(apps);

    let effects = shell.seek_open(crate::seek::Hit::Desktop(0));
    assert!(effects.is_empty(), "launching is an effect, not a window");
    assert!(
        shell
            .app_status
            .as_deref()
            .is_some_and(|status| status.starts_with("Started Exiting Fixture")),
        "the launcher first reports a real start: {:?}",
        shell.app_status
    );

    // The daemon reaps the child; the shell's bounded liveness check must then
    // replace the optimistic start with the observed exit.
    std::thread::sleep(Duration::from_millis(250));
    shell.tick(2_100);
    assert_eq!(
        shell.app_status.as_deref(),
        Some("Exiting Fixture exited before opening a window"),
        "an app that never mapped is not left looking started"
    );
    assert!(
        shell
            .notifications
            .items()
            .iter()
            .any(|note| note.title == "Exiting Fixture exited"),
        "the exit is a visible notification, not only a variable"
    );

    // Discovery still holds the entry, but the executable is gone. The daemon
    // refuses the stale launch and the launcher must say so.
    std::fs::remove_file(&installed_app).expect("remove the installed executable");
    let effects = shell.seek_open(crate::seek::Hit::Desktop(0));
    assert!(effects.is_empty());
    assert_eq!(
        shell.app_status.as_deref(),
        Some("Could not start Exiting Fixture"),
        "a stale launch is refused visibly"
    );

    let removed = litric_sysd::server::query(&socket, "APP UNINSTALL com.example.exiting")
        .expect("uninstall over the socket");
    assert_eq!(removed, "OK 2\nid=com.example.exiting\nstate=removed\n");
    let _ = std::fs::remove_dir_all(&root);
}

/// The real third-party application this repository's e2e installs.
///
/// Provenance and hashes are in `tests/fixtures/README.md`. The binary is the
/// Ubuntu Weston 13 `weston-simple-shm`; the library beside it is the exact
/// `libwayland-client.so.0` that binary was built against.
#[cfg(unix)]
fn weston_simple_shm() -> (std::path::PathBuf, std::path::PathBuf) {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    (
        fixture.join("weston-simple-shm"),
        fixture.join("wayland-libs"),
    )
}

/// Write the 1280x800 composition as the committed review PNG.
#[cfg(unix)]
fn write_app_png(path: &std::path::Path, canvas: &crate::canvas::Canvas) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let file = std::fs::File::create(path).expect("png file");
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, canvas.width(), canvas.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("png header");
    let capacity = (canvas.width() as usize)
        .saturating_mul(canvas.height() as usize)
        .saturating_mul(4);
    let mut bytes = Vec::with_capacity(capacity);
    for px in canvas.pixels() {
        bytes.extend_from_slice(&[
            ((px >> 16) & 0xFF) as u8,
            ((px >> 8) & 0xFF) as u8,
            (*px & 0xFF) as u8,
            ((px >> 24) & 0xFF) as u8,
        ]);
    }
    writer.write_image_data(&bytes).expect("png data");
}

/// Install a real Wayland application through the shipping socket, let the
/// shell's launcher start it through the daemon, prove its window is in shell
/// state and its pixels are painted by `views::draw`, then close and uninstall
/// it.
#[cfg(unix)]
#[test]
fn journey_real_linux_app_is_installed_launched_mapped_rendered_and_removed() {
    use litric_design::{Theme, ThemeKind};
    use litric_locale::Regional;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    use crate::canvas::Canvas;
    use crate::desktop;
    use crate::shell::{DesktopMode, Shell};
    use crate::ui::Tokens;

    let (client, libs) = weston_simple_shm();
    assert!(
        client.is_file(),
        "the real Wayland fixture is present: {}",
        client.display()
    );
    assert!(
        libs.join("libwayland-client.so.0").is_file(),
        "the fixture's Wayland client library is present"
    );
    // The fixture's one non-glibc dependency is not installed on this host.
    // Put it on the loader path of the daemon's children; the child itself is
    // started by the real launch path below.
    let mut loader = libs.clone().into_os_string();
    if let Some(existing) = std::env::var_os("LD_LIBRARY_PATH") {
        loader.push(":");
        loader.push(existing);
    }
    std::env::set_var("LD_LIBRARY_PATH", loader);

    let root =
        std::env::temp_dir().join(format!("litric-shell-e2e-linux-app-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let data = root.join("data");
    let runtime = root.join("runtime");
    std::fs::create_dir_all(&data).expect("data root");
    std::fs::create_dir_all(&runtime).expect("runtime root");
    std::fs::set_permissions(&runtime, std::fs::Permissions::from_mode(0o700)).expect("chmod");

    let incoming = data.join("litric/incoming/weston");
    std::fs::create_dir_all(&incoming).expect("incoming bundle");
    std::fs::copy(&client, incoming.join("app")).expect("stage the real client");
    std::fs::set_permissions(incoming.join("app"), std::fs::Permissions::from_mode(0o755))
        .expect("chmod the client");
    std::fs::write(
        incoming.join("app.desktop"),
        "[Desktop Entry]\nType=Application\nName=Weston Simple SHM\nExec=app\n",
    )
    .expect("the bundle description");

    let socket = root.join("sysd.sock");
    let listener = litric_sysd::server::bind(&socket).expect("bind sysd");
    let state = litric_sysd::state::Sysd::new(2, 512, "/tmp", std::path::Path::new("/"))
        .with_application_root(&data);
    std::thread::spawn(move || {
        let _ = litric_sysd::server::serve(&listener, state);
    });

    let installed = litric_sysd::server::query(&socket, "APP INSTALL com.weston.simple-shm weston")
        .expect("install over the socket");
    assert_eq!(
        installed, "OK 2\nid=com.weston.simple-shm\nstate=installed\n",
        "the daemon installed the real client"
    );
    let installed_app = data.join("litric/apps/com.weston.simple-shm/app");
    assert!(installed_app.is_file(), "the installed executable exists");
    assert_eq!(
        std::fs::read(&installed_app).expect("read installed client"),
        std::fs::read(&client).expect("read fixture client"),
        "the installer copied the real binary byte-for-byte"
    );

    let apps = crate::desktop_entry::discover_in(&[data.join("applications")]);
    assert_eq!(apps.len(), 1, "the installed desktop entry is discovered");
    assert_eq!(
        apps.first().map(|app| app.name.as_str()),
        Some("Weston Simple SHM")
    );

    let mut shell = Shell::new(
        Tokens::new(Theme::of(ThemeKind::Dark), 1.0),
        1280.0,
        800.0,
        Regional::fallback("test"),
    );
    shell.mode = DesktopMode::Ready;
    shell.sysd_socket = socket.clone();
    shell.load_desktop_apps(apps);
    shell
        .attach_clients(&runtime)
        .expect("the desktop can host the application's Wayland window");
    assert_eq!(
        shell.application_env(),
        vec![
            (
                "XDG_RUNTIME_DIR".to_owned(),
                runtime.to_string_lossy().into_owned()
            ),
            ("WAYLAND_DISPLAY".to_owned(), "wayland-0".to_owned()),
        ],
        "the launcher has the real socket to pass"
    );

    // The real launcher path: a discovered tile becomes `Hit::Desktop`, and
    // `launch_desktop` asks the daemon to start the installed executable.
    let effects = shell.seek_open(crate::seek::Hit::Desktop(0));
    assert!(effects.is_empty(), "launching is an effect, not a window");
    let status = shell.app_status.as_deref().unwrap_or_default();
    assert!(
        status.starts_with("Started Weston Simple SHM"),
        "the shell reports a started application: {status:?}"
    );

    let start = Instant::now();
    let mut t = 1_000_u64;
    let mut window = None;
    while start.elapsed() < Duration::from_secs(20) {
        t = t.saturating_add(16);
        shell.tick(t);
        if let Some(found) = shell
            .wm
            .all()
            .iter()
            .find(|window| window.app_id == "litric.client")
        {
            window = Some((found.id, found.frame, found.title.clone()));
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let Some((id, frame, title)) = window else {
        panic!(
            "the real client mapped no window; shell status: {:?}; client notes: {:?}",
            shell.app_status,
            shell.clients.as_ref().map(|host| host.notes())
        );
    };
    assert_eq!(title, "simple-shm", "the client's title reached the WM");
    assert!(
        frame.w > 100.0 && frame.h > 100.0,
        "the window has the client's size: {}x{}",
        frame.w,
        frame.h
    );
    assert!(
        shell.clients.as_ref().is_some_and(|host| host.hosts(id)),
        "the window is backed by a real client surface"
    );

    // The launch notification has served its purpose in `app_status`; let its
    // four-second lifetime elapse so the review frame shows the application,
    // not a toast cut off by the screen edge.
    t = t.saturating_add(5_000);
    shell.tick(t);

    // Compose through the shipping path. `draw_window` takes the newest client
    // frame and answers its frame callback; the reference client's ring pixels
    // must be present in the window body.
    let title_h = shell.tokens.px(shell.wm.chrome().title_h);
    let body = crate::canvas::Rect::new(
        frame.x,
        frame.y + title_h,
        frame.w,
        (frame.h - title_h).max(1.0),
    );
    let mut canvas = Canvas::new(1280, 800, 0x101820);
    let mut dark = 0_u32;
    let mut light = 0_u32;
    let draw_start = Instant::now();
    while draw_start.elapsed() < Duration::from_secs(10) {
        t = t.saturating_add(16);
        shell.tick(t);
        canvas = Canvas::new(1280, 800, 0x101820);
        desktop::wallpaper(&mut canvas, &Theme::of(ThemeKind::Dark));
        crate::views::draw(&mut canvas, &mut shell);
        dark = 0;
        light = 0;
        let mut y = body.y + 2.0;
        while y < body.bottom() - 2.0 {
            let mut x = body.x + 2.0;
            while x < body.right() - 2.0 {
                if let Some(px) = canvas.get(x as i64, y as i64) {
                    let (r, g, b) = ((px >> 16) & 0xFF, (px >> 8) & 0xFF, px & 0xFF);
                    if r < 8 && g < 8 && b < 8 {
                        dark = dark.saturating_add(1);
                    }
                    if r > 220 && g > 220 && b > 220 {
                        light = light.saturating_add(1);
                    }
                }
                x += 3.0;
            }
            y += 3.0;
        }
        if light > 5 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    if let Some(path) = std::env::var_os("LITRIC_E2E_PNG") {
        let path = std::path::Path::new(&path);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(path)
        };
        write_app_png(&path, &canvas);
    }
    assert!(
        light > 5,
        "the client's own ring pixels reached views::draw (dark={dark}, light={light})"
    );

    // Closing the WM window asks the real client to close; its socket then
    // reports `Gone`, which removes the surface and the window.
    assert!(shell.wm.close(id));
    let close_start = Instant::now();
    while close_start.elapsed() < Duration::from_secs(10) {
        t = t.saturating_add(16);
        shell.tick(t);
        if shell.clients.as_ref().is_some_and(|host| !host.hosts(id)) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        shell.clients.as_ref().is_some_and(|host| !host.hosts(id)),
        "the client surface is gone after the window closes"
    );
    assert!(
        shell
            .wm
            .all()
            .iter()
            .all(|window| window.app_id != "litric.client"),
        "no client window remains"
    );

    let removed = litric_sysd::server::query(&socket, "APP UNINSTALL com.weston.simple-shm")
        .expect("uninstall over the socket");
    assert_eq!(removed, "OK 2\nid=com.weston.simple-shm\nstate=removed\n");
    assert!(!installed_app.exists(), "the executable is gone");
    assert!(
        crate::desktop_entry::discover_in(&[data.join("applications")]).is_empty(),
        "the uninstalled application is no longer discoverable"
    );
    let _ = std::fs::remove_dir_all(&root);
}
