//! A foreign application becomes a LitricOS window.
//!
//! This is the whole vertical slice in one test: a Wayland socket is created
//! inside a session runtime directory, a real client from outside this
//! repository connects to it, the desktop gives it a window with a LitricOS
//! title bar, the client's pixels are painted inside that window's body, input
//! reaches it, and closing the window ends the process.
//!
//! The client is `weston-simple-shm` — C, libwayland-client, `wl_shm` and
//! `xdg-shell` — so nothing in this path is two halves of the same Rust code
//! agreeing with each other.
//!
//! ```text
//! LITRIC_WAYLAND_CLIENT=/path/to/weston-simple-shm \
//! LITRIC_E2E_PNG=artifacts/foreign-client-window.png \
//!   cargo test -p litric-shell --test foreign_clients
//! ```
//!
//! Without `LITRIC_WAYLAND_CLIENT` the test says so and does nothing. A test
//! that silently passes without its subject is worse than no test.

// A test's job is to stop at the first surprise and say what it saw. That is
// a panic, on purpose, so the workspace's never-unwrap and no-indexing rules
// are waived here the same way the other integration tests waive them.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use litric_design::{Theme, ThemeKind};
use litric_input::event::{InputEvent, Point, PointerButton};
use litric_input::keys::{Key, Modifiers};
use litric_locale::Regional;
use litric_shell::canvas::Canvas;
use litric_shell::shell::{DesktopMode, Shell};
use litric_shell::ui::Tokens;
use litric_shell::views;

const W: u32 = 1280;
const H: u32 = 800;

fn scratch(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "litric-shell-clients-{tag}-{}-{nanos}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("runtime directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }
    dir
}

fn client_binary() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os("LITRIC_WAYLAND_CLIENT")?);
    path.is_file().then_some(path)
}

fn desktop() -> Shell {
    let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
    let mut shell = Shell::new(tokens, W as f32, H as f32, Regional::fallback("test"));
    shell.mode = DesktopMode::Ready;
    shell
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_client(binary: &Path, runtime: &Path) -> KillOnDrop {
    let log = runtime.join("client.log");
    let out = std::fs::File::create(&log).expect("client log");
    let err = out.try_clone().expect("client log clone");
    let child = Command::new(binary)
        .env("XDG_RUNTIME_DIR", runtime)
        .env("WAYLAND_DISPLAY", "wayland-0")
        .env_remove("DISPLAY")
        .stdin(Stdio::null())
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .spawn()
        .expect("the client starts");
    KillOnDrop(child)
}

fn write_png(path: &Path, canvas: &Canvas) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let file = std::fs::File::create(path).expect("png file");
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, canvas.width(), canvas.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("png header");
    let mut bytes = Vec::with_capacity((canvas.width() * canvas.height() * 4) as usize);
    for px in canvas.pixels() {
        bytes.extend_from_slice(&[
            (px >> 16) as u8,
            (px >> 8) as u8,
            *px as u8,
            (px >> 24) as u8,
        ]);
    }
    writer.write_image_data(&bytes).expect("png data");
}

