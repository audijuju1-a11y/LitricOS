//! The window manager: z-order, focus, workspaces and interaction.
//!
//! No rendering, no input devices, no clock. It is handed positions and told
//! what happened, and it decides what that means. Everything below is therefore
//! reachable from a test, including the parts that are normally only reachable
//! by a person with a mouse: dragging, resizing against a minimum size, snapping
//! to an edge, and what happens to focus when the focused window closes.
//!
//! # Damage
//!
//! Every operation that changes what is on screen records the affected
//! rectangles. The shell drains them with [`WindowManager::take_damage`] and
//! repaints only those. That is what turns a compositor that redraws 1280x800
//! twice a second into one that draws nothing at all until something moves.

use crate::geom::{Chrome, Rect};
use crate::window::{Content, SnapEdge, Window, WindowId, WindowState};

/// Where on a window a point landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HitZone {
    /// The title bar: drag from here.
    TitleBar,
    /// The window's contents.
    Content,
    /// The close button.
    Close,
    /// The minimize button.
    Minimize,
    /// The maximize / restore button.
    Maximize,
    /// A resize grip.
    Resize(ResizeEdge),
}

/// Which edge or corner a resize grabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResizeEdge {
    /// Top.
    North,
    /// Bottom.
    South,
    /// Right.
    East,
    /// Left.
    West,
    /// Top-right.
    NorthEast,
    /// Top-left.
    NorthWest,
    /// Bottom-right.
    SouthEast,
    /// Bottom-left.
    SouthWest,
}

impl ResizeEdge {
    const fn horizontal(self) -> i8 {
        match self {
            Self::East | Self::NorthEast | Self::SouthEast => 1,
            Self::West | Self::NorthWest | Self::SouthWest => -1,
            _ => 0,
        }
    }
    const fn vertical(self) -> i8 {
        match self {
            Self::South | Self::SouthEast | Self::SouthWest => 1,
            Self::North | Self::NorthEast | Self::NorthWest => -1,
            _ => 0,
        }
    }
    /// The cursor shape this edge should show.
    #[must_use]
    pub const fn cursor(self) -> &'static str {
        match self {
            Self::North | Self::South => "ns-resize",
            Self::East | Self::West => "ew-resize",
            Self::NorthEast | Self::SouthWest => "nesw-resize",
            Self::NorthWest | Self::SouthEast => "nwse-resize",
        }
    }
}

/// A hit-test result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    /// Which window.
    pub window: WindowId,
    /// Where on it.
    pub zone: HitZone,
}

/// What the user is currently doing with the pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
enum DragKind {
    Move,
    Resize(ResizeEdge),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Drag {
    window: WindowId,
    kind: DragKind,
    grab: (f32, f32),
    origin: Rect,
    snap_hint: Option<SnapEdge>,
}

/// The window manager.
#[derive(Debug, Clone)]
pub struct WindowManager {
    chrome: Chrome,
    /// The whole display.
    screen: Rect,
    /// The display minus the top bar: where windows are allowed to be.
    work_area: Rect,
    /// Z-order, back to front. The last entry is on top.
    windows: Vec<Window>,
    /// Most-recently-focused first. What Alt+Tab walks.
    mru: Vec<WindowId>,
    focus: Option<WindowId>,
    workspaces: Vec<String>,
    active_workspace: usize,
    next_id: WindowId,
    drag: Option<Drag>,
    damage: Vec<Rect>,
    /// How close to an edge a drag has to get before it offers to snap.
    snap_trigger: f32,
}

impl WindowManager {
    /// A manager for a display of this size, with the top bar taking
    /// `top_inset` logical units.
    #[must_use]
    pub fn new(width: f32, height: f32, top_inset: f32) -> Self {
        let screen = Rect::new(0.0, 0.0, width, height);
        let work_area = Rect::new(0.0, top_inset, width, (height - top_inset).max(1.0));
        Self {
            chrome: Chrome::default(),
            screen,
            work_area,
            windows: Vec::new(),
            mru: Vec::new(),
            focus: None,
            workspaces: vec![
                "1".to_owned(),
                "2".to_owned(),
                "3".to_owned(),
                "4".to_owned(),
            ],
            active_workspace: 0,
            next_id: 1,
            drag: None,
            damage: Vec::new(),
            snap_trigger: 12.0,
        }
    }

    /// The frame metrics, so the renderer draws what the hit test expects.
    #[must_use]
    pub const fn chrome(&self) -> &Chrome {
        &self.chrome
    }

    /// The whole display.
    #[must_use]
    pub const fn screen(&self) -> Rect {
        self.screen
    }

    /// Where windows may be placed.
    #[must_use]
    pub const fn work_area(&self) -> Rect {
        self.work_area
    }

    /// Resize the display, keeping every window on it.
    pub fn set_display(&mut self, width: f32, height: f32, top_inset: f32) {
        self.screen = Rect::new(0.0, 0.0, width, height);
        self.work_area = Rect::new(0.0, top_inset, width, (height - top_inset).max(1.0));
        let work = self.work_area;
        let mut moved = Vec::new();
        for w in &mut self.windows {
            let before = w.frame;
            w.frame = match w.state {
                WindowState::Maximized => work,
                WindowState::Fullscreen => self.screen,
                WindowState::Snapped(edge) => edge.bounds(work),
                _ => w.frame.constrained_to(&work),
            };
            w.restore = w.restore.constrained_to(&work);
            if before != w.frame {
                moved.push(before);
                moved.push(w.frame);
            }
        }
        self.damage.push(self.screen);
        self.damage.extend(moved);
    }

    // ---- workspaces -------------------------------------------------------

