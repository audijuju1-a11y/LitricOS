//! Rectangles and the chrome metrics that hit-testing and drawing must share.
//!
//! Everything here is in **logical units**. A logical unit is a pixel at 100%
//! scale and half a pixel at 200%, so a layout written once works on a 1280x800
//! console and a 4K panel. The renderer multiplies by the scale factor exactly
//! once, at the edge; nothing in the window manager knows what a physical pixel
//! is.

/// A rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect {
    /// Construct.
    #[must_use]
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// The empty rectangle at the origin.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Right edge.
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    /// Bottom edge.
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    /// Centre.
    #[must_use]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
    /// Whether a point is inside.
    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }
    /// Whether two rectangles overlap at all.
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
    /// Grown by `d` on every side.
    #[must_use]
    pub fn inflate(&self, d: f32) -> Self {
        Self::new(self.x - d, self.y - d, self.w + d * 2.0, self.h + d * 2.0)
    }
    /// Moved by a delta.
    #[must_use]
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.w, self.h)
    }
    /// The smallest rectangle containing both.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        if self.w <= 0.0 || self.h <= 0.0 {
            return *other;
        }
        if other.w <= 0.0 || other.h <= 0.0 {
            return *self;
        }
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self::new(
            x,
            y,
            self.right().max(other.right()) - x,
            self.bottom().max(other.bottom()) - y,
        )
    }
    /// The overlap, or `None` when they do not touch.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = self.right().min(other.right());
        let b = self.bottom().min(other.bottom());
        if r <= x || b <= y {
            None
        } else {
            Some(Self::new(x, y, r - x, b - y))
        }
    }
    /// Clamped so it lies within `bounds`, shrinking only if it must.
    #[must_use]
    pub fn constrained_to(&self, bounds: &Self) -> Self {
        let w = self.w.min(bounds.w);
        let h = self.h.min(bounds.h);
        let x = self.x.clamp(bounds.x, (bounds.right() - w).max(bounds.x));
        let y = self.y.clamp(bounds.y, (bounds.bottom() - h).max(bounds.y));
        Self::new(x, y, w, h)
    }
    /// Whether this rectangle has any area.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }
}

/// The measurements of a window's frame.
///
/// Shared between hit-testing and drawing on purpose. When these are two
/// separate sets of numbers they drift, and the symptom is a close button that
/// does not close until you click two pixels above it — a bug that is nearly
/// impossible to see in a screenshot and instantly obvious to a user.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chrome {
    /// Height of the title bar.
    pub title_h: f32,
    /// How far outside and inside the edge counts as a resize grip.
    pub resize_grip: f32,
    /// Diameter of a title-bar button.
    pub button: f32,
    /// Gap between title-bar buttons.
    pub button_gap: f32,
    /// Left inset before the first button.
    pub button_inset: f32,
    /// Corner radius.
    pub radius: f32,
}

impl Default for Chrome {
    fn default() -> Self {
        Self {
            title_h: 34.0,
            resize_grip: 6.0,
            button: 12.0,
            button_gap: 8.0,
            button_inset: 16.0,
            radius: 14.0,
        }
    }
}

impl Chrome {
    /// The rectangle of the `n`th title-bar button, counted from the left.
    #[must_use]
    pub fn button_rect(&self, frame: &Rect, n: u32) -> Rect {
        let step = self.button + self.button_gap;
        let x = frame.x + self.button_inset + f32::from(n as u16) * step;
        let y = frame.y + (self.title_h - self.button) / 2.0;
        Rect::new(x, y, self.button, self.button)
    }

    /// The title bar.
    #[must_use]
    pub fn title_rect(&self, frame: &Rect) -> Rect {
        Rect::new(frame.x, frame.y, frame.w, self.title_h.min(frame.h))
    }

    /// The area a client would draw into.
    #[must_use]
    pub fn content_rect(&self, frame: &Rect) -> Rect {
        let h = (frame.h - self.title_h).max(0.0);
        Rect::new(frame.x, frame.y + self.title_h, frame.w, h)
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn containment_excludes_the_far_edges() {
        // Half-open on the right and bottom, so two rectangles that share an
        // edge do not both claim the same pixel column.
        let r = Rect::new(10.0, 10.0, 20.0, 20.0);
        assert!(r.contains(10.0, 10.0));
        assert!(r.contains(29.9, 29.9));
        assert!(!r.contains(30.0, 20.0));
        assert!(!r.contains(20.0, 30.0));
        assert!(!r.contains(9.9, 20.0));
    }

    #[test]
    fn intersection_and_intersects_agree() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        let c = Rect::new(20.0, 20.0, 5.0, 5.0);
        assert!(a.intersects(&b));
        assert_eq!(a.intersection(&b), Some(Rect::new(5.0, 5.0, 5.0, 5.0)));
        assert!(!a.intersects(&c));
        assert_eq!(a.intersection(&c), None);
    }

    #[test]
    fn touching_rectangles_do_not_intersect() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(10.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&b));
        assert_eq!(a.intersection(&b), None);
    }

    #[test]
    fn union_covers_both_and_ignores_empties() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(20.0, 5.0, 5.0, 5.0);
        assert_eq!(a.union(&b), Rect::new(0.0, 0.0, 25.0, 10.0));
        assert_eq!(Rect::ZERO.union(&a), a);
        assert_eq!(a.union(&Rect::ZERO), a);
    }

    #[test]
    fn constraining_moves_before_it_shrinks() {
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r = Rect::new(90.0, 90.0, 40.0, 40.0);
        let c = r.constrained_to(&bounds);
        assert_eq!(c, Rect::new(60.0, 60.0, 40.0, 40.0), "moved, not resized");

        let huge = Rect::new(-10.0, -10.0, 500.0, 500.0);
        assert_eq!(
            huge.constrained_to(&bounds),
            bounds,
            "too big: shrunk to fit"
        );
    }

    #[test]
    fn chrome_geometry_is_one_source_of_truth() {
        let c = Chrome::default();
        let f = Rect::new(100.0, 200.0, 400.0, 300.0);
        assert_eq!(c.title_rect(&f), Rect::new(100.0, 200.0, 400.0, 34.0));
        assert_eq!(c.content_rect(&f), Rect::new(100.0, 234.0, 400.0, 266.0));

        let b0 = c.button_rect(&f, 0);
        let b1 = c.button_rect(&f, 1);
        assert!((b1.x - b0.x - (c.button + c.button_gap)).abs() < 1e-6);
        assert!(
            c.title_rect(&f).contains(b0.x, b0.y),
            "button outside its bar"
        );
    }

    #[test]
    fn a_window_shorter_than_its_title_bar_does_not_produce_negative_content() {
        let c = Chrome::default();
        let tiny = Rect::new(0.0, 0.0, 100.0, 10.0);
        assert_eq!(c.title_rect(&tiny).h, 10.0);
        assert_eq!(c.content_rect(&tiny).h, 0.0);
    }
}
