//! The protocol side of the compositor: one `Dispatch` per interface.
//!
//! Wayland's server library asks the compositor to be a state machine per
//! object. There is no clever way around that shape — one implementation per
//! interface *is* the protocol — so this module is long by necessity, and each
//! block does one of three things: create an object with its identity, record a
//! client's intent, or refuse something the shell must never see.
//!
//! # What is advertised
//!
//! | Global | Version | Why |
//! |---|---|---|
//! | `wl_compositor` | 4 | surfaces, regions, buffer scale, damage |
//! | `wl_shm` | 1 | `ARGB8888` and `XRGB8888` only |
//! | `wl_seat` | 5 | pointer and keyboard; no touch hardware yet |
//! | `wl_output` | 4 | one output, its mode, scale and name |
//! | `xdg_wm_base` | 2 | toplevels and popups |
//!
//! Deliberately absent, and reported as such rather than pretended:
//! `wl_data_device_manager` (clipboard and drag-and-drop), `wl_subcompositor`,
//! `wp_viewporter`, `linux-dmabuf`, `presentation-time`, `xdg_decoration`.
//! A client that needs one of those does without it exactly as it would on a
//! compositor that never offered it; nothing here lies about supporting them.

use std::sync::Arc;

use wayland_protocols::xdg::shell::server::{
    xdg_popup, xdg_positioner, xdg_surface, xdg_toplevel, xdg_wm_base,
};
use wayland_server::backend::{ClientData, ClientId};
use wayland_server::protocol::{
    wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_region, wl_seat,
    wl_shm, wl_shm_pool, wl_surface, wl_touch,
};
use wayland_server::{DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource, WEnum};

use crate::{BufferRef, Event, Format, Pool, State, Surface};

/// Per-client data. The pid is not trusted for anything (the protocol says so);
/// it exists so a log line can name the process that misbehaved.
pub struct ClientInfo;

impl ClientData for ClientInfo {
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("litric-wayland-client")
    }
}

/// `wl_surface` → the compositor's own surface identity.
pub struct SurfaceData(pub u32);

/// `xdg_surface` → the `wl_surface` it belongs to.
pub struct XdgSurfaceData(pub u32);

/// `xdg_toplevel` → the `wl_surface` it belongs to.
pub struct ToplevelData(pub u32);

/// `xdg_popup` → the `wl_surface` it belongs to.
pub struct PopupData(pub u32);

/// A shared-memory pool, shared by every buffer cut from it.
pub struct PoolData(pub Arc<Pool>);

/// One buffer's geometry inside its pool.
pub struct BufferData(pub BufferRef);

/// What an `xdg_positioner` was told, reduced to what this compositor uses.
///
/// The protocol sends the positioner piece by piece across several requests,
/// and `Dispatch` only hands us `&Positioner`, so the parts that change live
/// behind a mutex rather than in a `Cell` — the resource data must be `Sync`.
#[derive(Debug, Default)]
pub struct Positioner {
    fields: std::sync::Mutex<Fields>,
}

#[derive(Debug, Clone, Copy, Default)]
struct Fields {
    anchor: (i32, i32),
    offset: (i32, i32),
}

impl Positioner {
    fn update(&self, f: impl FnOnce(&mut Fields)) {
        let mut guard = match self.fields.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        f(&mut guard);
    }

    fn position(&self) -> (i32, i32) {
        let guard = match self.fields.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        (
            guard.anchor.0.saturating_add(guard.offset.0),
            guard.anchor.1.saturating_add(guard.offset.1),
        )
    }
}

/// Advertise the globals.
pub fn register(handle: &DisplayHandle, _state: &mut State) {
    handle.create_global::<State, wl_compositor::WlCompositor, ()>(4, ());
    handle.create_global::<State, wl_shm::WlShm, ()>(1, ());
    handle.create_global::<State, wl_seat::WlSeat, ()>(5, ());
    handle.create_global::<State, wl_output::WlOutput, ()>(4, ());
    handle.create_global::<State, xdg_wm_base::XdgWmBase, ()>(2, ());
}

