//! Foreign applications, on the desktop.
//!
//! This is the desk between `litric-compositor` (which speaks Wayland and owns
//! nothing visual) and the shell (which owns the screen and knows nothing about
//! the protocol). It keeps one record per client surface, maps that record to a
//! window the window manager already understands, forwards the input that lands
//! on it, and paints the pixels the client committed.
//!
//! # What a client window is
//!
//! A `WindowManager` window with a title bar the shell draws, a body that is
//! entirely somebody else's pixels, and a life independent of the shell's
//! built-in applications. That is the whole point of [`Content::Client`]: the
//! window manager was written before there were clients so that it would never
//! assume there was only one kind of window, and this module is the second kind.
//!
//! [`Content::Client`]: litric_wm::Content::Client
//!
//! # Failure is a window that goes away
//!
//! Every path here treats "the client is gone" as normal: a crashed app, a
//! closed socket, a surface destroyed by a toolkit that is recreating its
//! window. The desktop loses one window and keeps running, which is the
//! property that matters when the alternative is losing the session.

use std::collections::BTreeMap;
use std::path::Path;

use litric_compositor::{Format, Frame, Server};
use litric_wm::{Rect as WmRect, WindowId};

use crate::canvas::{Canvas, Rect};
use crate::font::Style;
use crate::shell::Shell;
use crate::ui::Tokens;

/// One client surface the desktop is hosting.
#[derive(Debug)]
struct ClientSurface {
    window: Option<WindowId>,
    parent: Option<u32>,
    /// Popup offset from the parent's surface origin, in logical pixels.
    at: (i32, i32),
    title: String,
    app_id: String,
    frame: Option<Frame>,
    /// Where the body was last painted, in screen pixels. Input is mapped
    /// through this rectangle, so the pointer lands where it visually does
    /// rather than where the window manager thinks the window is.
    body: Rect,
    /// Size the client was last asked for, to avoid configuring every frame.
    asked: (u32, u32),
    last_configure_ms: u64,
}

/// Every client surface, and the server they arrived on.
pub struct Host {
    server: Server,
    surfaces: BTreeMap<u32, ClientSurface>,
    windows: BTreeMap<WindowId, u32>,
    pointer: Option<u32>,
    keyboard: Option<u32>,
    notes: Vec<String>,
}

impl std::fmt::Debug for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host")
            .field("socket", &self.server.socket_path())
            .field("surfaces", &self.surfaces.len())
            .field("windows", &self.windows.len())
            .finish_non_exhaustive()
    }
}

impl Host {
    /// Listen for clients on `<runtime_dir>/wayland-0`.
    ///
    /// The socket is created inside the desktop's own runtime directory, which
    /// is mode 0700 and owned by the session user: a Wayland socket is an
    /// unauthenticated door to the compositor, and the filesystem is the lock.
    pub fn start(runtime_dir: &Path) -> Result<Self, String> {
        let server = Server::start(runtime_dir, None).map_err(|e| e.to_string())?;
        Ok(Self {
            server,
            surfaces: BTreeMap::new(),
            windows: BTreeMap::new(),
            pointer: None,
            keyboard: None,
            notes: Vec::new(),
        })
    }

