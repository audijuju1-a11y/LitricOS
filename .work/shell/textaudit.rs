//! Finding text drawn over other text.
//!
//! The complaint this answers was concrete: "ตัวอักษรทับกัน" -- letters on top
//! of letters. A layout that is reviewed by looking at a picture catches that
//! only when someone happens to look at the window where it happens, and a
//! window is a different size for every user. The mechanical version is to
//! record every text run a frame draws ([`Canvas::record_text`]) and check the
//! geometry afterwards, which is what this module does.
//!
//! Two kinds of collision are distinguished, because they are fixed in
//! different places:
//!
//! * [`Kind::SameLine`] -- two runs on the same baseline whose horizontal
//!   extents overlap. A label and a right-aligned value in a window too narrow
//!   for both. The fix is a width or an elision, not a line height.
//! * [`Kind::LinesCollide`] -- two runs on different baselines whose bands
//!   overlap. Text set closer than its own height: the descenders of one line
//!   meeting the ascenders of the next. The fix is the row height.
//!
//! It reports, it does not repair: a repair needs to know which of the two runs
//! was in the wrong place, and that is a decision about the layout, not about
//! the geometry.

use crate::canvas::{Canvas, TextRun};

/// Horizontal overlap below this is treated as touching, not colliding: two
/// runs whose edges meet exactly are the normal case for adjacent columns.
pub const MIN_OVERLAP_X: f32 = 0.5;

/// Vertical overlap below this is treated as touching. Bands that share an edge
/// are letters that sit on top of each other's frames without meeting.
pub const MIN_OVERLAP_Y: f32 = 1.0;

/// Which kind of collision was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Two runs on the same baseline, overlapping horizontally.
    SameLine,
    /// Two runs on different baselines whose bands overlap.
    LinesCollide,
}

impl Kind {
    /// The word the report uses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SameLine => "same-line",
            Self::LinesCollide => "line-spacing",
        }
    }
}

/// One pair of runs that collide.
#[derive(Debug, Clone, PartialEq)]
pub struct Collision {
    /// The first run, in draw order.
    pub first: TextRun,
    /// The second run, which was drawn over the first.
    pub second: TextRun,
    /// What kind of collision it is.
    pub kind: Kind,
    /// How much they overlap horizontally, in pixels.
    pub overlap_x: f32,
    /// How much they overlap vertically, in pixels.
    pub overlap_y: f32,
    /// Whether the two runs were drawn on unrelated surfaces.
    ///
    /// Two windows overlapping on the desktop put their text over each other
    /// legitimately: the upper window's own background covers the lower one's
    /// letters, and nothing user-visible is wrong. Those pairs are reported but
    /// tagged, so a gate can require zero collisions *within* a surface without
    /// pretending the desktop does not stack windows.
    pub cross_surface: bool,
}

/// Whether two runs were drawn on the same surface.
///
/// The desktop chrome is surface 0 and each window its own, so two windows that
/// overlap on the screen -- whose text is legitimately on top of each other,
/// covered by the upper window's own background -- are not a collision, while
/// two labels colliding inside one window are.
#[must_use]
fn same_surface(a: u32, b: u32) -> bool {
    a == b
}

/// Every colliding pair among `runs`.
///
/// Pairs are compared in draw order, so `second` is always the run drawn later
/// -- the one that lands on top. Whitespace-only runs and runs with no width are
/// skipped: a space cannot collide with anything, and reporting it would bury
/// the real defect.
#[must_use]
pub fn collisions(runs: &[TextRun]) -> Vec<Collision> {
    let mut out = Vec::new();
    for (i, a) in runs.iter().enumerate() {
        if !a.has_ink() {
            continue;
        }
        for b in runs.iter().skip(i.saturating_add(1)) {
            if !b.has_ink() {
                continue;
            }
            let (ax0, ax1) = a.span();
            let (bx0, bx1) = b.span();
            let ox = ax1.min(bx1) - ax0.max(bx0);
            if ox < MIN_OVERLAP_X {
                continue;
            }
            let (atop, abottom) = a.band();
            let (btop, bbottom) = b.band();
            let oy = abottom.min(bbottom) - atop.max(btop);
            if oy < MIN_OVERLAP_Y {
                continue;
            }
            let kind = if (a.baseline - b.baseline).abs() < 0.5 {
                Kind::SameLine
            } else {
                Kind::LinesCollide
            };
            out.push(Collision {
                first: a.clone(),
                second: b.clone(),
                kind,
                overlap_x: ox,
                overlap_y: oy,
                cross_surface: !same_surface(a.surface, b.surface),
            });
        }
    }
    out
}