// ---------------------------------------------------------------------------
// wl_compositor
// ---------------------------------------------------------------------------

impl GlobalDispatch<wl_compositor::WlCompositor, ()> for State {
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &wayland_server::Client,
        resource: New<wl_compositor::WlCompositor>,
        _data: &(),
        data_init: &mut DataInit<'_, State>,
    ) {
        let _ = data_init.init::<wl_compositor::WlCompositor, ()>(resource, ());
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        _resource: &wl_compositor::WlCompositor,
        request: wl_compositor::Request,
        _data: &(),
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                let sid = state.next_surface;
                state.next_surface = state.next_surface.wrapping_add(1).max(1);
                let surface =
                    data_init.init::<wl_surface::WlSurface, SurfaceData>(id, SurfaceData(sid));
                state.surfaces.insert(sid, Surface::new(surface));
            }
            wl_compositor::Request::CreateRegion { id } => {
                let _ = data_init.init::<wl_region::WlRegion, ()>(id, ());
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// wl_region
// ---------------------------------------------------------------------------

impl Dispatch<wl_region::WlRegion, ()> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        _resource: &wl_region::WlRegion,
        request: wl_region::Request,
        _data: &(),
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        // Input and opaque regions are hints a compositor may ignore; ignoring
        // them is what every compositor does for a shaped window it cannot
        // represent in its scene graph.
        if let wl_region::Request::Destroy = request {}
    }
}

// ---------------------------------------------------------------------------
// wl_shm
// ---------------------------------------------------------------------------

impl GlobalDispatch<wl_shm::WlShm, ()> for State {
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &wayland_server::Client,
        resource: New<wl_shm::WlShm>,
        _data: &(),
        data_init: &mut DataInit<'_, State>,
    ) {
        let shm = data_init.init(resource, ());
        shm.format(wl_shm::Format::Argb8888);
        shm.format(wl_shm::Format::Xrgb8888);
    }
}

impl Dispatch<wl_shm::WlShm, ()> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        _resource: &wl_shm::WlShm,
        request: wl_shm::Request,
        _data: &(),
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        if let wl_shm::Request::CreatePool { id, fd, size } = request {
            let file = std::fs::File::from(fd);
            // The size the client declares is the size it mapped; a short
            // read later is caught by `read_frame` against the real length.
            let pool = Arc::new(Pool::new(file, u64::try_from(size).unwrap_or(0)));
            let _ = data_init.init::<wl_shm_pool::WlShmPool, PoolData>(id, PoolData(pool));
        }
    }
}

// ---------------------------------------------------------------------------
// wl_shm_pool
// ---------------------------------------------------------------------------

impl Dispatch<wl_shm_pool::WlShmPool, PoolData> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        resource: &wl_shm_pool::WlShmPool,
        request: wl_shm_pool::Request,
        data: &PoolData,
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_shm_pool::Request::CreateBuffer {
                id,
                offset,
                width,
                height,
                stride,
                format,
            } => {
                let format = match format {
                    WEnum::Value(wl_shm::Format::Argb8888) => Format::Argb8888,
                    WEnum::Value(wl_shm::Format::Xrgb8888) => Format::Xrgb8888,
                    _ => {
                        // A client must only ask for a format we advertised.
                        // The object still has to be initialised before the
                        // error travels, or the library panics on cleanup.
                        let _ = data_init.init::<wl_buffer::WlBuffer, BufferData>(
                            id,
                            BufferData(BufferRef::empty(Arc::clone(&data.0))),
                        );
                        resource.post_error(wl_shm::Error::InvalidFormat, "unsupported format");
                        return;
                    }
                };
                let Ok(offset) = u64::try_from(offset) else {
                    let _ = data_init.init::<wl_buffer::WlBuffer, BufferData>(
                        id,
                        BufferData(BufferRef::empty(Arc::clone(&data.0))),
                    );
                    resource.post_error(wl_shm::Error::InvalidStride, "negative offset");
                    return;
                };
                let (Ok(w), Ok(h)) = (u32::try_from(width), u32::try_from(height)) else {
                    let _ = data_init.init::<wl_buffer::WlBuffer, BufferData>(
                        id,
                        BufferData(BufferRef::empty(Arc::clone(&data.0))),
                    );
                    resource.post_error(wl_shm::Error::InvalidFd, "negative size");
                    return;
                };
                let Ok(stride) = u64::try_from(stride) else {
                    let _ = data_init.init::<wl_buffer::WlBuffer, BufferData>(
                        id,
                        BufferData(BufferRef::empty(Arc::clone(&data.0))),
                    );
                    resource.post_error(wl_shm::Error::InvalidStride, "negative stride");
                    return;
                };
                let _ = data_init.init::<wl_buffer::WlBuffer, BufferData>(
                    id,
                    BufferData(BufferRef {
                        pool: Arc::clone(&data.0),
                        offset,
                        width: w,
                        height: h,
                        stride,
                        format,
                    }),
                );
            }
            wl_shm_pool::Request::Resize { size } => {
                if let Ok(size) = u64::try_from(size) {
                    data.0.resize(size);
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// wl_buffer
// ---------------------------------------------------------------------------

impl Dispatch<wl_buffer::WlBuffer, BufferData> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        _resource: &wl_buffer::WlBuffer,
        request: wl_buffer::Request,
        _data: &BufferData,
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        if let wl_buffer::Request::Destroy = request {}
    }
}

