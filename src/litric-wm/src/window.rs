//! What a window *is*.
//!
//! A window here is an object with an identity, not a rectangle the shell
//! remembered to draw again. That distinction is the whole point of this crate:
//! a rectangle cannot be focused, cannot be raised above another rectangle,
//! cannot be moved to a workspace and cannot tell you whether the thing that
//! owns it is still alive.

use crate::geom::Rect;

/// A window's identity. Never reused within a session, so a stale reference is
/// a miss rather than a hit on somebody else's window.
pub type WindowId = u32;

/// Which half or quarter of the work area a window is snapped to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SnapEdge {
    /// Left half.
    Left,
    /// Right half.
    Right,
    /// Top-left quarter.
    TopLeft,
    /// Top-right quarter.
    TopRight,
    /// Bottom-left quarter.
    BottomLeft,
    /// Bottom-right quarter.
    BottomRight,
}

impl SnapEdge {
    /// The rectangle this edge means within a work area.
    #[must_use]
    pub fn bounds(self, work: Rect) -> Rect {
        let half_w = work.w / 2.0;
        let half_h = work.h / 2.0;
        match self {
            Self::Left => Rect::new(work.x, work.y, half_w, work.h),
            Self::Right => Rect::new(work.x + half_w, work.y, half_w, work.h),
            Self::TopLeft => Rect::new(work.x, work.y, half_w, half_h),
            Self::TopRight => Rect::new(work.x + half_w, work.y, half_w, half_h),
            Self::BottomLeft => Rect::new(work.x, work.y + half_h, half_w, half_h),
            Self::BottomRight => Rect::new(work.x + half_w, work.y + half_h, half_w, half_h),
        }
    }

    /// A short name for diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
        }
    }
}

/// How a window is currently placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WindowState {
    /// Free-floating at its own size.
    Normal,
    /// Hidden, but still open and still in the switcher.
    Minimized,
    /// Filling the work area, below the top bar.
    Maximized,
    /// Filling part of the work area.
    Snapped(SnapEdge),
    /// Filling the whole display, over the top bar.
    Fullscreen,
}

impl WindowState {
    /// Whether a window in this state is on screen.
    #[must_use]
    pub const fn is_visible(self) -> bool {
        !matches!(self, Self::Minimized)
    }

    /// Whether the user can drag or resize it in this state.
    ///
    /// A maximized window that can be dragged is a window that ends up
    /// half-off the screen with no way back.
    #[must_use]
    pub const fn is_free(self) -> bool {
        matches!(self, Self::Normal)
    }

    /// A short name for diagnostics and the switcher.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Minimized => "minimized",
            Self::Maximized => "maximized",
            Self::Snapped(_) => "snapped",
            Self::Fullscreen => "fullscreen",
        }
    }
}

/// How a window's contents are produced.
///
/// Today every window is drawn by the shell itself. The distinction exists now,
/// before there are any clients, so that the window manager is never written in
/// a way that assumes there is only one kind — which is exactly how a shell ends
/// up unable to host anything but itself.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Content {
    /// A panel the shell draws: Activity, Environments, Settings.
    Builtin {
        /// Which one.
        view: String,
    },
    /// A surface belonging to another process. Not implemented: no client
    /// protocol exists yet, and a window in this state renders as a placeholder
    /// that says so rather than pretending to host something.
    Client {
        /// The process that would own it.
        pid: u32,
    },
}

/// One window.
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    /// Identity.
    pub id: WindowId,
    /// The application that owns it, e.g. `com.litric.activity`.
    pub app_id: String,
    /// What the title bar says.
    pub title: String,
    /// Where it is now, including its frame.
    pub frame: Rect,
    /// Where [`WindowState::Normal`] would put it. Preserved across maximize
    /// and snap so restoring puts a window back where the user left it rather
    /// than at some default.
    pub restore: Rect,
    /// Placement.
    pub state: WindowState,
    /// Which workspace it lives on.
    pub workspace: usize,
    /// Smallest size the user may drag it to.
    pub min_size: (f32, f32),
    /// Whether it may be resized at all.
    pub resizable: bool,
    /// Whether it may be closed.
    pub closable: bool,
    /// How its contents are produced.
    pub content: Content,
}

impl Window {
    /// A new normal window.
    #[must_use]
    pub fn new(id: WindowId, app_id: &str, title: &str, frame: Rect) -> Self {
        Self {
            id,
            app_id: app_id.to_owned(),
            title: title.to_owned(),
            frame,
            restore: frame,
            state: WindowState::Normal,
            workspace: 0,
            min_size: (220.0, 120.0),
            resizable: true,
            closable: true,
            content: Content::Builtin {
                view: app_id.to_owned(),
            },
        }
    }

    /// Whether it is on screen on its own workspace.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.state.is_visible()
    }

    /// A one-line form for the switcher and diagnostics.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "#{} {} \"{}\" {}x{}+{}+{} {} ws{}",
            self.id,
            self.app_id,
            self.title,
            self.frame.w as i32,
            self.frame.h as i32,
            self.frame.x as i32,
            self.frame.y as i32,
            self.state.name(),
            self.workspace
        )
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    const WORK: Rect = Rect::new(0.0, 34.0, 1280.0, 766.0);

    #[test]
    fn halves_tile_the_work_area_exactly() {
        let l = SnapEdge::Left.bounds(WORK);
        let r = SnapEdge::Right.bounds(WORK);
        assert_eq!(l.right(), r.x, "a gap or an overlap down the middle");
        assert_eq!(l.w + r.w, WORK.w);
        assert_eq!(l.h, WORK.h);
        assert_eq!(l.y, WORK.y, "snapping must not go under the top bar");
    }

    #[test]
    fn quarters_tile_the_work_area_exactly() {
        let quarters = [
            SnapEdge::TopLeft,
            SnapEdge::TopRight,
            SnapEdge::BottomLeft,
            SnapEdge::BottomRight,
        ];
        let area: f32 = quarters
            .iter()
            .map(|q| q.bounds(WORK))
            .map(|r| r.w * r.h)
            .sum();
        assert!((area - WORK.w * WORK.h).abs() < 1.0, "quarters do not tile");
        for a in quarters {
            for b in quarters {
                if a != b {
                    assert!(
                        !a.bounds(WORK).intersects(&b.bounds(WORK)),
                        "{} overlaps {}",
                        a.name(),
                        b.name()
                    );
                }
            }
        }
    }

    #[test]
    fn a_minimized_window_is_not_visible_but_is_still_a_window() {
        assert!(!WindowState::Minimized.is_visible());
        assert!(WindowState::Normal.is_visible());
        assert!(WindowState::Maximized.is_visible());
    }

    #[test]
    fn only_a_normal_window_can_be_dragged() {
        // Dragging a maximized window off the screen with no way back is a
        // classic and infuriating bug.
        assert!(WindowState::Normal.is_free());
        assert!(!WindowState::Maximized.is_free());
        assert!(!WindowState::Snapped(SnapEdge::Left).is_free());
        assert!(!WindowState::Fullscreen.is_free());
    }

    #[test]
    fn a_new_window_remembers_where_it_started() {
        let f = Rect::new(10.0, 50.0, 400.0, 300.0);
        let w = Window::new(1, "com.litric.activity", "Activity", f);
        assert_eq!(w.restore, f);
        assert_eq!(w.state, WindowState::Normal);
        assert!(w.is_visible());
        assert!(w.summary().contains("Activity"));
    }
}