    /// Every workspace's name.
    #[must_use]
    pub fn workspaces(&self) -> &[String] {
        &self.workspaces
    }

    /// Which workspace is showing.
    #[must_use]
    pub const fn active_workspace(&self) -> usize {
        self.active_workspace
    }

    /// The active workspace's name.
    #[must_use]
    pub fn active_workspace_name(&self) -> &str {
        self.workspaces
            .get(self.active_workspace)
            .map_or("?", String::as_str)
    }

    /// Show a workspace. Returns whether anything changed.
    pub fn set_workspace(&mut self, index: usize) -> bool {
        if index >= self.workspaces.len() || index == self.active_workspace {
            return false;
        }
        self.active_workspace = index;
        // Focus follows to the most recent window that lives here, so switching
        // workspaces does not leave the keyboard pointed at a window nobody can
        // see.
        self.focus = self.mru.iter().copied().find(|id| {
            self.window(*id)
                .is_some_and(|w| w.workspace == index && w.is_visible())
        });
        self.damage.push(self.screen);
        true
    }

    /// Move to the next or previous workspace, wrapping.
    pub fn cycle_workspace(&mut self, forward: bool) -> bool {
        let n = self.workspaces.len();
        if n == 0 {
            return false;
        }
        let next = if forward {
            self.active_workspace
                .saturating_add(1)
                .checked_rem(n)
                .unwrap_or(0)
        } else {
            self.active_workspace
                .checked_sub(1)
                .unwrap_or(n.saturating_sub(1))
        };
        self.set_workspace(next)
    }

    /// Move a window to another workspace.
    pub fn move_to_workspace(&mut self, id: WindowId, index: usize) -> bool {
        if index >= self.workspaces.len() {
            return false;
        }
        let Some(w) = self.windows.iter_mut().find(|w| w.id == id) else {
            return false;
        };
        if w.workspace == index {
            return false;
        }
        let frame = w.frame;
        w.workspace = index;
        self.damage.push(frame);
        true
    }

    // ---- windows ----------------------------------------------------------

    /// Open a window on the active workspace and focus it.
    pub fn open(&mut self, app_id: &str, title: &str, size: (f32, f32)) -> WindowId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        // Cascade, so two windows opened in a row are not exactly on top of
        // each other with the lower one unreachable.
        let step = 28.0;
        let n = f32::from((self.windows.len() % 6) as u16);
        let w = size.0.min(self.work_area.w);
        let h = size.1.min(self.work_area.h);
        let frame = Rect::new(
            self.work_area.x + 64.0 + n * step,
            self.work_area.y + 48.0 + n * step,
            w,
            h,
        )
        .constrained_to(&self.work_area);

        let mut win = Window::new(id, app_id, title, frame);
        win.workspace = self.active_workspace;
        self.damage.push(frame.inflate(24.0));
        self.windows.push(win);
        self.focus_window(id);
        id
    }