// ---------------------------------------------------------------------------
// wl_callback
// ---------------------------------------------------------------------------

impl Dispatch<wl_callback::WlCallback, ()> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        _resource: &wl_callback::WlCallback,
        _request: wl_callback::Request,
        _data: &(),
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
    }
}

// ---------------------------------------------------------------------------
// wl_surface
// ---------------------------------------------------------------------------

impl Dispatch<wl_surface::WlSurface, SurfaceData> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        resource: &wl_surface::WlSurface,
        request: wl_surface::Request,
        data: &SurfaceData,
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        let id = data.0;
        match request {
            wl_surface::Request::Attach { buffer, x, y } => {
                let _ = (x, y);
                let Some(s) = state.surfaces.get_mut(&id) else {
                    return;
                };
                s.pending_buffer = buffer.and_then(|b| {
                    b.data::<BufferData>()
                        .filter(|d| d.0.is_valid())
                        .map(|d| d.0.clone())
                });
            }
            wl_surface::Request::Frame { callback } => {
                let cb = data_init.init::<wl_callback::WlCallback, ()>(callback, ());
                if let Some(s) = state.surfaces.get_mut(&id) {
                    if s.frame_callbacks.len() < 8 {
                        s.frame_callbacks.push(cb);
                    } else {
                        // A client that queues frames without ever drawing one
                        // must not grow our heap. Answer it and move on.
                        cb.done(state.now_ms());
                    }
                }
            }
            wl_surface::Request::Commit => state.commit(id),
            wl_surface::Request::SetBufferScale { scale } => {
                if let Some(s) = state.surfaces.get_mut(&id) {
                    if scale > 0 {
                        s.buffer_scale = scale;
                    }
                }
            }
            wl_surface::Request::Destroy => {
                state.remove_surface(id);
            }
            _ => {
                let _ = resource;
            }
        }
    }

    fn destroyed(
        state: &mut State,
        _client: ClientId,
        _resource: &wl_surface::WlSurface,
        data: &SurfaceData,
    ) {
        state.remove_surface(data.0);
    }
}

// ---------------------------------------------------------------------------
// wl_seat
// ---------------------------------------------------------------------------