#[test]
fn a_linux_application_opens_a_window_on_the_litric_desktop() {
    let Some(binary) = client_binary() else {
        eprintln!(
            "SKIP: set LITRIC_WAYLAND_CLIENT to a wl_shm client (for example \
             weston-simple-shm) to run the desktop hosting test"
        );
        return;
    };
    let runtime = scratch("map");
    let mut shell = desktop();
    shell
        .attach_clients(&runtime)
        .expect("the desktop can listen for clients");
    assert_eq!(
        shell.clients.as_ref().map(|c| c.display_name()),
        Some("wayland-0")
    );

    let child = spawn_client(&binary, &runtime);

    // Drive the desktop the way the main loop does: tick, and let the tick
    // drain the compositor. There is no sleeping input here, so the only
    // clock is wall time.
    let start = Instant::now();
    let mut t: u64 = 1_000;
    let mut window = None;
    while start.elapsed() < Duration::from_secs(10) {
        t = t.saturating_add(16);
        shell.tick(t);
        if let Some(w) = shell.wm.all().iter().find(|w| w.app_id == "litric.client") {
            window = Some((w.id, w.frame, w.title.clone()));
            break;
        }
        std::thread::sleep(Duration::from_millis(4));
    }
    let (id, frame, title) = window.unwrap_or_else(|| {
        let log = std::fs::read_to_string(runtime.join("client.log")).unwrap_or_default();
        panic!("no client window appeared. client log:\n{log}");
    });
    assert_eq!(
        title, "simple-shm",
        "the client's own title reached the bar"
    );
    assert!(
        frame.w > 100.0 && frame.h > 100.0,
        "the window has the client's size: {}x{}",
        frame.w,
        frame.h
    );
    assert!(
        shell.client_focused().is_some(),
        "a newly mapped application has the keyboard"
    );

    // Compose a frame and check the client's pixels are in the window body.
    let mut canvas = Canvas::new(W, H, 0x101820);
    views::draw(&mut canvas, &mut shell);
    let title_h = shell.tokens.px(shell.wm.chrome().title_h);
    let body =
        litric_shell::canvas::Rect::new(frame.x, frame.y + title_h, frame.w, frame.h - title_h);
    let mut painted = 0u32;
    let mut samples = 0u32;
    let step = ((body.w / 32.0).max(1.0), (body.h / 32.0).max(1.0));
    let mut y = body.y + 4.0;
    while y < body.bottom() - 4.0 {
        let mut x = body.x + 4.0;
        while x < body.right() - 4.0 {
            #[allow(clippy::cast_possible_truncation)]
            let at = (x as i64, y as i64);
            if canvas.get(at.0, at.1).is_some_and(|p| p != 0x101820) {
                painted = painted.saturating_add(1);
            }
            samples = samples.saturating_add(1);
            x += step.0;
        }
        y += step.1;
    }
    assert_eq!(
        samples, painted,
        "every sampled pixel of the window body came from the client ({painted}/{samples})"
    );

    // The title bar must be the shell's, not the client's: a strip of the
    // window above the body belongs to LitricOS.
    let bar = litric_shell::canvas::Rect::new(frame.x, frame.y, frame.w, title_h);
    assert!(
        canvas.get(bar.x as i64 + 2, bar.y as i64 + 2)
            != canvas.get(body.x as i64 + 2, body.y as i64 + 2),
        "the title bar and the client's pixels are different surfaces"
    );

    if let Some(path) = std::env::var_os("LITRIC_E2E_PNG") {
        // `cargo test` runs an integration test from the crate directory, so the
        // documented relative path would land in `src/litric-shell/artifacts/`
        // rather than in the repository's. Resolve it from the workspace root.
        let path = Path::new(&path);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(path)
        };
        write_png(&path, &canvas);
    }

    // Input: a click inside the body, a key, and the application must still be
    // running. A malformed event makes a strict C client exit.
    let at = Point::new(body.x + body.w / 2.0, body.y + body.h / 2.0);
    let _ = shell.handle(
        &InputEvent::PointerMove {
            at,
            delta: Point::new(1.0, 1.0),
        },
        t,
    );
    let _ = shell.handle(
        &InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: true,
            at,
            clicks: 1,
            mods: Modifiers::NONE,
        },
        t,
    );
    let _ = shell.handle(
        &InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: false,
            at,
            clicks: 1,
            mods: Modifiers::NONE,
        },
        t,
    );
    let _ = shell.handle(
        &InputEvent::KeyDown {
            key: Key::Char('a'),
            code: 30,
            mods: Modifiers::NONE,
            repeat: false,
        },
        t,
    );
    let _ = shell.handle(
        &InputEvent::KeyUp {
            key: Key::Char('a'),
            code: 30,
            mods: Modifiers::NONE,
        },
        t,
    );
    let alive = shell.clients.as_ref().is_some_and(|h| h.hosts(id));
    assert!(alive, "the application survived the input");

    // Closing the window closes the application.
    assert!(shell.wm.close(id));
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        t = t.saturating_add(16);
        shell.tick(t);
        if shell.clients.as_ref().is_some_and(|h| !h.hosts(id)) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        shell.clients.as_ref().is_some_and(|h| !h.hosts(id)),
        "the surface is gone after its window closed"
    );

    drop(child);
    let _ = std::fs::remove_dir_all(&runtime);
}

#[test]
fn a_command_typed_into_the_terminal_launches_a_graphical_application() {
    let Some(binary) = client_binary() else {
        eprintln!("SKIP: no LITRIC_WAYLAND_CLIENT");
        return;
    };
    let runtime = scratch("launch");
    let mut shell = desktop();
    shell
        .attach_clients(&runtime)
        .expect("a socket to launch into");

    // Open the Terminal the way the launcher does, then type the command and
    // press Enter: the launch path a user actually has.
    let id = shell
        .wm
        .open("com.litric.terminal", "Terminal", (640.0, 420.0));
    // `open` already focuses the new window; assert what the key path needs.
    assert_eq!(shell.wm.focused(), Some(id));
    let envs = shell.application_env();
    assert!(
        envs.iter()
            .any(|(k, v)| k == "WAYLAND_DISPLAY" && v == "wayland-0"),
        "the launch environment names the socket: {envs:?}"
    );
    shell.terminal.input = binary.display().to_string();
    let _ = shell.handle(
        &InputEvent::KeyDown {
            key: Key::Enter,
            code: 28,
            mods: Modifiers::NONE,
            repeat: false,
        },
        1_000,
    );

    let start = Instant::now();
    let mut t: u64 = 1_000;
    let mut window = None;
    while start.elapsed() < Duration::from_secs(15) {
        t = t.saturating_add(16);
        shell.tick(t);
        if let Some(w) = shell.wm.all().iter().find(|w| w.app_id == "litric.client") {
            window = Some(w.id);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let window = window.unwrap_or_else(|| {
        let lines: Vec<String> = shell.terminal.lines.iter().cloned().collect();
        panic!("no window from the launched application. terminal:\n{lines:#?}");
    });
    assert!(shell.clients.as_ref().is_some_and(|h| h.hosts(window)));
    // Clean up: closing the window ends the process the Terminal started.
    assert!(shell.wm.close(window));
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        t = t.saturating_add(16);
        shell.tick(t);
        if shell.clients.as_ref().is_some_and(|h| !h.hosts(window)) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = std::fs::remove_dir_all(&runtime);
}

#[test]
fn a_desktop_without_a_runtime_directory_says_why_it_cannot_host_apps() {
    let mut shell = desktop();
    let err = shell
        .attach_clients(Path::new("not-absolute"))
        .expect_err("a relative runtime directory is refused");
    assert!(err.contains("XDG_RUNTIME_DIR"), "{err}");
    assert!(shell.clients.is_none(), "no half-started server is kept");
}
