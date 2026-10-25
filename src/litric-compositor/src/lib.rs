//! LitricOS Wayland server: the protocol half of the desktop.
//!
//! # What this is
//!
//! A real Wayland compositor server. It listens on `$XDG_RUNTIME_DIR/wayland-0`,
//! speaks the wire protocol through Smithay's `wayland-server` (ADR-0005),
//! advertises `wl_compositor`, `wl_shm`, `wl_seat`, `wl_output` and
//! `xdg_wm_base`, and hands the desktop a tight little packet per client
//! surface: pixels, a title, and an identity. Input travels the other way.
//!
//! It is **not** the renderer. There is no GPU path, no scene graph and no
//! window management in here: those live in `litric-shell` and `litric-wm`,
//! which own the display and the policy. This crate exists so a foreign process
//! can put pixels on that display without the shell having to trust it — every
//! buffer is bounds-checked, every size capped, and a dead client is one
//! `Gone` event rather than a dead desktop.
//!
//! # Why a thread
//!
//! `wayland-server`'s display is single-threaded and not `Send`. The shell's
//! loop is also single-threaded and blocks in `poll`. Rather than teach the
//! input stack to poll a second descriptor, this crate owns one thread with the
//! display in it, and talks to the desktop over two channels. The thread sleeps
//! in `poll(2)` on the client socket, the listening socket and a wake pipe, so
//! an idle desktop with no clients genuinely sleeps; a client commit wakes the
//! shell through [`Wake`] instead of waiting for the next frame budget.
//!
//! # Pixels
//!
//! Shared memory only, for now: `wl_shm` with `ARGB8888` and `XRGB8888`. The
//! pool file is read back with `pread` after each commit. That is not the fast
//! path — a real compositor imports the buffer as a texture and never touches
//! the pixels — and it is why [`Frame`] is documented as a copy. It is correct,
//! it needs no GPU, and it can be verified on a machine with no display, which
//! is what this stage needs.

#![forbid(unsafe_code)]

mod dispatch;
mod keymap;

use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::FileExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use wayland_server::protocol::{wl_callback, wl_keyboard, wl_pointer, wl_surface};
use wayland_server::Resource;

/// The largest buffer edge accepted from a client, in pixels.
///
/// The protocol advertises no limit; a compositor that trusts one is a
/// compositor whose heap belongs to whichever client allocates first. 8K is
/// above any panel LitricOS targets and far below "the machine swapped".
pub const MAX_EDGE: u32 = 8192;

/// The largest frame accepted from a client, in bytes.
pub const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

/// The name of the socket inside the runtime directory.
pub const SOCKET_NAME: &str = "wayland-0";

/// Something that can end the desktop's blocked wait for input.
///
/// Implemented by the shell for `litric_input::Waker`; kept as a trait here so
/// the compositor does not depend on the input stack.
pub trait Wake: Send + Sync {
    /// Wake the desktop's loop. Must never block.
    fn wake(&self);
}

/// How a client's pixels are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 32-bit little-endian ARGB, alpha used.
    Argb8888,
    /// 32-bit little-endian ARGB, alpha ignored (the client says it is opaque).
    Xrgb8888,
}

impl Format {
    /// Bytes per pixel. Both formats are four.
    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        4
    }
}

/// One client frame, copied out of the client's shared-memory buffer.
///
/// Rows are tightly packed, top-down, four bytes per pixel in memory order
/// `B, G, R, A` (little-endian `0xAARRGGBB`). `pixels.len()` is exactly
/// `width * height * 4`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Width in buffer pixels.
    pub width: u32,
    /// Height in buffer pixels.
    pub height: u32,
    /// Whether the alpha channel carries meaning.
    pub format: Format,
    /// The client's buffer scale: logical size is `width / scale`.
    pub scale: i32,
    /// The pixels.
    pub pixels: Arc<[u8]>,
}

impl Frame {
    /// Logical width, i.e. what the desktop should lay out.
    #[must_use]
    pub fn logical_width(&self) -> u32 {
        divide_by_scale(self.width, self.scale)
    }

    /// Logical height, i.e. what the desktop should lay out.
    #[must_use]
    pub fn logical_height(&self) -> u32 {
        divide_by_scale(self.height, self.scale)
    }
}

/// `value` at a buffer scale, where scale 1 (or nonsense) means unchanged.
///
/// `checked_div` rather than `/`: a client that named a scale of zero must not
/// be able to turn a paint into a panic.
fn divide_by_scale(value: u32, scale: i32) -> u32 {
    match u32::try_from(scale) {
        Ok(scale) if scale > 1 => value.checked_div(scale).unwrap_or(value),
        _ => value,
    }
}