    /// The `WAYLAND_DISPLAY` an application must be given.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.server.display_name()
    }

    /// The socket path, for logs and diagnostics.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        self.server.socket_path()
    }

    /// Notes to put in the status file: what mapped, what went away, and why.
    #[must_use]
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// The environment an application needs to reach this desktop.
    ///
    /// A process started from the Terminal, or by a launcher, inherits these
    /// or it cannot connect: `WAYLAND_DISPLAY` names the socket and
    /// `XDG_RUNTIME_DIR` names the directory it lives in. Both are explicit so
    /// a launcher never depends on how the shell itself was started.
    #[must_use]
    pub fn application_env(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(dir) = self.server.socket_path().parent() {
            out.push((
                "XDG_RUNTIME_DIR".to_owned(),
                dir.to_string_lossy().into_owned(),
            ));
        }
        out.push((
            "WAYLAND_DISPLAY".to_owned(),
            self.server.display_name().to_owned(),
        ));
        out
    }

    /// The surface behind a window, if that window is a client's.
    #[must_use]
    pub fn surface_of(&self, window: WindowId) -> Option<u32> {
        self.windows.get(&window).copied()
    }

    /// Whether this window's body is somebody else's pixels.
    #[must_use]
    pub fn hosts(&self, window: WindowId) -> bool {
        self.windows.contains_key(&window)
    }

    /// The title a client set for itself, if it has one.
    #[must_use]
    pub fn title_of(&self, surface: u32) -> Option<&str> {
        self.surfaces
            .get(&surface)
            .map(|s| s.title.as_str())
            .filter(|t| !t.is_empty())
    }

    /// Take the newest frame for a surface, to be painted once.
    fn frame(&mut self, surface: u32) -> Option<Frame> {
        self.server.take_frame(surface)
    }

    /// Ask a client for a new size, at most once per 120 ms.
    ///
    /// A window being dragged changes size every frame; a client that is
    /// reconfigured every frame cannot keep up and will lag the drag by a
    /// second. Four times a second is fast enough to feel live and slow enough
    /// that the client is never chasing a size that no longer exists.
    fn resize(&mut self, surface: u32, width: u32, height: u32, now_ms: u64) {
        let Some(s) = self.surfaces.get_mut(&surface) else {
            return;
        };
        if width == 0 || height == 0 || s.asked == (width, height) {
            return;
        }
        if now_ms.saturating_sub(s.last_configure_ms) < 120 {
            return;
        }
        s.asked = (width, height);
        s.last_configure_ms = now_ms;
        self.server.configure(surface, width, height);
    }

    /// Tell the compositor which of our windows owns the keyboard.
    fn focus(&mut self, window: Option<WindowId>) {
        let surface = window.and_then(|w| self.windows.get(&w).copied());
        if self.keyboard == surface {
            return;
        }
        self.keyboard = surface;
        self.server.keyboard_focus(surface);
    }

    /// A key, by evdev code. Returns whether a client took it.
    fn key(&mut self, evdev: u32, pressed: bool, now_ms: u64) -> bool {
        if self.keyboard.is_none() || evdev == 0 {
            return false;
        }
        let time = u32::try_from(now_ms).unwrap_or(u32::MAX);
        self.server.key(evdev, pressed, time);
        true
    }

    /// Pointer motion over a surface, in surface-logical coordinates.
    fn motion(&mut self, surface: u32, x: f64, y: f64) {
        if self.pointer == Some(surface) {
            self.server.pointer_motion(surface, x, y);
        } else {
            self.pointer = Some(surface);
            self.server.pointer_enter(surface, x, y);
        }
    }

    fn leave(&mut self) {
        if self.pointer.take().is_some() {
            self.server.pointer_leave();
        }
    }

    /// Where a screen point lands inside one client window, in the client's
    /// own coordinates. Popups win over the toplevel, because a menu is drawn
    /// above the window that opened it.
    fn local(&self, window: WindowId, at: (f32, f32)) -> Option<(u32, f64, f64)> {
        let toplevel = self.windows.get(&window).copied()?;
        let popup = self
            .surfaces
            .iter()
            .filter(|(_, s)| s.parent == Some(toplevel))
            .filter_map(|(id, _)| {
                let (origin, size) = self.popup_rect(*id)?;
                let inside = at.0 >= origin.0
                    && at.1 >= origin.1
                    && at.0 < origin.0 + size.0
                    && at.1 < origin.1 + size.1;
                inside.then_some((*id, origin))
            })
            .next();
        let (surface, origin) = match popup {
            Some((id, origin)) => (id, origin),
            None => {
                let s = self.surfaces.get(&toplevel)?;
                (toplevel, (s.body.x, s.body.y))
            }
        };
        let (x, y) = self.to_surface(surface, at.0 - origin.0, at.1 - origin.1)?;
        Some((surface, x, y))
    }

    /// Convert a point inside a surface's painted rectangle into the logical
    /// coordinates the client thinks in.
    fn to_surface(&self, surface: u32, x: f32, y: f32) -> Option<(f64, f64)> {
        let s = self.surfaces.get(&surface)?;
        let frame = s.frame.as_ref()?;
        let (w, h) = self.painted_size(surface)?;
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let sx = f64::from(frame.logical_width()) / f64::from(w);
        let sy = f64::from(frame.logical_height()) / f64::from(h);
        Some((f64::from(x) * sx, f64::from(y) * sy))
    }

    fn painted_size(&self, surface: u32) -> Option<(f32, f32)> {
        let s = self.surfaces.get(&surface)?;
        if s.parent.is_some() {
            let (_, size) = self.popup_rect(surface)?;
            Some(size)
        } else {
            Some((s.body.w, s.body.h))
        }
    }

    /// Where a popup is painted: its parent's body origin plus its offset, and
    /// its own frame size (not the parent's).
    fn popup_rect(&self, surface: u32) -> Option<((f32, f32), (f32, f32))> {
        let s = self.surfaces.get(&surface)?;
        let parent = self.surfaces.get(&s.parent?)?;
        let frame = s.frame.as_ref()?;
        let origin = (parent.body.x + s.at.0 as f32, parent.body.y + s.at.1 as f32);
        Some((
            origin,
            (frame.logical_width() as f32, frame.logical_height() as f32),
        ))
    }

    /// Paint one window's body, if it is a client's.
    pub fn draw_window(&mut self, canvas: &mut Canvas, window: WindowId, body: Rect) -> bool {
        let Some(surface) = self.windows.get(&window).copied() else {
            return false;
        };
        if let Some(fresh) = self.frame(surface) {
            if let Some(s) = self.surfaces.get_mut(&surface) {
                s.frame = Some(fresh);
            }
        }
        let Some(s) = self.surfaces.get_mut(&surface) else {
            return true;
        };
        s.body = body;
        match &s.frame {
            Some(frame) => {
                let (w, h) = (frame.logical_width(), frame.logical_height());
                canvas.blit_bgra(body, &frame.pixels, w, h, frame.format == Format::Xrgb8888);
            }
            None => {
                // No frame yet: say so, rather than showing a black rectangle
                // a user would read as a broken application.
                canvas.fill_round_rect(body, 0.0, 0x101418);
                crate::font::draw(
                    canvas,
                    "waiting for the application's first frame",
                    body.x + 18.0,
                    body.y + 34.0,
                    &Style::new(13.0, crate::canvas::with_alpha(0xFFFFFF, 0.55)),
                );
            }
        }
        // The client's frame callbacks are answered now: this is the moment its
        // pixels reached the screen, which is exactly what the protocol means
        // by "a frame was presented".
        self.server.presented(surface, 0);
        true
    }

    /// Paint every popup above the windows.
    pub fn draw_popups(&mut self, canvas: &mut Canvas) {
        let ids: Vec<u32> = self
            .surfaces
            .iter()
            .filter(|(_, s)| s.parent.is_some())
            .map(|(id, _)| *id)
            .collect();
        for surface in ids {
            if let Some(fresh) = self.frame(surface) {
                if let Some(s) = self.surfaces.get_mut(&surface) {
                    s.frame = Some(fresh);
                }
            }
            let Some((origin, size)) = self.popup_rect(surface) else {
                continue;
            };
            let Some(s) = self.surfaces.get(&surface) else {
                continue;
            };
            let Some(frame) = &s.frame else { continue };
            let rect = Rect::new(origin.0, origin.1, size.0, size.1);
            canvas.blit_bgra(
                rect,
                &frame.pixels,
                frame.logical_width(),
                frame.logical_height(),
                frame.format == Format::Xrgb8888,
            );
            self.server.presented(surface, 0);
        }
    }

    /// Dismiss every popup, the way clicking outside a menu does.
    pub fn dismiss_popups(&mut self) {
        let ids: Vec<u32> = self
            .surfaces
            .iter()
            .filter(|(_, s)| s.parent.is_some())
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.server.dismiss_popup(id);
            self.surfaces.remove(&id);
        }
    }

    /// Forward a pointer press or release that landed on `window`.
    pub fn button(&mut self, window: Option<WindowId>, at: (f32, f32), button: u8, pressed: bool) {
        let landed = window.and_then(|w| self.local(w, at));
        match landed {
            Some((surface, x, y)) => {
                self.motion(surface, x, y);
                self.server.pointer_button(button_code(button), pressed);
            }
            None => self.leave(),
        }
    }

    /// Forward motion. Returns whether a client now holds the pointer.
    pub fn pointer(&mut self, window: Option<WindowId>, at: (f32, f32)) -> bool {
        match window.and_then(|w| self.local(w, at)) {
            Some((surface, x, y)) => {
                self.motion(surface, x, y);
                true
            }
            None => {
                self.leave();
                false
            }
        }
    }

    /// Forward a scroll wheel over a client.
    pub fn scroll(&mut self, window: Option<WindowId>, at: (f32, f32), dy: f32) -> bool {
        let Some((surface, x, y)) = window.and_then(|w| self.local(w, at)) else {
            return false;
        };
        self.motion(surface, x, y);
        // Wayland's vertical axis is positive downwards, like the shell's.
        self.server.pointer_axis(0.0, f64::from(dy) * 24.0);
        true
    }

    /// Forward a key that the shell did not claim.
    pub fn key_press(&mut self, evdev: u32, pressed: bool, now_ms: u64) -> bool {
        self.key(evdev, pressed, now_ms)
    }
}