/// A one-line summary of the worst collisions, for the frame dumper.
///
/// The worst are the ones with the largest overlapping area, because those are
/// the ones a person can see from across the room; a 0.6-pixel kiss between two
/// columns is a rounding artefact and is reported only in the count.
#[must_use]
pub fn report(runs: &[TextRun], limit: usize) -> String {
    let found = collisions(runs);
    if found.is_empty() {
        return format!("none ({} runs checked)", runs.len());
    }
    // Collisions inside one surface first, then by overlapping area: the ones a
    // user can see are the ones inside a window, and a report of stacked windows
    // would otherwise bury them.
    let mut worst: Vec<&Collision> = found.iter().collect();
    worst.sort_by(|a, b| {
        let area = |c: &Collision| c.overlap_x * c.overlap_y;
        a.cross_surface
            .cmp(&b.cross_surface)
            .then_with(|| area(b).total_cmp(&area(a)))
    });
    let local = found.iter().filter(|c| !c.cross_surface).count();
    let samples: Vec<String> = worst
        .iter()
        .take(limit)
        .map(|c| {
            format!(
                "{} {:?}(x={:.0},y={:.0}) over {:?}(x={:.0},y={:.0}) [{:.0}x{:.0}]",
                if c.cross_surface {
                    "cross-surface"
                } else {
                    c.kind.as_str()
                },
                c.first.text,
                c.first.x,
                c.first.baseline,
                c.second.text,
                c.second.x,
                c.second.baseline,
                c.overlap_x,
                c.overlap_y
            )
        })
        .collect();
    format!(
        "{local} within a surface, {} across stacked windows, in {} runs; worst: {}",
        found.len().saturating_sub(local),
        runs.len(),
        samples.join(" | ")
    )
}

/// Audit a canvas that has been drawing with recording on.
#[must_use]
pub fn audit(canvas: &Canvas) -> Vec<Collision> {
    collisions(canvas.text_runs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, x: f32, baseline: f32, size: f32) -> TextRun {
        TextRun {
            text: text.to_owned(),
            x,
            end_x: x + 40.0,
            baseline,
            size,
            surface: 0,
        }
    }

    #[test]
    fn a_label_over_a_right_aligned_value_is_a_same_line_collision() {
        // "Latency" from x=0 to 40, and a value that starts at 20: the same
        // 13px line, 20 pixels of overlap.
        let runs = vec![
            run("Latency", 0.0, 20.0, 13.0),
            run("412 ms", 20.0, 20.0, 13.0),
        ];
        let found = collisions(&runs);
        assert_eq!(found.len(), 1);
        assert_eq!(found.first().map(|c| c.kind), Some(Kind::SameLine));
        assert!((found.first().map_or(0.0, |c| c.overlap_x) - 20.0).abs() < 1e-9);
    }

    #[test]
    fn rows_set_closer_than_their_own_height_are_reported_even_without_x_overlap() {
        // 17px text on a 20px row pitch: bands are 23.8px tall, so consecutive
        // rows overlap by nearly 4px. This is the "text touching text" case.
        let runs = vec![
            run("Memory", 0.0, 20.0, 17.0),
            run("Memory", 0.0, 40.0, 17.0),
        ];
        let found = collisions(&runs);
        assert_eq!(found.len(), 1);
        assert_eq!(found.first().map(|c| c.kind), Some(Kind::LinesCollide));
        assert!(found.first().map_or(0.0, |c| c.overlap_y) > 3.0);
    }

    #[test]
    fn properly_spaced_rows_do_not_collide() {
        // The same rows at a 26px pitch: 23.8px bands leave a gap.
        let runs = vec![
            run("Memory", 0.0, 20.0, 17.0),
            run("Memory", 0.0, 46.0, 17.0),
        ];
        assert!(collisions(&runs).is_empty());
    }

    #[test]
    fn text_in_two_stacked_windows_is_reported_as_cross_surface() {
        let mut a = run("Activity", 300.0, 100.0, 13.0);
        a.surface = 3;
        let mut b = run("Files", 310.0, 102.0, 13.0);
        b.surface = 7;
        let found = collisions(&[a, b]);
        assert_eq!(found.len(), 1);
        assert!(found.first().is_some_and(|c| c.cross_surface));
    }

    #[test]
    fn adjacent_columns_are_not_a_collision() {
        let runs = vec![
            run("Label", 0.0, 20.0, 13.0),
            run("Value", 40.0, 20.0, 13.0),
        ];
        assert!(
            collisions(&runs).is_empty(),
            "sharing an edge is not overlapping"
        );
    }

    #[test]
    fn a_space_cannot_collide_with_anything() {
        let runs = vec![
            TextRun {
                text: "   ".to_owned(),
                x: 0.0,
                end_x: 12.0,
                baseline: 20.0,
                size: 13.0,
                surface: 0,
            },
            run("real", 0.0, 20.0, 13.0),
        ];
        assert!(collisions(&runs).is_empty());
    }

    #[test]
    fn the_report_names_the_worst_case_first() {
        let runs = vec![
            run("a", 0.0, 20.0, 13.0),
            run("b", 39.0, 20.0, 13.0),
            run("c", 5.0, 20.0, 13.0),
        ];
        let text = report(&runs, 1);
        assert!(text.contains("same-line"), "{text}");
        // The 35-pixel overlap between `a` and `c` outranks the 1-pixel kiss.
        assert!(text.contains("\"a\"") && text.contains("\"c\""), "{text}");
    }
}