impl GlobalDispatch<wl_seat::WlSeat, ()> for State {
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &wayland_server::Client,
        resource: New<wl_seat::WlSeat>,
        _data: &(),
        data_init: &mut DataInit<'_, State>,
    ) {
        let seat = data_init.init::<wl_seat::WlSeat, ()>(resource, ());
        seat.capabilities(wl_seat::Capability::Pointer | wl_seat::Capability::Keyboard);
        // `wl_seat.name` is version 8. Advertising five and sending it would
        // be a protocol error for every client that binds what we offer, so it
        // is not sent at all: a seat with no name is a seat, a seat with an
        // event its version does not define is a disconnected client.
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn request(
        state: &mut State,
        client: &wayland_server::Client,
        _resource: &wl_seat::WlSeat,
        request: wl_seat::Request,
        _data: &(),
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_seat::Request::GetPointer { id } => {
                let pointer = data_init.init::<wl_pointer::WlPointer, ()>(id, ());
                state.pointers.push((client.id(), pointer));
            }
            wl_seat::Request::GetKeyboard { id } => {
                let keyboard = data_init.init::<wl_keyboard::WlKeyboard, ()>(id, ());
                if let Ok(file) = state.keymap.dup() {
                    use std::os::unix::io::AsFd;
                    let fd = file.as_fd();
                    keyboard.keymap(wl_keyboard::KeymapFormat::XkbV1, fd, state.keymap.size());
                    // The protocol names a borrowed descriptor that is only
                    // duplicated at flush time, so the file must outlive this
                    // request. The thread drops it after the next flush.
                    state.keep_alive(file);
                }
                if keyboard.version() >= 4 {
                    keyboard.repeat_info(25, 250);
                }
                state.keyboards.push((client.id(), keyboard));
            }
            wl_seat::Request::GetTouch { id } => {
                // No touch capability is advertised, but a client is allowed
                // to ask; giving it an object that never sends an event is the
                // protocol-correct answer.
                let _ = data_init.init::<wl_touch::WlTouch, ()>(id, ());
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// wl_touch
// ---------------------------------------------------------------------------

impl Dispatch<wl_touch::WlTouch, ()> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        _resource: &wl_touch::WlTouch,
        _request: wl_touch::Request,
        _data: &(),
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        // No touch capability is advertised, so nothing arrives. The object
        // exists only because `wl_seat.get_touch` must be answerable.
    }
}

// ---------------------------------------------------------------------------
// wl_pointer
// ---------------------------------------------------------------------------

impl Dispatch<wl_pointer::WlPointer, ()> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        resource: &wl_pointer::WlPointer,
        request: wl_pointer::Request,
        _data: &(),
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_pointer::Request::SetCursor { .. } => {
                // The desktop draws the pointer itself, in its own style and
                // at its own size. A client-supplied cursor image is accepted
                // and ignored rather than refused, which is what lets a
                // toolkit that always sets one keep working.
            }
            wl_pointer::Request::Release => {
                state.pointers.retain(|(_, p)| p.id() != resource.id());
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// wl_keyboard
// ---------------------------------------------------------------------------

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        resource: &wl_keyboard::WlKeyboard,
        request: wl_keyboard::Request,
        _data: &(),
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        if let wl_keyboard::Request::Release = request {
            state.keyboards.retain(|(_, k)| k.id() != resource.id());
        }
    }
}

// ---------------------------------------------------------------------------
// wl_output
// ---------------------------------------------------------------------------