impl Shell {
    /// Start the Wayland server the first time an external application needs
    /// one, and remember it.
    ///
    /// Called once at startup. Failure is not fatal: a desktop that cannot
    /// host foreign applications is still a desktop, and the status file says
    /// why it could not rather than leaving a socket that nobody serves.
    pub fn attach_clients(&mut self, runtime: &std::path::Path) -> Result<(), String> {
        if self.clients.is_some() {
            return Ok(());
        }
        let host = crate::clients::Host::start(runtime)?;
        self.clients = Some(host);
        Ok(())
    }

    /// The environment a launched application needs. Empty when the desktop
    /// is not hosting clients, which is a real state: a shell built without a
    /// compositor still runs its own applications.
    #[must_use]
    pub fn application_env(&self) -> Vec<(String, String)> {
        self.clients
            .as_ref()
            .map(crate::clients::Host::application_env)
            .unwrap_or_default()
    }

    /// Whether the focused window belongs to a client.
    #[must_use]
    pub fn client_focused(&self) -> Option<u32> {
        let focused = self.wm.focused()?;
        self.clients.as_ref().and_then(|h| h.surface_of(focused))
    }

    /// Whether a key should go to the focused client.
    ///
    /// Meta combinations stay with the shell: Win+E must open Files even while
    /// a game has focus, or the desktop becomes a window that cannot be left.
    #[must_use]
    pub fn client_takes_key(
        &self,
        key: litric_input::keys::Key,
        mods: litric_input::keys::Modifiers,
    ) -> bool {
        if mods.meta {
            return false;
        }
        if key.is_modifier() {
            // Modifier state still has to reach the client, or Ctrl-click and
            // Shift-select do nothing.
            return self.client_focused().is_some();
        }
        self.client_focused().is_some()
    }