    /// Open a window at an exact position, for restoring a layout.
    pub fn open_at(&mut self, app_id: &str, title: &str, frame: Rect) -> WindowId {
        let id = self.open(app_id, title, (frame.w, frame.h));
        let work = self.work_area;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            let placed = frame.constrained_to(&work);
            w.frame = placed;
            w.restore = placed;
            self.damage.push(placed.inflate(24.0));
        }
        id
    }

    /// Close a window.
    pub fn close(&mut self, id: WindowId) -> bool {
        let Some(pos) = self.windows.iter().position(|w| w.id == id) else {
            return false;
        };
        let Some(win) = self.windows.get(pos) else {
            return false;
        };
        if !win.closable {
            return false;
        }
        let frame = win.frame;
        self.windows.remove(pos);
        self.mru.retain(|w| *w != id);
        if self.drag.map(|d| d.window) == Some(id) {
            self.drag = None;
        }
        if self.focus == Some(id) {
            // Focus the next most recent window on this workspace. Leaving
            // focus on a closed window is how keystrokes end up going nowhere.
            self.focus = self.mru.iter().copied().find(|w| {
                self.windows
                    .iter()
                    .any(|x| x.id == *w && x.workspace == self.active_workspace && x.is_visible())
            });
        }
        self.damage.push(frame.inflate(24.0));
        true
    }

    /// Every window, back to front, regardless of workspace.
    #[must_use]
    pub fn all(&self) -> &[Window] {
        &self.windows
    }

    /// One window.
    #[must_use]
    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    /// The windows to draw, back to front.
    #[must_use]
    pub fn visible(&self) -> Vec<&Window> {
        self.windows
            .iter()
            .filter(|w| w.workspace == self.active_workspace && w.is_visible())
            .collect()
    }

    /// Windows for the switcher: everything on this workspace, most recently
    /// used first, including minimized ones.
    #[must_use]
    pub fn switcher_order(&self) -> Vec<&Window> {
        let mut out: Vec<&Window> = self
            .mru
            .iter()
            .filter_map(|id| self.window(*id))
            .filter(|w| w.workspace == self.active_workspace)
            .collect();
        // Anything never focused goes on the end rather than being dropped.
        for w in &self.windows {
            if w.workspace == self.active_workspace && !out.iter().any(|x| x.id == w.id) {
                out.push(w);
            }
        }
        out
    }

    /// Which window has the keyboard.
    #[must_use]
    pub const fn focused(&self) -> Option<WindowId> {
        self.focus
    }

    /// Give a window focus and raise it.
    pub fn focus_window(&mut self, id: WindowId) -> bool {
        if self.window(id).is_none() {
            return false;
        }
        let changed = self.focus != Some(id);
        self.focus = Some(id);
        self.mru.retain(|w| *w != id);
        self.mru.insert(0, id);
        self.raise(id);
        if changed {
            if let Some(w) = self.window(id) {
                self.damage.push(w.frame.inflate(8.0));
            }
        }
        changed
    }

    /// Move a window to the top of the stack without changing focus.
    pub fn raise(&mut self, id: WindowId) -> bool {
        let Some(pos) = self.windows.iter().position(|w| w.id == id) else {
            return false;
        };
        if pos.saturating_add(1) == self.windows.len() {
            return false;
        }
        let win = self.windows.remove(pos);
        let frame = win.frame;
        self.windows.push(win);
        self.damage.push(frame.inflate(24.0));
        true
    }

    /// Focus the next window in most-recently-used order. This is Alt+Tab.
    pub fn cycle_focus(&mut self, forward: bool) -> Option<WindowId> {
        let order: Vec<WindowId> = self
            .switcher_order()
            .iter()
            .filter(|w| w.is_visible())
            .map(|w| w.id)
            .collect();
        if order.len() < 2 {
            return order.first().copied().inspect(|id| {
                self.focus_window(*id);
            });
        }
        let current = self
            .focus
            .and_then(|f| order.iter().position(|id| *id == f));
        let n = order.len();
        let next = match current {
            Some(i) if forward => i.saturating_add(1).checked_rem(n).unwrap_or(0),
            Some(i) => i.checked_sub(1).unwrap_or(n.saturating_sub(1)),
            None => 0,
        };
        let id = order.get(next).copied()?;
        self.focus_window(id);
        Some(id)
    }

    /// Focus the next window that belongs to the same application as the
    /// focused window, wrapping. This is the window-level "cycle" shortcut
    /// (`Meta+\``): the desktop-level cycle walks every window, this one stays
    /// inside the application the user is already working in.
    ///
    /// Minimized windows are included and are brought back, because a cycle
    /// that silently skips the window the user minimized is a cycle that
    /// cannot reach half of their work.
    pub fn cycle_app(&mut self, forward: bool) -> Option<WindowId> {
        let focused = self.focus?;
        let app_id = self.window(focused)?.app_id.clone();
        let order: Vec<WindowId> = self
            .windows
            .iter()
            .filter(|w| w.app_id == app_id)
            .map(|w| w.id)
            .collect();
        if order.len() < 2 {
            // Nothing to cycle: one window is already the one the user sees.
            return None;
        }
        let current = order.iter().position(|id| *id == focused);
        let n = order.len();
        let next = match current {
            Some(i) if forward => i.saturating_add(1).checked_rem(n).unwrap_or(0),
            Some(i) => i.checked_sub(1).unwrap_or(n.saturating_sub(1)),
            None => 0,
        };
        let id = order.get(next).copied()?;
        self.reveal(id);
        Some(id)
    }

    /// Bring a window to the user: switch to its workspace, undo a minimize,
    /// and focus it. This is what a click in Mission Control or a window result
    /// in the search field means, and it is one operation so no caller can
    /// forget one of the three.
    pub fn reveal(&mut self, id: WindowId) -> bool {
        let Some(w) = self.window(id) else {
            return false;
        };
        let workspace = w.workspace;
        let minimized = w.state == WindowState::Minimized;
        if workspace != self.active_workspace {
            self.set_workspace(workspace);
        }
        if minimized {
            self.unminimize(id);
        }
        self.focus_window(id);
        true
    }

    // ---- placement --------------------------------------------------------

    fn set_frame(&mut self, id: WindowId, frame: Rect) {
        let Some(w) = self.windows.iter_mut().find(|w| w.id == id) else {
            return;
        };
        let before = w.frame;
        if before == frame {
            return;
        }
        w.frame = frame;
        self.damage.push(before.inflate(24.0));
        self.damage.push(frame.inflate(24.0));
    }

    /// Maximize, or restore if already maximized.
    pub fn toggle_maximize(&mut self, id: WindowId) -> bool {
        let work = self.work_area;
        let Some(w) = self.windows.iter_mut().find(|w| w.id == id) else {
            return false;
        };
        let (target, state) = match w.state {
            WindowState::Maximized => (w.restore, WindowState::Normal),
            _ => {
                if w.state.is_free() {
                    w.restore = w.frame;
                }
                (work, WindowState::Maximized)
            }
        };
        w.state = state;
        self.set_frame(id, target);
        true
    }

    /// Snap a window to an edge.
    pub fn snap(&mut self, id: WindowId, edge: SnapEdge) -> bool {
        let work = self.work_area;
        let Some(w) = self.windows.iter_mut().find(|w| w.id == id) else {
            return false;
        };
        if w.state.is_free() {
            w.restore = w.frame;
        }
        w.state = WindowState::Snapped(edge);
        self.set_frame(id, edge.bounds(work));
        true
    }

    /// Put a window back to its free-floating size and position.
    pub fn restore(&mut self, id: WindowId) -> bool {
        let Some(w) = self.windows.iter_mut().find(|w| w.id == id) else {
            return false;
        };
        if w.state == WindowState::Normal {
            return false;
        }
        w.state = WindowState::Normal;
        let target = w.restore;
        self.set_frame(id, target);
        true
    }

    /// Hide a window without closing it.
    pub fn minimize(&mut self, id: WindowId) -> bool {
        let Some(w) = self.windows.iter_mut().find(|w| w.id == id) else {
            return false;
        };
        if w.state == WindowState::Minimized {
            return false;
        }
        let frame = w.frame;
        w.state = WindowState::Minimized;
        self.damage.push(frame.inflate(24.0));
        if self.focus == Some(id) {
            self.focus = self.mru.iter().copied().find(|other| {
                *other != id
                    && self.windows.iter().any(|x| {
                        x.id == *other && x.workspace == self.active_workspace && x.is_visible()
                    })
            });
        }
        true
    }

    /// Show a minimized window again and focus it.
    pub fn unminimize(&mut self, id: WindowId) -> bool {
        let Some(w) = self.windows.iter_mut().find(|w| w.id == id) else {
            return false;
        };
        if w.state != WindowState::Minimized {
            return false;
        }
        w.state = WindowState::Normal;
        let frame = w.restore;
        w.frame = frame;
        self.damage.push(frame.inflate(24.0));
        self.focus_window(id);
        true
    }

    /// Centre a window in the work area.
    pub fn center(&mut self, id: WindowId) -> bool {
        let work = self.work_area;
        let Some(w) = self.window(id) else {
            return false;
        };
        let frame = Rect::new(
            work.x + (work.w - w.frame.w) / 2.0,
            work.y + (work.h - w.frame.h) / 2.0,
            w.frame.w,
            w.frame.h,
        );
        self.set_frame(id, frame);
        true
    }

    // ---- hit testing ------------------------------------------------------

    /// What is at this point, topmost first.
    #[must_use]
    pub fn hit_test(&self, x: f32, y: f32) -> Option<Hit> {
        for w in self.windows.iter().rev() {
            if w.workspace != self.active_workspace || !w.is_visible() {
                continue;
            }
            if let Some(zone) = self.zone_of(w, x, y) {
                return Some(Hit { window: w.id, zone });
            }
        }
        None
    }

    fn zone_of(&self, w: &Window, x: f32, y: f32) -> Option<HitZone> {
        let grip = self.chrome.resize_grip;
        let outer = w.frame.inflate(grip);
        if !outer.contains(x, y) {
            return None;
        }

        // Resize grips first: they overlap the title bar's corners, and a user
        // aiming at the corner of a window means to resize it.
        if w.resizable && w.state.is_free() {
            let near_left = x < w.frame.x + grip;
            let near_right = x > w.frame.right() - grip;
            let near_top = y < w.frame.y + grip;
            let near_bottom = y > w.frame.bottom() - grip;
            let edge = match (near_top, near_bottom, near_left, near_right) {
                (true, _, true, _) => Some(ResizeEdge::NorthWest),
                (true, _, _, true) => Some(ResizeEdge::NorthEast),
                (_, true, true, _) => Some(ResizeEdge::SouthWest),
                (_, true, _, true) => Some(ResizeEdge::SouthEast),
                (true, _, _, _) => Some(ResizeEdge::North),
                (_, true, _, _) => Some(ResizeEdge::South),
                (_, _, true, _) => Some(ResizeEdge::West),
                (_, _, _, true) => Some(ResizeEdge::East),
                _ => None,
            };
            if let Some(e) = edge {
                return Some(HitZone::Resize(e));
            }
        }

        if !w.frame.contains(x, y) {
            return None;
        }

        let title = self.chrome.title_rect(&w.frame);
        if title.contains(x, y) {
            for (n, zone) in [
                (0u32, HitZone::Close),
                (1, HitZone::Minimize),
                (2, HitZone::Maximize),
            ] {
                // Buttons get a slightly larger target than they are drawn:
                // a 12-pixel circle is a hard thing to hit and an easy thing to
                // miss by one pixel.
                if self
                    .chrome
                    .button_rect(&w.frame, n)
                    .inflate(4.0)
                    .contains(x, y)
                {
                    return Some(zone);
                }
            }
            return Some(HitZone::TitleBar);
        }
        Some(HitZone::Content)
    }

    // ---- dragging ---------------------------------------------------------

    /// Begin a move or resize from a hit.
    ///
    /// Returns false for a hit that is not draggable, so the caller can treat
    /// the click as a plain click rather than the start of a gesture.
    pub fn begin_drag(&mut self, hit: Hit, x: f32, y: f32) -> bool {
        let Some(w) = self.window(hit.window) else {
            return false;
        };
        let kind = match hit.zone {
            HitZone::TitleBar if w.state.is_free() || w.state == WindowState::Maximized => {
                DragKind::Move
            }
            HitZone::Resize(e) if w.resizable && w.state.is_free() => DragKind::Resize(e),
            _ => return false,
        };
        self.drag = Some(Drag {
            window: hit.window,
            kind,
            grab: (x, y),
            origin: w.frame,
            snap_hint: None,
        });
        self.focus_window(hit.window);
        true
    }

    /// Continue a drag. Returns the snap that would be applied on release.
    pub fn update_drag(&mut self, x: f32, y: f32) -> Option<SnapEdge> {
        let mut drag = self.drag?;
        let dx = x - drag.grab.0;
        let dy = y - drag.grab.1;

        match drag.kind {
            DragKind::Move => {
                // Dragging a maximized window unmaximizes it and puts it under
                // the cursor, which is what every desktop does and what a user
                // means by the gesture.
                if self.window(drag.window).map(|w| w.state) == Some(WindowState::Maximized) {
                    let restore = self.window(drag.window).map_or(Rect::ZERO, |w| w.restore);
                    let grab_ratio = if drag.origin.w > 0.0 {
                        (drag.grab.0 - drag.origin.x) / drag.origin.w
                    } else {
                        0.5
                    };
                    let new_origin = Rect::new(
                        x - restore.w * grab_ratio,
                        y - self.chrome.title_h / 2.0,
                        restore.w,
                        restore.h,
                    );
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == drag.window) {
                        w.state = WindowState::Normal;
                    }
                    drag.origin = new_origin;
                    drag.grab = (x, y);
                    self.drag = Some(drag);
                    self.set_frame(drag.window, new_origin.constrained_to(&self.work_area));
                    return None;
                }

                let moved = drag.origin.offset(dx, dy);
                // A window may go off the sides and the bottom but never above
                // the top bar: a title bar you cannot reach is a window you
                // cannot move back.
                let clamped = Rect::new(
                    moved.x.clamp(
                        self.work_area.x - moved.w + 80.0,
                        self.work_area.right() - 80.0,
                    ),
                    moved.y.clamp(
                        self.work_area.y,
                        self.work_area.bottom() - self.chrome.title_h,
                    ),
                    moved.w,
                    moved.h,
                );
                self.set_frame(drag.window, clamped);
                drag.snap_hint = self.snap_hint_at(x, y);
                self.drag = Some(drag);
                drag.snap_hint
            }
            DragKind::Resize(edge) => {
                let w = self.window(drag.window)?;
                let (min_w, min_h) = w.min_size;
                let mut r = drag.origin;
                match edge.horizontal() {
                    1 => r.w = (drag.origin.w + dx).max(min_w),
                    -1 => {
                        let new_w = (drag.origin.w - dx).max(min_w);
                        r.x = drag.origin.right() - new_w;
                        r.w = new_w;
                    }
                    _ => {}
                }
                match edge.vertical() {
                    1 => r.h = (drag.origin.h + dy).max(min_h),
                    -1 => {
                        let new_h = (drag.origin.h - dy).max(min_h);
                        r.y = (drag.origin.bottom() - new_h).max(self.work_area.y);
                        r.h = drag.origin.bottom() - r.y;
                    }
                    _ => {}
                }
                let placed = Rect::new(
                    r.x,
                    r.y.max(self.work_area.y),
                    r.w,
                    r.h.min(self.work_area.bottom() - r.y.max(self.work_area.y)),
                );
                self.set_frame(drag.window, placed);
                if let Some(win) = self.windows.iter_mut().find(|w| w.id == drag.window) {
                    win.restore = placed;
                }
                None
            }
        }
    }

    /// What a drag ending here would snap to.
    #[must_use]
    fn snap_hint_at(&self, x: f32, y: f32) -> Option<SnapEdge> {
        let w = self.work_area;
        let t = self.snap_trigger;
        let left = x <= w.x + t;
        let right = x >= w.right() - t;
        let top = y <= w.y + t;
        let bottom = y >= w.bottom() - t;
        match (top, bottom, left, right) {
            (true, _, true, _) => Some(SnapEdge::TopLeft),
            (true, _, _, true) => Some(SnapEdge::TopRight),
            (_, true, true, _) => Some(SnapEdge::BottomLeft),
            (_, true, _, true) => Some(SnapEdge::BottomRight),
            (_, _, true, _) => Some(SnapEdge::Left),
            (_, _, _, true) => Some(SnapEdge::Right),
            // The top edge maximizes, which is the one snap gesture everybody
            // already knows.
            (true, _, _, _) => None,
            _ => None,
        }
    }

    /// Finish a drag, applying any snap it was offering.
    pub fn end_drag(&mut self, x: f32, y: f32) -> Option<SnapEdge> {
        let drag = self.drag.take()?;
        if drag.kind != DragKind::Move {
            return None;
        }
        let w = self.work_area;
        if y <= w.y + self.snap_trigger {
            if let Some(edge) = self.snap_hint_at(x, y) {
                self.snap(drag.window, edge);
                return Some(edge);
            }
            self.toggle_maximize(drag.window);
            return None;
        }
        if let Some(edge) = self.snap_hint_at(x, y) {
            self.snap(drag.window, edge);
            return Some(edge);
        }
        None
    }

    /// Whether a drag is in progress.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Cancel a drag and put the window back.
    pub fn cancel_drag(&mut self) -> bool {
        let Some(drag) = self.drag.take() else {
            return false;
        };
        self.set_frame(drag.window, drag.origin);
        true
    }

    // ---- damage -----------------------------------------------------------

    /// Mark an area as needing a repaint.
    pub fn damage(&mut self, r: Rect) {
        self.damage.push(r);
    }

    /// Take everything that needs repainting since the last call.
    pub fn take_damage(&mut self) -> Vec<Rect> {
        core::mem::take(&mut self.damage)
    }

    /// Whether anything needs repainting.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        !self.damage.is_empty()
    }

    /// Lines describing every window, for the diagnostic overlay.
    #[must_use]
    pub fn diagnostics(&self) -> Vec<String> {
        let mut out = vec![
            format!("windows={}", self.windows.len()),
            format!(
                "workspace={}/{}",
                self.active_workspace.saturating_add(1),
                self.workspaces.len()
            ),
            format!(
                "focus={}",
                self.focus
                    .map_or_else(|| "none".to_owned(), |f| format!("#{f}"))
            ),
        ];
        for w in &self.windows {
            out.push(format!("window={}", w.summary()));
        }
        out
    }

    /// Whether a window's content is a placeholder for a client that does not
    /// exist. Used by the renderer to say so on screen rather than draw an
    /// empty box that looks like a bug.
    #[must_use]
    pub fn is_placeholder(&self, id: WindowId) -> bool {
        self.window(id)
            .is_some_and(|w| matches!(w.content, Content::Client { .. }))
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn wm() -> WindowManager {
        WindowManager::new(1280.0, 800.0, 34.0)
    }

    #[test]
    fn the_work_area_starts_below_the_top_bar() {
        let m = wm();
        assert_eq!(m.work_area(), Rect::new(0.0, 34.0, 1280.0, 766.0));
    }

    #[test]
    fn opening_windows_cascades_them_so_none_is_hidden_exactly_behind_another() {
        let mut m = wm();
        let a = m.open("app.a", "A", (400.0, 300.0));
        let b = m.open("app.b", "B", (400.0, 300.0));
        let fa = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let fb = m.window(b).map(|w| w.frame).unwrap_or(Rect::ZERO);
        assert_ne!(fa, fb, "two windows opened at the same place");
    }

    #[test]
    fn a_new_window_takes_focus_and_the_top_of_the_stack() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        let b = m.open("app.b", "B", (300.0, 200.0));
        assert_eq!(m.focused(), Some(b));
        assert_eq!(m.all().last().map(|w| w.id), Some(b), "b is not on top");
        assert_eq!(m.all().first().map(|w| w.id), Some(a));
    }

    #[test]
    fn closing_the_focused_window_moves_focus_to_the_next_most_recent() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        let b = m.open("app.b", "B", (300.0, 200.0));
        assert_eq!(m.focused(), Some(b));
        assert!(m.close(b));
        assert_eq!(m.focused(), Some(a), "focus was left on a closed window");
    }

    #[test]
    fn closing_the_last_window_leaves_no_focus_rather_than_a_dangling_one() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        assert!(m.close(a));
        assert_eq!(m.focused(), None);
        assert!(m.visible().is_empty());
    }

    #[test]
    fn window_ids_are_never_reused() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        m.close(a);
        let b = m.open("app.b", "B", (300.0, 200.0));
        assert_ne!(a, b, "a stale reference would now hit a different window");
        assert!(m.window(a).is_none());
    }

    #[test]
    fn raising_reorders_without_changing_focus() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        let b = m.open("app.b", "B", (300.0, 200.0));
        assert!(m.raise(a));
        assert_eq!(m.all().last().map(|w| w.id), Some(a));
        assert_eq!(m.focused(), Some(b), "raise must not steal focus");
    }

    #[test]
    fn alt_tab_walks_most_recently_used_order_and_comes_back() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        let b = m.open("app.b", "B", (300.0, 200.0));
        let c = m.open("app.c", "C", (300.0, 200.0));
        assert_eq!(m.focused(), Some(c));
        assert_eq!(
            m.cycle_focus(true),
            Some(b),
            "first Alt+Tab goes to the last used"
        );
        assert_eq!(m.cycle_focus(true), Some(c));
        let _ = (a, m.cycle_focus(false));
    }

    #[test]
    fn alt_tab_with_one_window_stays_on_it() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        assert_eq!(m.cycle_focus(true), Some(a));
        assert_eq!(m.focused(), Some(a));
    }

    #[test]
    fn alt_tab_with_no_windows_does_nothing() {
        let mut m = wm();
        assert_eq!(m.cycle_focus(true), None);
        assert_eq!(m.focused(), None);
    }

    #[test]
    fn maximize_fills_the_work_area_and_restore_puts_it_back() {
        let mut m = wm();
        let a = m.open("app.a", "A", (400.0, 300.0));
        let before = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        m.toggle_maximize(a);
        assert_eq!(m.window(a).map(|w| w.frame), Some(m.work_area()));
        m.toggle_maximize(a);
        assert_eq!(
            m.window(a).map(|w| w.frame),
            Some(before),
            "restore lost the position"
        );
    }

    #[test]
    fn snapping_left_then_restoring_returns_to_the_original_size() {
        let mut m = wm();
        let a = m.open("app.a", "A", (400.0, 300.0));
        let before = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        assert!(m.snap(a, SnapEdge::Left));
        assert_eq!(
            m.window(a).map(|w| w.frame),
            Some(SnapEdge::Left.bounds(m.work_area()))
        );
        assert!(m.restore(a));
        assert_eq!(m.window(a).map(|w| w.frame), Some(before));
    }

    #[test]
    fn minimizing_hides_a_window_but_keeps_it_in_the_switcher() {
        let mut m = wm();
        let a = m.open("app.a", "A", (300.0, 200.0));
        let b = m.open("app.b", "B", (300.0, 200.0));
        assert!(m.minimize(b));
        assert!(!m.visible().iter().any(|w| w.id == b));
        assert!(m.switcher_order().iter().any(|w| w.id == b));
        assert_eq!(m.focused(), Some(a), "focus stayed on a hidden window");
        assert!(m.unminimize(b));
        assert!(m.visible().iter().any(|w| w.id == b));
        assert_eq!(m.focused(), Some(b));
    }

    #[test]
    fn the_title_bar_is_hit_where_it_is_drawn() {
        let mut m = wm();
        let a = m.open("app.a", "A", (400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let title = m.chrome().title_rect(&f);
        let (cx, cy) = title.center();
        assert_eq!(
            m.hit_test(cx, cy),
            Some(Hit {
                window: a,
                zone: HitZone::TitleBar
            })
        );
    }

    #[test]
    fn the_close_button_is_hit_where_it_is_drawn() {
        // The bug this prevents: a close button that needs a click two pixels
        // above it, because the hit test and the renderer disagreed.
        let mut m = wm();
        let a = m.open("app.a", "A", (400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let b = m.chrome().button_rect(&f, 0);
        let (cx, cy) = b.center();
        assert_eq!(m.hit_test(cx, cy).map(|h| h.zone), Some(HitZone::Close));
        let b2 = m.chrome().button_rect(&f, 2);
        let (cx2, cy2) = b2.center();
        assert_eq!(
            m.hit_test(cx2, cy2).map(|h| h.zone),
            Some(HitZone::Maximize)
        );
    }

    #[test]
    fn corners_resize_and_the_middle_does_not() {
        let mut m = wm();
        let a = m.open("app.a", "A", (400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        assert_eq!(
            m.hit_test(f.x + 1.0, f.y + 1.0).map(|h| h.zone),
            Some(HitZone::Resize(ResizeEdge::NorthWest))
        );
        assert_eq!(
            m.hit_test(f.right() - 1.0, f.bottom() - 1.0)
                .map(|h| h.zone),
            Some(HitZone::Resize(ResizeEdge::SouthEast))
        );
        let (cx, cy) = f.center();
        assert_eq!(m.hit_test(cx, cy).map(|h| h.zone), Some(HitZone::Content));
    }

    #[test]
    fn hit_testing_finds_the_topmost_window() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(100.0, 100.0, 300.0, 300.0));
        let b = m.open_at("app.b", "B", Rect::new(150.0, 150.0, 300.0, 300.0));
        assert_eq!(m.hit_test(200.0, 200.0).map(|h| h.window), Some(b));
        m.raise(a);
        assert_eq!(m.hit_test(200.0, 200.0).map(|h| h.window), Some(a));
    }

    #[test]
    fn hit_testing_empty_desktop_finds_nothing() {
        let m = wm();
        assert_eq!(m.hit_test(400.0, 400.0), None);
    }

    #[test]
    fn a_window_on_another_workspace_is_neither_drawn_nor_hit() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(100.0, 100.0, 300.0, 300.0));
        assert!(m.move_to_workspace(a, 1));
        assert!(m.visible().is_empty());
        assert_eq!(m.hit_test(200.0, 200.0), None);
        assert!(m.set_workspace(1));
        assert_eq!(m.hit_test(200.0, 200.0).map(|h| h.window), Some(a));
    }

    #[test]
    fn workspaces_wrap_in_both_directions() {
        let mut m = wm();
        assert_eq!(m.active_workspace(), 0);
        assert!(m.cycle_workspace(false));
        assert_eq!(m.active_workspace(), 3, "wrapping backwards from the first");
        assert!(m.cycle_workspace(true));
        assert_eq!(m.active_workspace(), 0, "wrapping forwards from the last");
    }

    #[test]
    fn switching_to_the_workspace_already_showing_changes_nothing() {
        let mut m = wm();
        assert!(!m.set_workspace(0));
        assert!(!m.set_workspace(99), "out of range must be refused");
    }

    #[test]
    fn dragging_a_title_bar_moves_the_window_by_the_pointer_delta() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(200.0, 200.0, 400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let (cx, cy) = m.chrome().title_rect(&f).center();
        let hit = m.hit_test(cx, cy).expect("title bar");
        assert!(m.begin_drag(hit, cx, cy));
        m.update_drag(cx + 60.0, cy + 40.0);
        assert_eq!(
            m.window(a).map(|w| w.frame.x),
            Some(260.0),
            "window did not follow the pointer"
        );
        assert_eq!(m.window(a).map(|w| w.frame.y), Some(240.0));
        m.end_drag(cx + 60.0, cy + 40.0);
        assert!(!m.is_dragging());
    }

    #[test]
    fn a_drag_can_never_put_the_title_bar_above_the_top_bar() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(200.0, 200.0, 400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let (cx, cy) = m.chrome().title_rect(&f).center();
        let hit = m.hit_test(cx, cy).expect("title bar");
        m.begin_drag(hit, cx, cy);
        m.update_drag(cx, -500.0);
        let y = m.window(a).map(|w| w.frame.y).unwrap_or(0.0);
        assert!(
            y >= m.work_area().y,
            "title bar went under the top bar at y={y}"
        );
    }

    #[test]
    fn dragging_to_the_left_edge_snaps_to_the_left_half() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(300.0, 300.0, 400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let (cx, cy) = m.chrome().title_rect(&f).center();
        let hit = m.hit_test(cx, cy).expect("title bar");
        m.begin_drag(hit, cx, cy);
        assert_eq!(
            m.update_drag(2.0, 400.0),
            Some(SnapEdge::Left),
            "no snap offered"
        );
        assert_eq!(m.end_drag(2.0, 400.0), Some(SnapEdge::Left));
        assert_eq!(
            m.window(a).map(|w| w.frame),
            Some(SnapEdge::Left.bounds(m.work_area()))
        );
    }

    #[test]
    fn resizing_from_the_south_east_grows_the_window() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(200.0, 200.0, 400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let hit = m
            .hit_test(f.right() - 1.0, f.bottom() - 1.0)
            .expect("corner");
        assert!(m.begin_drag(hit, f.right(), f.bottom()));
        m.update_drag(f.right() + 50.0, f.bottom() + 30.0);
        assert_eq!(m.window(a).map(|w| w.frame.w), Some(450.0));
        assert_eq!(m.window(a).map(|w| w.frame.h), Some(330.0));
    }

    #[test]
    fn resizing_stops_at_the_minimum_size() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(200.0, 200.0, 400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let hit = m
            .hit_test(f.right() - 1.0, f.bottom() - 1.0)
            .expect("corner");
        m.begin_drag(hit, f.right(), f.bottom());
        m.update_drag(f.x, f.y);
        let (mw, mh) = m.window(a).map(|w| w.min_size).unwrap_or((0.0, 0.0));
        assert_eq!(m.window(a).map(|w| w.frame.w), Some(mw));
        assert_eq!(m.window(a).map(|w| w.frame.h), Some(mh));
    }

    #[test]
    fn resizing_from_the_west_moves_the_left_edge_and_keeps_the_right_one() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(300.0, 300.0, 400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let right_before = f.right();
        let hit = m.hit_test(f.x + 1.0, f.center().1).expect("west edge");
        m.begin_drag(hit, f.x, f.center().1);
        m.update_drag(f.x - 60.0, f.center().1);
        let now = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        assert!(
            (now.right() - right_before).abs() < 1e-3,
            "the right edge moved"
        );
        assert_eq!(now.x, 240.0);
    }

    #[test]
    fn a_cancelled_drag_puts_the_window_back() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(200.0, 200.0, 400.0, 300.0));
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        let (cx, cy) = m.chrome().title_rect(&f).center();
        let hit = m.hit_test(cx, cy).expect("title bar");
        m.begin_drag(hit, cx, cy);
        m.update_drag(cx + 200.0, cy + 100.0);
        assert!(m.cancel_drag());
        assert_eq!(m.window(a).map(|w| w.frame), Some(f));
        assert!(!m.is_dragging());
    }

    #[test]
    fn a_maximized_window_cannot_be_resized_but_can_be_dragged_free() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(200.0, 200.0, 400.0, 300.0));
        m.toggle_maximize(a);
        let f = m.window(a).map(|w| w.frame).unwrap_or(Rect::ZERO);
        assert_eq!(
            m.hit_test(f.right() - 1.0, f.bottom() - 1.0)
                .map(|h| h.zone),
            Some(HitZone::Content),
            "a maximized window offered a resize grip"
        );
        let (cx, cy) = m.chrome().title_rect(&f).center();
        let hit = m.hit_test(cx, cy).expect("title bar");
        assert!(m.begin_drag(hit, cx, cy));
        m.update_drag(cx, cy + 60.0);
        assert_eq!(
            m.window(a).map(|w| w.state),
            Some(WindowState::Normal),
            "dragging a maximized window should free it"
        );
    }

    #[test]
    fn every_change_records_damage_and_taking_it_clears_it() {
        let mut m = wm();
        let _ = m.take_damage();
        let a = m.open("app.a", "A", (300.0, 200.0));
        assert!(m.is_dirty());
        let d = m.take_damage();
        assert!(!d.is_empty());
        assert!(!m.is_dirty(), "damage was not cleared");

        m.toggle_maximize(a);
        let d2 = m.take_damage();
        assert!(
            d2.len() >= 2,
            "a move should damage where it was and where it is"
        );
    }

    #[test]
    fn nothing_happening_produces_no_damage() {
        let mut m = wm();
        m.open("app.a", "A", (300.0, 200.0));
        let _ = m.take_damage();
        m.raise(m.focused().unwrap_or(0));
        m.set_workspace(0);
        assert!(!m.is_dirty(), "an idle desktop asked for a repaint");
    }

    #[test]
    fn resizing_the_display_keeps_every_window_reachable() {
        let mut m = wm();
        let a = m.open_at("app.a", "A", Rect::new(1000.0, 600.0, 400.0, 300.0));
        m.toggle_maximize(a);
        let b = m.open_at("app.b", "B", Rect::new(1100.0, 700.0, 300.0, 200.0));
        m.set_display(800.0, 600.0, 34.0);
        assert_eq!(m.window(a).map(|w| w.frame), Some(m.work_area()));
        let fb = m.window(b).map(|w| w.frame).unwrap_or(Rect::ZERO);
        assert!(
            fb.right() <= 800.0 + 1e-3 && fb.bottom() <= 600.0 + 1e-3,
            "window left off screen: {fb:?}"
        );
    }

    #[test]
    fn cycling_an_application_stays_inside_it_and_wraps() {
        let mut m = wm();
        let a1 = m.open("app.files", "Documents", (300.0, 200.0));
        let a2 = m.open("app.files", "Pictures", (300.0, 200.0));
        let other = m.open("app.browser", "Browser", (300.0, 200.0));
        assert_eq!(m.focused(), Some(other));
        // Cycling from another application does nothing: there is nothing to
        // cycle inside Browser, and silence is better than stealing a window
        // from Files.
        assert_eq!(m.cycle_app(true), None);
        m.focus_window(a2);
        assert_eq!(m.cycle_app(true), Some(a1));
        assert_eq!(m.cycle_app(true), Some(a2), "the cycle did not wrap");
        assert_eq!(m.cycle_app(false), Some(a1));
    }

    #[test]
    fn cycling_brings_back_a_minimized_window_of_the_same_application() {
        let mut m = wm();
        let a = m.open("app.files", "Documents", (300.0, 200.0));
        let b = m.open("app.files", "Pictures", (300.0, 200.0));
        m.minimize(a);
        m.focus_window(b);
        assert_eq!(m.cycle_app(true), Some(a));
        assert_eq!(m.focused(), Some(a));
        assert_eq!(m.window(a).map(|w| w.state), Some(WindowState::Normal));
    }

    #[test]
    fn revealing_a_window_switches_to_its_workspace_and_undoes_a_minimize() {
        let mut m = wm();
        let a = m.open("app.files", "Documents", (300.0, 200.0));
        assert!(m.move_to_workspace(a, 2));
        assert_eq!(m.active_workspace(), 0);
        m.minimize(a);
        assert!(m.reveal(a));
        assert_eq!(m.active_workspace(), 2);
        assert_eq!(m.focused(), Some(a));
        let w = m.window(a).expect("revealed window still exists");
        assert_eq!(w.state, WindowState::Normal);
        assert!(w.is_visible());
    }

    #[test]
    fn diagnostics_name_the_focus_and_every_window() {
        let mut m = wm();
        let a = m.open("com.litric.activity", "Activity", (300.0, 200.0));
        let d = m.diagnostics();
        assert!(d.iter().any(|l| l == &format!("focus=#{a}")), "{d:?}");
        assert!(d.iter().any(|l| l.contains("Activity")), "{d:?}");
        assert!(d.iter().any(|l| l == "windows=1"));
    }
}