impl GlobalDispatch<wl_output::WlOutput, ()> for State {
    fn bind(
        state: &mut State,
        _handle: &DisplayHandle,
        _client: &wayland_server::Client,
        resource: New<wl_output::WlOutput>,
        _data: &(),
        data_init: &mut DataInit<'_, State>,
    ) {
        let output = data_init.init::<wl_output::WlOutput, ()>(resource, ());
        let (w, h, refresh) = (
            state.output.width,
            state.output.height,
            state.output.refresh_mhz,
        );
        // Every event below exists only from the version that introduced it,
        // and a client that bound version 1 must not receive it. Sending an
        // event above the object's version is a protocol error.
        let version = output.version();
        output.geometry(
            0,
            0,
            340,
            190,
            wl_output::Subpixel::Unknown,
            "LitricOS".to_owned(),
            "Litric Virtual Display".to_owned(),
            wl_output::Transform::Normal,
        );
        output.mode(wl_output::Mode::Current, w, h, refresh);
        if version >= 2 {
            output.scale(1);
            output.done();
        }
        if version >= 4 {
            output.name("LITRIC-1".to_owned());
            output.description("LitricOS desktop output".to_owned());
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        _resource: &wl_output::WlOutput,
        _request: wl_output::Request,
        _data: &(),
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
    }
}

// ---------------------------------------------------------------------------
// xdg_wm_base
// ---------------------------------------------------------------------------

impl GlobalDispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &wayland_server::Client,
        resource: New<xdg_wm_base::XdgWmBase>,
        _data: &(),
        data_init: &mut DataInit<'_, State>,
    ) {
        let _ = data_init.init::<xdg_wm_base::XdgWmBase, ()>(resource, ());
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        _resource: &xdg_wm_base::XdgWmBase,
        request: xdg_wm_base::Request,
        _data: &(),
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            xdg_wm_base::Request::CreatePositioner { id } => {
                let _ = data_init
                    .init::<xdg_positioner::XdgPositioner, Positioner>(id, Positioner::default());
            }
            xdg_wm_base::Request::GetXdgSurface { id, surface } => {
                let Some(sid) = surface.data::<SurfaceData>().map(|d| d.0) else {
                    return;
                };
                let xdg = data_init
                    .init::<xdg_surface::XdgSurface, XdgSurfaceData>(id, XdgSurfaceData(sid));
                if let Some(s) = state.surfaces.get_mut(&sid) {
                    s.xdg = Some(xdg);
                }
            }
            xdg_wm_base::Request::Pong { serial } => {
                let _ = serial;
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// xdg_positioner
// ---------------------------------------------------------------------------

impl Dispatch<xdg_positioner::XdgPositioner, Positioner> for State {
    fn request(
        _state: &mut State,
        _client: &wayland_server::Client,
        _resource: &xdg_positioner::XdgPositioner,
        request: xdg_positioner::Request,
        data: &Positioner,
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            xdg_positioner::Request::SetAnchorRect { x, y, .. } => {
                data.update(|f| f.anchor = (x, y));
            }
            xdg_positioner::Request::SetOffset { x, y } => {
                data.update(|f| f.offset = (x, y));
            }
            // Size, anchor, gravity and constraint adjustment decide where a
            // popup lands *within* its anchor rect and how it flips when it
            // would leave the output. This compositor anchors to the rect's
            // origin and does not flip; the shell clamps the result to the
            // screen instead. Recorded as a known limitation rather than
            // silently dropping a menu off the edge.
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// xdg_surface
// ---------------------------------------------------------------------------

impl Dispatch<xdg_surface::XdgSurface, XdgSurfaceData> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        resource: &xdg_surface::XdgSurface,
        request: xdg_surface::Request,
        data: &XdgSurfaceData,
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        let id = data.0;
        match request {
            xdg_surface::Request::GetToplevel { id: new_id } => {
                let top = data_init
                    .init::<xdg_toplevel::XdgToplevel, ToplevelData>(new_id, ToplevelData(id));
                if let Some(s) = state.surfaces.get_mut(&id) {
                    s.toplevel = Some(top);
                }
            }
            xdg_surface::Request::GetPopup {
                id: new_id,
                parent,
                positioner,
            } => {
                let _ = parent;
                let rect = positioner
                    .data::<Positioner>()
                    .map(Positioner::position)
                    .unwrap_or((0, 0));
                let popup = data_init.init::<xdg_popup::XdgPopup, PopupData>(new_id, PopupData(id));
                if let Some(s) = state.surfaces.get_mut(&id) {
                    s.popup = Some(popup);
                    s.parent = parent
                        .as_ref()
                        .and_then(|p| p.data::<XdgSurfaceData>().map(|d| d.0));
                    s.popup_rect = rect;
                }
            }
            xdg_surface::Request::AckConfigure { serial } => state.ack(id, serial),
            xdg_surface::Request::SetWindowGeometry { .. } => {
                // The frame is the window: LitricOS draws server-side
                // decorations, so the client's shadow margin is not a region
                // the compositor needs to know about.
            }
            xdg_surface::Request::Destroy => {
                state.remove_surface(id);
            }
            _ => {
                let _ = resource;
            }
        }
    }

    fn destroyed(
        state: &mut State,
        _client: ClientId,
        _resource: &xdg_surface::XdgSurface,
        data: &XdgSurfaceData,
    ) {
        state.remove_surface(data.0);
    }
}

// ---------------------------------------------------------------------------
// xdg_toplevel
// ---------------------------------------------------------------------------

impl Dispatch<xdg_toplevel::XdgToplevel, ToplevelData> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        _resource: &xdg_toplevel::XdgToplevel,
        request: xdg_toplevel::Request,
        data: &ToplevelData,
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        let id = data.0;
        match request {
            xdg_toplevel::Request::SetTitle { title } => {
                if let Some(s) = state.surfaces.get_mut(&id) {
                    s.title = title;
                    let (title, app_id) = (s.title.clone(), s.app_id.clone());
                    state.emit(Event::Title {
                        surface: id,
                        title,
                        app_id,
                    });
                }
            }
            xdg_toplevel::Request::SetAppId { app_id } => {
                if let Some(s) = state.surfaces.get_mut(&id) {
                    s.app_id = app_id;
                    let (title, app_id) = (s.title.clone(), s.app_id.clone());
                    state.emit(Event::Title {
                        surface: id,
                        title,
                        app_id,
                    });
                }
            }
            xdg_toplevel::Request::Move { .. } => {
                // A client-side-decorated window dragged by its own header.
                state.emit(Event::MoveRequested { surface: id });
            }
            xdg_toplevel::Request::Resize { edges, .. } => {
                let edges = match edges {
                    WEnum::Value(xdg_toplevel::ResizeEdge::Top) => 1,
                    WEnum::Value(xdg_toplevel::ResizeEdge::Bottom) => 2,
                    WEnum::Value(xdg_toplevel::ResizeEdge::Left) => 4,
                    WEnum::Value(xdg_toplevel::ResizeEdge::Right) => 8,
                    WEnum::Value(xdg_toplevel::ResizeEdge::TopLeft) => 1 | 4,
                    WEnum::Value(xdg_toplevel::ResizeEdge::TopRight) => 1 | 8,
                    WEnum::Value(xdg_toplevel::ResizeEdge::BottomLeft) => 2 | 4,
                    WEnum::Value(xdg_toplevel::ResizeEdge::BottomRight) => 2 | 8,
                    WEnum::Value(_) | WEnum::Unknown(_) => 0,
                };
                state.emit(Event::ResizeRequested { surface: id, edges });
            }
            xdg_toplevel::Request::Destroy => {
                state.unmap(id);
            }
            _ => {
                // set_maximized, set_fullscreen, set_minimized and friends are
                // requests the compositor may decline. Declining silently is
                // what keeps the window where the user put it; the shell has
                // its own title-bar controls for all of them.
            }
        }
    }

    fn destroyed(
        state: &mut State,
        _client: ClientId,
        _resource: &xdg_toplevel::XdgToplevel,
        data: &ToplevelData,
    ) {
        state.unmap(data.0);
    }
}

// ---------------------------------------------------------------------------
// xdg_popup
// ---------------------------------------------------------------------------

impl Dispatch<xdg_popup::XdgPopup, PopupData> for State {
    fn request(
        state: &mut State,
        _client: &wayland_server::Client,
        _resource: &xdg_popup::XdgPopup,
        request: xdg_popup::Request,
        data: &PopupData,
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            xdg_popup::Request::Destroy => state.remove_surface(data.0),
            xdg_popup::Request::Grab { .. } | xdg_popup::Request::Reposition { .. } => {
                // Popups are dismissed by the shell on any click outside them,
                // which is the behaviour a grab asks for. Repositioning is not
                // implemented: the popup stays where it was anchored.
            }
            _ => {}
        }
    }

    fn destroyed(
        state: &mut State,
        _client: ClientId,
        _resource: &xdg_popup::XdgPopup,
        data: &PopupData,
    ) {
        state.remove_surface(data.0);
    }
}
