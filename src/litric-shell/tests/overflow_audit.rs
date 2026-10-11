//! Text overflow auditing: proof that no drawn string exceeds its box.
//!
//! The user's complaint was concrete — "ตัวอักษรหลุดกรอบข้อความ", letters
//! escaping their text boxes. A fix that is only a careful rewrite is a fix
//! nobody can verify, so the check is mechanical: an [`OverflowAudit`] counts
//! strings drawn wider than the region that drew them, and the test suite
//! asserts the count is zero for a full frame at every scale.
//!
//! The audit does not hook the renderer. Hooks in the renderer would tax
//! every frame a user ever draws for a property only tests need. Instead the
//! audit is a library: a view that wants proof draws through
//! [`clipped_text`], which measures first and elides or clamps, and records
//! an overflow when even the clamped string would not fit.

use litric_shell::font::{self, Style};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The global count of overflows recorded by any audit this process.
static OVERFLOWS: AtomicUsize = AtomicUsize::new(0);

/// An audit for one drawing pass.
#[derive(Debug, Default)]
pub struct OverflowAudit {
    /// Overflows recorded by this audit.
    local: Cell<usize>,
}

impl OverflowAudit {
    /// A fresh audit.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            local: Cell::new(0),
        }
    }

    /// Draw `text` inside `max_width`, eliding when needed, and return the
    /// string actually drawn.
    ///
    /// A string that cannot fit even as an ellipsis is drawn as much as the
    /// style allows and the overflow is recorded — the caller learns through
    /// [`Self::overflow_count`], not by pixels silently leaving a box.
    #[must_use]
    pub fn clipped_text(&self, text: &str, style: &Style, max_width: f32) -> String {
        if font::measure(text, style) <= max_width {
            return text.to_owned();
        }
        let elided = litric_shell::ui::elide(text, style, max_width);
        if font::measure(&elided, style) <= max_width {
            return elided;
        }
        // Even "..." does not fit: record and return the bare ellipsis, the
        // shortest honest string there is.
        self.local.set(self.local.get().saturating_add(1));
        OVERFLOWS.fetch_add(1, Ordering::Relaxed);
        String::from("…")
    }

    /// Record that a string had to be clamped beyond eliding.
    pub fn record_overflow(&self) {
        self.local.set(self.local.get().saturating_add(1));
        OVERFLOWS.fetch_add(1, Ordering::Relaxed);
    }

    /// Overflows this audit recorded.
    #[must_use]
    pub fn overflow_count(&self) -> usize {
        self.local.get()
    }

    /// Reset this audit's count (the global count is never reset).
    pub fn reset(&self) {
        self.local.set(0);
    }

    /// The process-wide overflow count, for tests and diagnostics.
    #[must_use]
    pub fn global_overflow_count() -> usize {
        OVERFLOWS.load(Ordering::Relaxed)
    }

    /// Reset the process-wide count. Only tests should do this.
    #[cfg(test)]
    pub fn reset_global() {
        OVERFLOWS.store(0, Ordering::Relaxed);
    }
}

/// Draw a string clipped to a width, returning the string drawn. Convenience
/// for views that want the audit discipline without constructing one.
#[must_use]
pub fn clipped_text(text: &str, style: &Style, max_width: f32) -> String {
    if font::measure(text, style) <= max_width {
        return text.to_owned();
    }
    let elided = litric_shell::ui::elide(text, style, max_width);
    if font::measure(&elided, style) <= max_width {
        return elided;
    }
    OVERFLOWS.fetch_add(1, Ordering::Relaxed);
    String::from("…")
}

/// The process-wide count of overflows this process recorded.
#[must_use]
pub fn global_overflow_count() -> usize {
    OVERFLOWS.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use litric_shell::canvas::{rgb, Canvas};

    #[test]
    fn clipping_elides_and_never_exceeds_the_box() {
        let style = Style::new(12.0, rgb(0xFFFFFF));
        let audit = OverflowAudit::new();
        let wide = "a very long string that certainly does not fit in eighty pixels";
        let drawn = audit.clipped_text(wide, &style, 80.0);
        assert!(font::measure(&drawn, &style) <= 80.0, "{drawn}");
        assert_eq!(audit.overflow_count(), 0, "eliding is not overflowing");

        let tight = audit.clipped_text(wide, &style, 4.0);
        // With 4px there is no room even for an ellipsis: the audit records
        // the overflow and returns the shortest thing it has.
        assert_eq!(audit.overflow_count(), 1, "a 4px box overflows");
        let _ = tight;
    }

    #[test]
    fn an_impossible_box_records_an_overflow_rather_than_lying() {
        let style = Style::new(12.0, rgb(0xFFFFFF));
        let audit = OverflowAudit::new();
        let drawn = audit.clipped_text("abc", &style, 0.5);
        assert_eq!(drawn, "…");
        assert_eq!(audit.overflow_count(), 1);
        assert!(global_overflow_count() >= 1);
    }

    #[test]
    fn drawing_through_the_helper_stays_inside_a_clip() {
        // The end-to-end guarantee: pixels of the drawn string never appear
        // outside the region the view measured for.
        let mut c = Canvas::new(200, 40, rgb(0x000000));
        let style = Style::new(12.0, rgb(0xFFFFFF));
        let text = clipped_text("the quick brown fox jumps over the lazy dog", &style, 100.0);
        let w = font::measure(&text, &style);
        // `draw` returns the pen position after the string: its right edge.
        let right = font::draw(&mut c, &text, 50.0, 20.0, &style);
        assert!((right - 50.0 - w).abs() < 1.0, "measure and draw disagree");
        let mut escaped = 0;
        let box_right = (right + 2.0) as i64;
        // The line box a view must reserve: ascender to descender, not just
        // the x-height band.
        for y in 0..40i64 {
            for x in 0..200i64 {
                let in_box = x >= 48 && x < box_right && (20 - 13..=20 + 5).contains(&y);
                let lit = litric_shell::canvas::luminance(c.get(x, y).unwrap_or(0)) > 0.25;
                if lit && !in_box {
                    escaped += 1;
                }
            }
        }
        assert_eq!(escaped, 0, "glyph ink escaped its measured box");
    }
}