    /// Send a key to the focused client.
    pub fn client_key(
        &mut self,
        code: u32,
        key: litric_input::keys::Key,
        pressed: bool,
        now_ms: u64,
    ) {
        let Some(host) = self.clients.as_mut() else {
            return;
        };
        let evdev = if code != 0 { code } else { evdev_for(key) };
        host.key_press(evdev, pressed, now_ms);
    }

    /// Drain compositor events into windows. Returns whether to repaint.
    pub fn pump_clients(&mut self) -> bool {
        let events = match self.clients.as_ref() {
            Some(host) => host.server.drain(),
            None => return false,
        };
        let mut dirty = false;
        for ev in events {
            match ev {
                litric_compositor::Event::Mapped {
                    surface,
                    parent,
                    x,
                    y,
                    width,
                    height,
                    title,
                    app_id,
                } => {
                    dirty = true;
                    if let Some(host) = self.clients.as_mut() {
                        let entry = host.surfaces.entry(surface).or_insert(ClientSurface {
                            window: None,
                            parent,
                            at: (0, 0),
                            title: String::new(),
                            app_id: String::new(),
                            frame: None,
                            body: Rect::new(0.0, 0.0, 0.0, 0.0),
                            asked: (0, 0),
                            last_configure_ms: 0,
                        });
                        entry.parent = parent;
                        entry.at = (x, y);
                        entry.title = title.clone();
                        entry.app_id = app_id.clone();
                    }
                    if let Some(parent) = parent {
                        // A menu is drawn over its parent, not as a window.
                        let _ = parent;
                        continue;
                    }
                    if self
                        .clients
                        .as_ref()
                        .and_then(|h| h.surface_of_window(surface))
                        .is_some()
                    {
                        continue;
                    }
                    let work = self.wm.work_area();
                    let w = (width as f32).max(240.0).min(work.w);
                    let h = (height as f32).max(160.0).min(work.h);
                    let id = self.wm.open_at(
                        "litric.client",
                        if title.is_empty() {
                            "External application"
                        } else {
                            &title
                        },
                        WmRect::new(
                            work.x + (work.w - w) / 2.0,
                            work.y + (work.h - h) / 2.0,
                            w,
                            h,
                        ),
                    );
                    if let Some(host) = self.clients.as_mut() {
                        if let Some(s) = host.surfaces.get_mut(&surface) {
                            s.window = Some(id);
                            s.asked = (width, height);
                        }
                        host.windows.insert(id, surface);
                        host.notes.push(format!(
                            "client.mapped={surface} window={id} app={} title={:?}",
                            if app_id.is_empty() { "-" } else { &app_id },
                            title
                        ));
                    }
                }
                litric_compositor::Event::Frame { surface } => {
                    if let Some(host) = self.clients.as_mut() {
                        if host.windows.values().any(|s| *s == surface)
                            || host
                                .surfaces
                                .get(&surface)
                                .is_some_and(|s| s.frame.is_some())
                        {
                            dirty = true;
                        }
                    }
                }
                litric_compositor::Event::Title { surface, title, .. } => {
                    let window = self
                        .clients
                        .as_ref()
                        .and_then(|h| h.surface_of_window(surface));
                    if let Some(id) = window {
                        if !title.is_empty() {
                            self.wm.set_title(id, &title);
                        }
                        dirty = true;
                    }
                }
                litric_compositor::Event::Gone { surface } => {
                    if let Some(host) = self.clients.as_mut() {
                        host.surfaces.remove(&surface);
                        if let Some((id, _)) = host.windows.iter().find(|(_, s)| **s == surface) {
                            let id = *id;
                            host.windows.remove(&id);
                        }
                    }
                    let window = self
                        .clients
                        .as_ref()
                        .and_then(|h| h.surface_of_window(surface));
                    if let Some(id) = window {
                        self.wm.close(id);
                    }
                    dirty = true;
                }
                litric_compositor::Event::MoveRequested { surface } => {
                    // A client-drawn title bar was dragged. The shell owns the
                    // drag; the client only asked for it.
                    if let Some(id) = self
                        .clients
                        .as_ref()
                        .and_then(|h| h.surface_of_window(surface))
                    {
                        if let Some(frame) = self.wm.window(id).map(|w| w.frame) {
                            let _ = self.wm.begin_drag(
                                litric_wm::Hit {
                                    window: id,
                                    zone: litric_wm::HitZone::TitleBar,
                                },
                                frame.x + 4.0,
                                frame.y + 4.0,
                            );
                            dirty = true;
                        }
                    }
                }
                litric_compositor::Event::ResizeRequested { surface, .. } => {
                    let _ = surface;
                }
            }
        }

        // Windows closed by the shell must close their clients, and windows
        // that no longer exist must not keep a surface alive for ever.
        if let Some(host) = self.clients.as_mut() {
            let stale: Vec<(WindowId, u32)> = host
                .windows
                .iter()
                .filter(|(w, _)| self.wm.window(**w).is_none())
                .map(|(w, s)| (*w, *s))
                .collect();
            for (window, surface) in stale {
                host.server.close(surface);
                host.windows.remove(&window);
                host.surfaces.remove(&surface);
                host.notes
                    .push(format!("client.closed={surface} window={window}"));
                dirty = true;
            }
            // A focused client is reconfigured to its window body, so a
            // maximised window is a maximised application.
            let focused = self.wm.focused();
            host.focus(focused);
            let now = self.now_ms;
            let bodies: Vec<(u32, u32, u32)> = host
                .windows
                .iter()
                .filter_map(|(w, s)| {
                    self.wm.window(*w).map(|win| {
                        // A window never configures a client to zero pixels:
                        // some toolkits interpret that as "unmap".
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let w = win.frame.w.max(1.0) as u32;
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let h = (win.frame.h - self.tokens.px(self.wm.chrome().title_h)).max(1.0)
                            as u32;
                        (*s, w, h)
                    })
                })
                .collect();
            for (surface, w, h) in bodies {
                host.resize(surface, w.max(1), h.max(1), now);
            }
            if host.notes.len() > 24 {
                let extra = host.notes.len().saturating_sub(24);
                host.notes.drain(..extra);
            }
        }
        if dirty {
            self.request_full_redraw();
        }
        dirty
    }