/// What the desktop needs to know about a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A surface now has content and wants to be on screen.
    ///
    /// `parent` is `Some` for an `xdg_popup` (a menu, a tooltip, a drop-down),
    /// which must be drawn above its parent at the given offset rather than
    /// becoming a window of its own.
    Mapped {
        /// Surface identity, stable for the client's lifetime.
        surface: u32,
        /// The popup's parent, if this is a popup.
        parent: Option<u32>,
        /// Popup position relative to the parent's surface origin, in logical
        /// pixels. Zero for a toplevel.
        x: i32,
        /// Popup position relative to the parent's surface origin, in logical
        /// pixels. Zero for a toplevel.
        y: i32,
        /// Logical width of the first buffer.
        width: u32,
        /// Logical height of the first buffer.
        height: u32,
        /// `xdg_toplevel.set_title`, or empty.
        title: String,
        /// `xdg_toplevel.set_app_id`, or empty.
        app_id: String,
    },
    /// New pixels are available; read them with [`Server::take_frame`].
    Frame {
        /// Which surface.
        surface: u32,
    },
    /// The title or application id changed.
    Title {
        /// Which surface.
        surface: u32,
        /// The new title, possibly empty.
        title: String,
        /// The new application id, possibly empty.
        app_id: String,
    },
    /// The client went away, destroyed its surface, or asked to close.
    Gone {
        /// Which surface.
        surface: u32,
    },
    /// A client asked the compositor to start an interactive move (a
    /// client-side-decorated window dragged by its own header bar).
    MoveRequested {
        /// Which surface.
        surface: u32,
    },
    /// A client asked the compositor to start an interactive resize.
    ResizeRequested {
        /// Which surface.
        surface: u32,
        /// `xdg_toplevel.resize` edges bitmask: top 1, bottom 2, left 4,
        /// right 8.
        edges: u32,
    },
}

/// Why the compositor could not start.
#[derive(Debug)]
pub enum StartError {
    /// The runtime directory is missing, relative, or not a directory.
    BadRuntimeDir(PathBuf),
    /// The socket could not be bound. Usually another compositor owns it.
    Bind(String),
    /// The Wayland backend could not be initialised.
    Display(String),
    /// The worker thread died before it was ready.
    Thread(String),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRuntimeDir(path) => write!(
                f,
                "XDG_RUNTIME_DIR {} is not an absolute existing directory",
                path.display()
            ),
            Self::Bind(why) => write!(f, "cannot listen on {SOCKET_NAME}: {why}"),
            Self::Display(why) => write!(f, "cannot start the Wayland display: {why}"),
            Self::Thread(why) => write!(f, "the compositor thread did not start: {why}"),
        }
    }
}

impl std::error::Error for StartError {}

/// The desktop's handle on the Wayland server.
pub struct Server {
    socket: PathBuf,
    name: String,
    to_thread: Sender<Command>,
    from_thread: Receiver<Event>,
    frames: FrameStore,
    wake_pipe: UnixStream,
    join: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("socket", &self.socket)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl Server {
    /// Listen on `<runtime_dir>/wayland-0`.
    ///
    /// `wake` is called from the compositor thread whenever a client commits,
    /// maps or leaves, so the desktop redraws at once instead of at its next
    /// frame budget.
    pub fn start(runtime_dir: &Path, wake: Option<Arc<dyn Wake>>) -> Result<Self, StartError> {
        if !runtime_dir.is_absolute() || !runtime_dir.is_dir() {
            return Err(StartError::BadRuntimeDir(runtime_dir.to_path_buf()));
        }
        let socket = runtime_dir.join(SOCKET_NAME);
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (evt_tx, evt_rx) = mpsc::channel::<Event>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<String, StartError>>();
        let (wake_pipe, wake_reader) =
            UnixStream::pair().map_err(|e| StartError::Thread(e.to_string()))?;
        let frames: FrameStore = Arc::new(Mutex::new(BTreeMap::new()));

        let path = socket.clone();
        let store = Arc::clone(&frames);
        let handle = thread::Builder::new()
            .name("litric-wayland".to_owned())
            .spawn(move || {
                run(path, wake, evt_tx, cmd_rx, wake_reader, ready_tx, store);
            })
            .map_err(|e| StartError::Thread(e.to_string()))?;

        match ready_rx.recv() {
            Ok(Ok(name)) => Ok(Self {
                socket,
                name,
                to_thread: cmd_tx,
                from_thread: evt_rx,
                frames,
                wake_pipe,
                join: Some(handle),
            }),
            Ok(Err(why)) => {
                let _ = handle.join();
                Err(why)
            }
            Err(_) => {
                let _ = handle.join();
                Err(StartError::Thread(
                    "the worker exited before binding".to_owned(),
                ))
            }
        }
    }

    /// The socket path clients connect to.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket
    }

    /// The `WAYLAND_DISPLAY` value a client needs.
    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Everything that happened since the last call. Never blocks.
    #[must_use]
    pub fn drain(&self) -> Vec<Event> {
        let mut out = Vec::new();
        while let Ok(ev) = self.from_thread.try_recv() {
            out.push(ev);
        }
        out
    }

