//! Mission Control: every window, on every workspace, on one screen.
//!
//! The model is deliberately separate from the drawing. A window overview is
//! a grid of cards laid over the desktop, and the two questions that can be
//! wrong about it are which card the keyboard is on and where each card is.
//! Both are answered here, in pure functions, so a journey can prove that the
//! right arrow moved the selection and that Enter would focus the window the
//! highlight was on — without a display.
//!
//! The cards are ordered by workspace and then by window id. That is stable
//! across raising and focusing, which matters: an overview whose cards
//! rearranged themselves as the pointer crossed them would move the target out
//! from under the click that was already on its way.

use crate::canvas::Rect;

/// How many cards fit across, for a given number of windows.
///
/// One window is not a grid; two to four read as two columns; five to nine as
/// three; anything larger as four. The cap is what keeps a card readable at
/// 1280x800 rather than shrinking titles to nothing.
#[must_use]
pub fn columns(count: usize) -> usize {
    match count {
        0 | 1 => 1,
        2..=4 => 2,
        5..=9 => 3,
        _ => 4,
    }
}

/// A direction the keyboard can move the highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Previous card in reading order.
    Left,
    /// Next card in reading order.
    Right,
    /// The card one row above.
    Up,
    /// The card one row below.
    Down,
}

/// The next selected index for a grid of `count` cards over `cols` columns.
///
/// Left and right wrap at the ends, the way a focus ring should; up and down
/// clamp, because wrapping from the top row to the bottom of a grid reads as a
/// jump rather than a move. An empty grid stays at zero.
#[must_use]
pub fn step(selected: usize, count: usize, cols: usize, step: Step) -> usize {
    if count == 0 {
        return 0;
    }
    let cols = cols.max(1);
    let last = count.saturating_sub(1);
    let current = selected.min(last);
    match step {
        Step::Left => current.checked_sub(1).unwrap_or(last),
        Step::Right => current.saturating_add(1).checked_rem(count).unwrap_or(0),
        Step::Up => current.saturating_sub(cols),
        Step::Down => {
            let down = current.saturating_add(cols);
            if down > last {
                current
            } else {
                down
            }
        }
    }
}

/// The geometry of the cards, in a rectangle the overview may use.
///
/// `area` is the region below the title; cards are laid out in reading order
/// with a gap between them, and the last row is top-aligned rather than
/// stretched, so a grid of five cards has two equal rows rather than one tall
/// one.
#[must_use]
pub fn grid(area: Rect, count: usize, gap: f32) -> Vec<Rect> {
    if count == 0 || area.w <= 0.0 || area.h <= 0.0 {
        return Vec::new();
    }
    let cols = columns(count);
    let rows = count.div_ceil(cols);
    let rows = rows.max(1);
    let cell_w = (area.w - gap * (cols.saturating_sub(1)) as f32) / cols as f32;
    let cell_h = (area.h - gap * (rows.saturating_sub(1)) as f32) / rows as f32;
    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let col = index.checked_rem(cols).unwrap_or(0);
        let row = index.checked_div(cols).unwrap_or(0);
        out.push(Rect::new(
            area.x + col as f32 * (cell_w + gap),
            area.y + row as f32 * (cell_h + gap),
            cell_w.max(0.0),
            cell_h.max(0.0),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_covers_the_area_without_overlap_or_overflow() {
        let area = Rect::new(100.0, 80.0, 1000.0, 600.0);
        let cells = grid(area, 5, 12.0);
        assert_eq!(cells.len(), 5);
        for cell in &cells {
            assert!(cell.x >= area.x - 1e-3 && cell.right() <= area.right() + 1e-3);
            assert!(cell.y >= area.y - 1e-3 && cell.bottom() <= area.bottom() + 1e-3);
        }
        for (i, a) in cells.iter().enumerate() {
            for b in cells.iter().skip(i.saturating_add(1)) {
                let overlap = a.intersection(b).is_some_and(|r| r.w > 1e-3 && r.h > 1e-3);
                assert!(!overlap, "cards overlap: {a:?} and {b:?}");
            }
        }
    }

    #[test]
    fn an_empty_overview_draws_nothing_and_keeps_the_selection_valid() {
        assert!(grid(Rect::new(0.0, 0.0, 100.0, 100.0), 0, 8.0).is_empty());
        assert_eq!(step(4, 0, 3, Step::Right), 0);
    }

    #[test]
    fn left_and_right_wrap_and_up_and_down_clamp() {
        assert_eq!(step(0, 5, 3, Step::Left), 4);
        assert_eq!(step(4, 5, 3, Step::Right), 0);
        assert_eq!(step(1, 5, 3, Step::Down), 4);
        assert_eq!(step(4, 5, 3, Step::Down), 4);
        assert_eq!(step(1, 5, 3, Step::Up), 0);
        assert_eq!(step(0, 5, 3, Step::Up), 0);
    }

    #[test]
    fn the_column_count_grows_with_the_windows_and_stops_at_four() {
        assert_eq!(columns(1), 1);
        assert_eq!(columns(4), 2);
        assert_eq!(columns(5), 3);
        assert_eq!(columns(12), 4);
    }
}