    /// Paint a client's popups, above every window.
    pub fn draw_client_popups(&mut self, canvas: &mut Canvas) {
        if let Some(host) = self.clients.as_mut() {
            host.draw_popups(canvas);
        }
    }
}

impl Host {
    /// The window that hosts a surface, for the shell's bookkeeping.
    fn surface_of_window(&self, surface: u32) -> Option<WindowId> {
        self.windows
            .iter()
            .find(|(_, s)| **s == surface)
            .map(|(w, _)| *w)
    }
}

/// The evdev code for a key the shell decoded, when it is one we can name.
///
/// Real keyboards already report the code, and that is used whenever it is
/// available. This table exists for the synthetic events a test drives and for
/// backends that decode without a code, so that a key does not silently do
/// nothing in a client because it arrived through a different door.
#[must_use]
pub fn evdev_for(key: litric_input::keys::Key) -> u32 {
    use litric_input::keys::Key;
    match key {
        Key::Char('a') => 30,
        Key::Char('b') => 48,
        Key::Char('c') => 46,
        Key::Char('d') => 32,
        Key::Char('e') => 18,
        Key::Char('f') => 33,
        Key::Char('g') => 34,
        Key::Char('h') => 35,
        Key::Char('i') => 23,
        Key::Char('j') => 36,
        Key::Char('k') => 37,
        Key::Char('l') => 38,
        Key::Char('m') => 50,
        Key::Char('n') => 49,
        Key::Char('o') => 24,
        Key::Char('p') => 25,
        Key::Char('q') => 16,
        Key::Char('r') => 19,
        Key::Char('s') => 31,
        Key::Char('t') => 20,
        Key::Char('u') => 22,
        Key::Char('v') => 47,
        Key::Char('w') => 17,
        Key::Char('x') => 45,
        Key::Char('y') => 21,
        Key::Char('z') => 44,
        Key::Char('1') => 2,
        Key::Char('2') => 3,
        Key::Char('3') => 4,
        Key::Char('4') => 5,
        Key::Char('5') => 6,
        Key::Char('6') => 7,
        Key::Char('7') => 8,
        Key::Char('8') => 9,
        Key::Char('9') => 10,
        Key::Char('0') => 11,
        Key::Char(' ') => 57,
        Key::Enter => 28,
        Key::Escape => 1,
        Key::Backspace => 14,
        Key::Tab => 15,
        Key::Delete => 111,
        Key::Left => 105,
        Key::Right => 106,
        Key::Up => 103,
        Key::Down => 108,
        Key::Home => 102,
        Key::End => 107,
        Key::PageUp => 104,
        Key::PageDown => 109,
        Key::Shift(_) => 42,
        Key::Ctrl(_) => 29,
        Key::Alt(_) => 56,
        Key::Meta(_) => 125,
        _ => 0,
    }
}

/// The Wayland button number for the shell's own.
const fn button_code(button: u8) -> u32 {
    match button {
        1 => 272, // left
        2 => 274, // middle
        3 => 273, // right
        other => other as u32,
    }
}

/// The default size a client window opens at when the client does not say.
#[must_use]
pub fn default_size(tokens: &Tokens) -> (f32, f32) {
    (tokens.px(760.0), tokens.px(520.0))
}