    /// Take the newest frame for a surface, leaving the store empty for it.
    ///
    /// The caller owns the returned copy; a later commit replaces the stored
    /// one, so a slow desktop skips frames rather than growing a queue.
    #[must_use]
    pub fn take_frame(&self, surface: u32) -> Option<Frame> {
        lock(&self.frames).remove(&surface)
    }

    /// Ask the client for a new size. It may refuse or take a frame to comply.
    pub fn configure(&self, surface: u32, width: u32, height: u32) {
        self.send(Command::Configure {
            surface,
            width,
            height,
        });
    }

    /// Ask the client to close itself, as a window manager's close button does.
    pub fn close(&self, surface: u32) {
        self.send(Command::Close { surface });
    }

    /// Tell the client that the pointer entered its surface at `x, y`.
    pub fn pointer_enter(&self, surface: u32, x: f64, y: f64) {
        self.send(Command::PointerEnter { surface, x, y });
    }

    /// Pointer motion inside the focused surface.
    pub fn pointer_motion(&self, surface: u32, x: f64, y: f64) {
        self.send(Command::PointerMotion { surface, x, y });
    }

    /// Pointer left every client surface.
    pub fn pointer_leave(&self) {
        self.send(Command::PointerLeave);
    }

    /// A pointer button, in Wayland's numbering (left 272, middle 274,
    /// right 273).
    pub fn pointer_button(&self, button: u32, pressed: bool) {
        self.send(Command::PointerButton { button, pressed });
    }

    /// A scroll step, in logical pixels of surface motion.
    pub fn pointer_axis(&self, horizontal: f64, vertical: f64) {
        self.send(Command::PointerAxis {
            horizontal,
            vertical,
        });
    }

    /// Give the keyboard to a surface (or to no client at all).
    pub fn keyboard_focus(&self, surface: Option<u32>) {
        self.send(Command::KeyboardFocus { surface });
    }

    /// One key, by evdev code (the number Linux input devices report).
    pub fn key(&self, evdev: u32, pressed: bool, time_ms: u32) {
        self.send(Command::Key {
            evdev,
            pressed,
            time_ms,
        });
    }

    /// The desktop drew this surface's newest frame; release its frame
    /// callbacks so the client may draw the next one.
    pub fn presented(&self, surface: u32, time_ms: u32) {
        self.send(Command::Presented { surface, time_ms });
    }

    /// Dismiss a popup the way a click outside it does.
    pub fn dismiss_popup(&self, surface: u32) {
        self.send(Command::DismissPopup { surface });
    }

    fn send(&self, cmd: Command) {
        if self.to_thread.send(cmd).is_ok() {
            // One byte on the wake pipe: the thread is in `poll`, not in a
            // spin, and this is what makes a command take effect at once.
            let _ = (&self.wake_pipe).write(&[1]);
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.send(Command::Shutdown);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
        let _ = std::fs::remove_file(&self.socket);
    }
}

/// Commands from the desktop to the compositor thread.
enum Command {
    Configure {
        surface: u32,
        width: u32,
        height: u32,
    },
    Close {
        surface: u32,
    },
    PointerEnter {
        surface: u32,
        x: f64,
        y: f64,
    },
    PointerMotion {
        surface: u32,
        x: f64,
        y: f64,
    },
    PointerLeave,
    PointerButton {
        button: u32,
        pressed: bool,
    },
    PointerAxis {
        horizontal: f64,
        vertical: f64,
    },
    KeyboardFocus {
        surface: Option<u32>,
    },
    Key {
        evdev: u32,
        pressed: bool,
        time_ms: u32,
    },
    Presented {
        surface: u32,
        time_ms: u32,
    },
    DismissPopup {
        surface: u32,
    },
    Shutdown,
}

type FrameStore = Arc<Mutex<BTreeMap<u32, Frame>>>;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    match m.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn run(
    socket: PathBuf,
    wake: Option<Arc<dyn Wake>>,
    events: Sender<Event>,
    commands: Receiver<Command>,
    wake_reader: UnixStream,
    ready: Sender<Result<String, StartError>>,
    frames: FrameStore,
) {
    let mut display = match wayland_server::Display::<State>::new() {
        Ok(d) => d,
        Err(e) => {
            let _ = ready.send(Err(StartError::Display(e.to_string())));
            return;
        }
    };
    let listener = match keymap::listen(&socket) {
        Ok(l) => l,
        Err(why) => {
            let _ = ready.send(Err(why));
            return;
        }
    };
    let name = socket
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| SOCKET_NAME.to_owned());

    let runtime = socket.parent().unwrap_or_else(|| Path::new("/tmp"));
    let keymap = match keymap::Keymap::load(runtime) {
        Ok(k) => Arc::new(k),
        Err(why) => {
            let _ = ready.send(Err(StartError::Display(format!("keymap: {why}"))));
            return;
        }
    };

    let mut state = State::new(events, frames, wake, keymap);
    dispatch::register(&display.handle(), &mut state);
    let _ = ready.send(Ok(name));
    drop(ready);

    let _ = wake_reader.set_nonblocking(true);
    let mut buf = [0u8; 64];
    loop {
        use std::io::Read;
        let woke = {
            use std::os::unix::io::AsFd;
            let client_fd = display.as_fd();
            let listener_fd = AsFdExt::as_fd(&listener);
            let wake_fd = AsFdExt::as_fd(&wake_reader);
            let mut fds = [
                rustix::event::PollFd::new(&client_fd, rustix::event::PollFlags::IN),
                rustix::event::PollFd::new(&listener_fd, rustix::event::PollFlags::IN),
                rustix::event::PollFd::new(&wake_fd, rustix::event::PollFlags::IN),
            ];
            match rustix::event::poll(&mut fds, state.poll_timeout().as_ref()) {
                Ok(n) => n > 0,
                Err(rustix::io::Errno::INTR) => true,
                Err(_) => break,
            }
        };
        if !woke {
            continue;
        }

        // Commands first: a click must reach the client before its next frame.
        let mut shutdown = false;
        loop {
            match commands.try_recv() {
                Ok(Command::Shutdown) => {
                    shutdown = true;
                    break;
                }
                Ok(cmd) => state.apply(cmd),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        if shutdown {
            break;
        }
        let _ = (&wake_reader).read(&mut buf);

        // A new client. `accept` is non-blocking-safe: the listener reports
        // readable only when there is somebody to take.
        while let Ok(Some(stream)) = listener.accept() {
            let _ = stream.set_nonblocking(false);
            if display
                .handle()
                .insert_client(stream, Arc::new(dispatch::ClientInfo))
                .is_err()
            {
                // The client went away between accept and insert.
                continue;
            }
        }

        if display.dispatch_clients(&mut state).is_err() {
            // A client's request was malformed. The backend disconnects it;
            // the desktop keeps running.
        }
        if display.flush_clients().is_err() {
            // Nothing to do: the socket is gone and the client will be reaped.
        }
        state.keepalive.clear();
    }
}

/// A tiny shim so both types can hand out a `BorrowedFd` without importing
/// three traits at each call site.
trait AsFdExt {
    fn as_fd(&self) -> std::os::unix::io::BorrowedFd<'_>;
}

impl AsFdExt for wayland_server::ListeningSocket {
    fn as_fd(&self) -> std::os::unix::io::BorrowedFd<'_> {
        use std::os::unix::io::AsFd;
        AsFd::as_fd(self)
    }
}

impl AsFdExt for UnixStream {
    fn as_fd(&self) -> std::os::unix::io::BorrowedFd<'_> {
        use std::os::unix::io::AsFd;
        AsFd::as_fd(self)
    }
}

impl AsFdExt for UnixListener {
    fn as_fd(&self) -> std::os::unix::io::BorrowedFd<'_> {
        use std::os::unix::io::AsFd;
        AsFd::as_fd(self)
    }
}

/// One client surface and everything the protocol attached to it.
struct Surface {
    surface: wl_surface::WlSurface,
    xdg: Option<wayland_protocols::xdg::shell::server::xdg_surface::XdgSurface>,
    toplevel: Option<wayland_protocols::xdg::shell::server::xdg_toplevel::XdgToplevel>,
    popup: Option<wayland_protocols::xdg::shell::server::xdg_popup::XdgPopup>,
    parent: Option<u32>,
    pending_buffer: Option<BufferRef>,
    buffer: Option<BufferRef>,
    /// Serial of the last configure we sent that the client has not acked.
    configure_serial: Option<u32>,
    configured: bool,
    mapped: bool,
    title: String,
    app_id: String,
    buffer_scale: i32,
    /// Popup anchor in parent coordinates.
    popup_rect: (i32, i32),
    frame_callbacks: Vec<wl_callback::WlCallback>,
}

impl Surface {
    fn new(surface: wl_surface::WlSurface) -> Self {
        Self {
            surface,
            xdg: None,
            toplevel: None,
            popup: None,
            parent: None,
            pending_buffer: None,
            buffer: None,
            configure_serial: None,
            configured: false,
            mapped: false,
            title: String::new(),
            app_id: String::new(),
            buffer_scale: 1,
            popup_rect: (0, 0),
            frame_callbacks: Vec::new(),
        }
    }
}

/// A buffer a client committed.
#[derive(Clone)]
struct BufferRef {
    pool: Arc<Pool>,
    offset: u64,
    width: u32,
    height: u32,
    stride: u64,
    format: Format,
}

impl BufferRef {
    /// A buffer that can never be drawn: used only to satisfy the library when
    /// a malformed `create_buffer` is refused with a protocol error.
    fn empty(pool: Arc<Pool>) -> Self {
        Self {
            pool,
            offset: 0,
            width: 0,
            height: 0,
            stride: 0,
            format: Format::Argb8888,
        }
    }

    /// Whether the geometry is one this compositor is willing to read.
    fn is_valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.width <= MAX_EDGE
            && self.height <= MAX_EDGE
            && self.stride >= u64::from(self.width).saturating_mul(4)
    }
}

/// A shared-memory pool, kept as a file so the pixels can be read without
/// mapping them.
struct Pool {
    file: std::fs::File,
    /// The pool can be resized by its client at any time, so the length is
    /// read on every buffer validation rather than captured once.
    size: std::sync::atomic::AtomicU64,
}

impl Pool {
    fn new(file: std::fs::File, size: u64) -> Self {
        Self {
            file,
            size: std::sync::atomic::AtomicU64::new(size),
        }
    }

    fn size(&self) -> u64 {
        self.size.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn resize(&self, size: u64) {
        self.size.store(size, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Compositor-wide state.
struct State {
    events: Sender<Event>,
    frames: FrameStore,
    wake: Option<Arc<dyn Wake>>,
    keymap: Arc<keymap::Keymap>,
    surfaces: BTreeMap<u32, Surface>,
    next_surface: u32,
    serial: u32,
    pointers: Vec<(wayland_server::backend::ClientId, wl_pointer::WlPointer)>,
    keyboards: Vec<(wayland_server::backend::ClientId, wl_keyboard::WlKeyboard)>,
    pointer_focus: Option<u32>,
    keyboard_focus: Option<u32>,
    pointer_entered: bool,
    pressed: Vec<u32>,
    /// Pending `wl_pointer.frame` support: the modifiers a client should see.
    mods: u32,
    output: Output,
    started: Instant,
    /// Descriptors named by events that have not been flushed yet.
    keepalive: Vec<std::fs::File>,
}

/// The single output this compositor pretends to have.
#[derive(Debug, Clone, Copy)]
struct Output {
    width: i32,
    height: i32,
    refresh_mhz: i32,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 800,
            refresh_mhz: 60_000,
        }
    }
}

impl State {
    fn new(
        events: Sender<Event>,
        frames: FrameStore,
        wake: Option<Arc<dyn Wake>>,
        keymap: Arc<keymap::Keymap>,
    ) -> Self {
        Self {
            events,
            frames,
            wake,
            keymap,
            surfaces: BTreeMap::new(),
            next_surface: 1,
            serial: 1,
            pointers: Vec::new(),
            keyboards: Vec::new(),
            pointer_focus: None,
            keyboard_focus: None,
            pointer_entered: false,
            pressed: Vec::new(),
            mods: 0,
            output: Output::default(),
            started: Instant::now(),
            keepalive: Vec::new(),
        }
    }

    /// Keep a descriptor alive until the events that reference it are sent.
    fn keep_alive(&mut self, file: std::fs::File) {
        if self.keepalive.len() >= 8 {
            self.keepalive.remove(0);
        }
        self.keepalive.push(file);
    }

    fn emit(&self, ev: Event) {
        let _ = self.events.send(ev);
        if let Some(wake) = &self.wake {
            wake.wake();
        }
    }

    fn now_ms(&self) -> u32 {
        u32::try_from(self.started.elapsed().as_millis()).unwrap_or(u32::MAX)
    }

    fn next_serial(&mut self) -> u32 {
        self.serial = self.serial.wrapping_add(1).max(1);
        self.serial
    }

    /// How long the thread may sleep with nothing to do.
    ///
    /// `None` is "until something happens": frame callbacks are answered when
    /// the desktop presents, which the shell drives, so there is no timer here
    /// to wake for. A compositor with a real vblank and a scene graph would
    /// return a deadline instead; this one genuinely sleeps.
    fn poll_timeout(&self) -> Option<rustix::event::Timespec> {
        None
    }

    /// Handle one command from the desktop.
    fn apply(&mut self, cmd: Command) {
        match cmd {
            Command::Configure {
                surface,
                width,
                height,
            } => self.configure(surface, width, height),
            Command::Close { surface } => {
                if let Some(top) = self
                    .surfaces
                    .get(&surface)
                    .and_then(|s| s.toplevel.as_ref())
                {
                    top.close();
                }
            }
            Command::PointerEnter { surface, x, y } => self.pointer_enter(surface, x, y),
            Command::PointerMotion { surface, x, y } => {
                if self.pointer_focus != Some(surface) {
                    self.pointer_enter(surface, x, y);
                }
                let time = self.now_ms();
                for pointer in self.focused_pointers() {
                    pointer.motion(time, x, y);
                    if pointer.version() >= 5 {
                        pointer.frame();
                    }
                }
            }
            Command::PointerLeave => self.pointer_leave(),
            Command::PointerButton { button, pressed } => {
                if self.pointer_focus.is_some() {
                    let serial = self.next_serial();
                    let time = self.now_ms();
                    let state = if pressed {
                        wl_pointer::ButtonState::Pressed
                    } else {
                        wl_pointer::ButtonState::Released
                    };
                    for pointer in self.focused_pointers() {
                        pointer.button(serial, time, button, state);
                        if pointer.version() >= 5 {
                            pointer.frame();
                        }
                    }
                }
            }
            Command::PointerAxis {
                horizontal,
                vertical,
            } => {
                if self.pointer_focus.is_some() {
                    let time = self.now_ms();
                    for pointer in self.focused_pointers() {
                        if vertical != 0.0 {
                            pointer.axis(time, wl_pointer::Axis::VerticalScroll, vertical);
                        }
                        if horizontal != 0.0 {
                            pointer.axis(time, wl_pointer::Axis::HorizontalScroll, horizontal);
                        }
                        if pointer.version() >= 5 {
                            pointer.frame();
                        }
                    }
                }
            }
            Command::KeyboardFocus { surface } => self.set_keyboard_focus(surface),
            Command::Key {
                evdev,
                pressed,
                time_ms,
            } => self.key(evdev, pressed, time_ms),
            Command::Presented { surface, time_ms } => {
                if let Some(s) = self.surfaces.get_mut(&surface) {
                    for cb in s.frame_callbacks.drain(..) {
                        cb.done(time_ms);
                    }
                }
            }
            Command::DismissPopup { surface } => {
                if let Some(popup) = self.surfaces.get(&surface).and_then(|s| s.popup.as_ref()) {
                    popup.popup_done();
                }
                self.remove_surface(surface);
            }
            Command::Shutdown => {}
        }
    }

    /// The pointer objects of the client whose surface the pointer is over.
    fn focused_pointers(&self) -> Vec<wl_pointer::WlPointer> {
        let Some(id) = self.pointer_focus else {
            return Vec::new();
        };
        let Some(s) = self.surfaces.get(&id) else {
            return Vec::new();
        };
        let client = s.surface.client().map(|c| c.id());
        self.pointers
            .iter()
            .filter(|(owner, _)| Some(owner.clone()) == client)
            .map(|(_, p)| p.clone())
            .collect()
    }

    /// The keyboard objects of the client that has keyboard focus.
    fn focused_keyboards(&self) -> Vec<wl_keyboard::WlKeyboard> {
        let Some(id) = self.keyboard_focus else {
            return Vec::new();
        };
        let Some(s) = self.surfaces.get(&id) else {
            return Vec::new();
        };
        let client = s.surface.client().map(|c| c.id());
        self.keyboards
            .iter()
            .filter(|(owner, _)| Some(owner.clone()) == client)
            .map(|(_, k)| k.clone())
            .collect()
    }

    fn pointer_enter(&mut self, surface: u32, x: f64, y: f64) {
        self.pointer_leave();
        let serial = self.next_serial();
        let Some(s) = self.surfaces.get(&surface) else {
            return;
        };
        let client = s.surface.client().map(|c| c.id());
        let target = s.surface.clone();
        for (owner, pointer) in self.pointers.clone() {
            if Some(owner) == client {
                pointer.enter(serial, &target, x, y);
            }
        }
        self.pointer_focus = Some(surface);
        self.pointer_entered = true;
    }

    fn pointer_leave(&mut self) {
        if !self.pointer_entered {
            return;
        }
        let serial = self.next_serial();
        if let Some(id) = self.pointer_focus {
            if let Some(s) = self.surfaces.get(&id) {
                let client = s.surface.client().map(|c| c.id());
                for (owner, pointer) in self.pointers.clone() {
                    if Some(owner) == client {
                        pointer.leave(serial, &s.surface);
                    }
                }
            }
        }
        self.pointer_focus = None;
        self.pointer_entered = false;
    }

    fn set_keyboard_focus(&mut self, surface: Option<u32>) {
        if self.keyboard_focus == surface {
            return;
        }
        if let Some(old) = self.keyboard_focus {
            let target = self
                .surfaces
                .get(&old)
                .map(|s| (s.surface.clone(), s.surface.client().map(|c| c.id())));
            if let Some((surface, client)) = target {
                let serial = self.next_serial();
                for (owner, keyboard) in self.keyboards.clone() {
                    if Some(owner) == client {
                        keyboard.leave(serial, &surface);
                    }
                }
            }
        }
        self.keyboard_focus = surface;
        let Some(id) = surface else {
            return;
        };
        let (target, client) = match self.surfaces.get(&id) {
            Some(s) => (s.surface.clone(), s.surface.client().map(|c| c.id())),
            None => return,
        };
        let serial = self.next_serial();
        // `wl_keyboard.enter` carries the keys already down so a client that
        // gains focus mid-chord (Alt+Tab, a click while typing) sees the same
        // modifiers the keyboard really has.
        let keys: Vec<u8> = self
            .pressed
            .iter()
            .filter_map(|k| u8::try_from(*k).ok())
            .collect();
        for (owner, keyboard) in self.keyboards.clone() {
            if Some(owner) == client {
                keyboard.enter(serial, &target, keys.clone());
                keyboard.modifiers(serial, self.mods, 0, 0, 0);
            }
        }
    }

    fn key(&mut self, evdev: u32, pressed: bool, time_ms: u32) {
        if pressed {
            if !self.pressed.contains(&evdev) {
                self.pressed.push(evdev);
            }
        } else {
            self.pressed.retain(|k| *k != evdev);
        }
        self.mods = modifier_mask(&self.pressed);
        // The protocol keycode is the evdev code plus eight, by definition.
        let protocol = match evdev.checked_add(8) {
            Some(k) => k,
            None => return,
        };
        let state = if pressed {
            wl_keyboard::KeyState::Pressed
        } else {
            wl_keyboard::KeyState::Released
        };
        let serial = self.next_serial();
        for keyboard in self.focused_keyboards() {
            keyboard.key(serial, time_ms, protocol, state);
            keyboard.modifiers(serial, self.mods, 0, 0, 0);
        }
    }

    /// Send `xdg_toplevel.configure`, remembering the serial the client must
    /// acknowledge before its next buffer is mapped.
    fn configure(&mut self, id: u32, width: u32, height: u32) {
        let Some(s) = self.surfaces.get_mut(&id) else {
            return;
        };
        let Some(xdg) = &s.xdg else {
            return;
        };
        let serial = self.serial.wrapping_add(1).max(1);
        self.serial = serial;
        if let Some(top) = &s.toplevel {
            top.configure(
                i32::try_from(width).unwrap_or(0),
                i32::try_from(height).unwrap_or(0),
                Vec::new(),
            );
        }
        xdg.configure(serial);
        s.configure_serial = Some(serial);
        s.configured = false;
    }

    /// The client acknowledged a configure.
    fn ack(&mut self, id: u32, serial: u32) {
        if let Some(s) = self.surfaces.get_mut(&id) {
            if s.configure_serial == Some(serial) {
                s.configured = true;
                s.configure_serial = None;
            }
        }
    }

    /// A client committed. Decide what the desktop needs to know.
    ///
    /// The borrow of the surface ends before anything is emitted, so an event
    /// handler can never observe a half-updated surface and a callback that
    /// writes to the desktop cannot deadlock against this state.
    fn commit(&mut self, id: u32) {
        let Some(s) = self.surfaces.get_mut(&id) else {
            return;
        };
        if let Some(buffer) = s.pending_buffer.take() {
            s.buffer = Some(buffer);
        }

        if s.xdg.is_some() && !s.configured && s.configure_serial.is_none() {
            // The first commit of an xdg surface: the client has told us what
            // it is, and now it must be told what to do. Zero means "choose",
            // which is right for the first buffer.
            let serial = self.serial.wrapping_add(1).max(1);
            self.serial = serial;
            if let Some(top) = &s.toplevel {
                top.configure(0, 0, Vec::new());
            }
            if let Some(xdg) = &s.xdg {
                xdg.configure(serial);
            }
            s.configure_serial = Some(serial);
            return;
        }

        let Some(buffer) = s.buffer.as_ref() else {
            return;
        };
        if !buffer.is_valid() {
            // A zero-sized or over-large buffer is not something to show. The
            // surface keeps its last good frame rather than flashing a hole.
            return;
        }
        let (width, height) = (buffer.width, buffer.height);
        let format = buffer.format;
        let offset = buffer.offset;
        let stride = buffer.stride;
        let pool = Arc::clone(&buffer.pool);
        let scale = s.buffer_scale;
        let frame = match read_frame(&pool, offset, width, height, stride, format, scale) {
            Ok(f) => f,
            Err(_why) => {
                // The client's buffer does not describe memory we hold. There
                // is nothing to show and nothing safe to read; tell the
                // desktop the surface is gone rather than drawing garbage.
                self.remove_surface(id);
                return;
            }
        };

        let was_mapped = s.mapped;
        s.mapped = true;
        let (title, app_id, parent, popup_rect) =
            (s.title.clone(), s.app_id.clone(), s.parent, s.popup_rect);
        let (w, h) = (frame.logical_width(), frame.logical_height());

        lock(&self.frames).insert(id, frame);
        if !was_mapped {
            self.emit(Event::Mapped {
                surface: id,
                parent,
                x: popup_rect.0,
                y: popup_rect.1,
                width: w,
                height: h,
                title,
                app_id,
            });
        }
        self.emit(Event::Frame { surface: id });
    }

    /// The client unmapped its toplevel without destroying the surface.
    ///
    /// `xdg_toplevel` may be destroyed and a new one created on the same
    /// `wl_surface`, which is how toolkits recreate a window after a state
    /// change. The surface stays; the window goes.
    fn unmap(&mut self, id: u32) {
        let Some(s) = self.surfaces.get_mut(&id) else {
            return;
        };
        s.toplevel = None;
        s.popup = None;
        s.buffer = None;
        s.pending_buffer = None;
        s.configured = false;
        s.configure_serial = None;
        let was_mapped = s.mapped;
        s.mapped = false;
        if was_mapped {
            self.emit(Event::Gone { surface: id });
        }
    }

    fn remove_surface(&mut self, id: u32) {
        let Some(s) = self.surfaces.remove(&id) else {
            return;
        };
        if s.mapped {
            self.emit(Event::Gone { surface: id });
        }
        lock(&self.frames).remove(&id);
        if self.keyboard_focus == Some(id) {
            self.keyboard_focus = None;
        }
        if self.pointer_focus == Some(id) {
            self.pointer_focus = None;
            self.pointer_entered = false;
        }
    }
}

/// The XKB modifier mask for the keys currently held.
///
/// The keymap shipped in `keymap.xkb` maps Shift to bit 0, Caps Lock to bit 1,
/// Control to bit 2, Alt to bit 3 (Mod1) and Super to bit 6 (Mod4). These are
/// the evdev codes the Linux input layer reports for each key, not the shell's
/// own key enum: a client must see the same physical key it would on any other
/// compositor.
fn modifier_mask(pressed: &[u32]) -> u32 {
    let mut mods = 0;
    for code in pressed {
        mods |= match code {
            42 | 54 => 1,    // LeftShift, RightShift
            58 => 2,         // CapsLock
            29 | 97 => 4,    // LeftCtrl, RightCtrl
            56 | 100 => 8,   // LeftAlt, RightAlt
            125 | 126 => 64, // LeftMeta, RightMeta
            _ => 0,
        };
    }
    mods
}

/// Copy a committed buffer out of the client's pool, row by row.
fn read_frame(
    pool: &Pool,
    offset: u64,
    width: u32,
    height: u32,
    stride: u64,
    format: Format,
    scale: i32,
) -> Result<Frame, String> {
    if width == 0 || height == 0 || width > MAX_EDGE || height > MAX_EDGE {
        return Err(format!("buffer {width}x{height} is outside 1..={MAX_EDGE}"));
    }
    let row = u64::from(width).saturating_mul(4);
    if stride < row {
        return Err(format!("stride {stride} is smaller than a row ({row})"));
    }
    let total = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "frame size overflowed".to_owned())?;
    let last = offset
        .checked_add(stride.saturating_mul(u64::from(height.saturating_sub(1))))
        .and_then(|v| v.checked_add(row))
        .ok_or_else(|| "buffer offset overflowed".to_owned())?;
    if last > pool.size() {
        return Err(format!(
            "buffer needs {last} bytes but the pool holds {}",
            pool.size()
        ));
    }
    let total_usize = usize::try_from(total).map_err(|_| "frame too large".to_owned())?;
    if total_usize > MAX_FRAME_BYTES {
        return Err(format!("frame of {total_usize} bytes is over the cap"));
    }
    let mut pixels = vec![0u8; total_usize];
    for y in 0..height {
        let start = usize::try_from(u64::from(y).saturating_mul(row)).unwrap_or(usize::MAX);
        let width_bytes = usize::try_from(row).unwrap_or(usize::MAX);
        let dest = pixels
            .get_mut(start..start.saturating_add(width_bytes))
            .ok_or_else(|| "row outside the frame".to_owned())?;
        let at = offset.saturating_add(u64::from(y).saturating_mul(stride));
        pool.file
            .read_exact_at(dest, at)
            .map_err(|e| format!("cannot read the client buffer at {at}: {e}"))?;
    }
    Ok(Frame {
        width,
        height,
        format,
        scale: scale.max(1),
        pixels: Arc::from(pixels),
    })
}

/// Replace the alpha byte of an `XRGB8888` frame, which clients are allowed to
/// leave at zero even though every pixel is opaque.
pub fn opaque(frame: &mut Frame) {
    if frame.format != Format::Xrgb8888 {
        return;
    }
    let mut pixels = Vec::with_capacity(frame.pixels.len());
    pixels.extend_from_slice(&frame.pixels);
    for px in pixels.chunks_exact_mut(4) {
        if let Some(a) = px.get_mut(3) {
            *a = 0xFF;
        }
    }
    frame.pixels = Arc::from(pixels);
}

/// Diagnostics for the status file.
#[must_use]
pub fn describe_error(why: &StartError) -> String {
    match why {
        StartError::BadRuntimeDir(p) => format!(
            "wayland=unavailable (XDG_RUNTIME_DIR {} is not an absolute directory)",
            p.display()
        ),
        StartError::Bind(msg) => format!("wayland=unavailable ({msg})"),
        StartError::Display(msg) => format!("wayland=unavailable ({msg})"),
        StartError::Thread(msg) => format!("wayland=unavailable ({msg})"),
    }
}

#[cfg(test)]
mod tests;
