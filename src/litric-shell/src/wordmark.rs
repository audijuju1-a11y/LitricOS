//! The LITRICOS wordmark: a handwritten mark, written left to right on boot.
//!
//! The mark is one pen path, not a typeface. Strokes are authored as a handful
//! of control points, smoothed with a Catmull-Rom spline and slanted a little,
//! the way a hand leans. The pen lifts only where a hand would: for the bar of
//! the T, and between the letters C-O and S. Everything else is joined by short
//! baseline links, so the letters run together as one written word.
//!
//! The reveal is by arc length: `progress` is the fraction of the total inked
//! path that is visible, the same mechanism as `stroke-dasharray` with
//! `stroke-dashoffset` in an SVG. A real pen is not a monoline, so the drawing
//! gives the line a width that follows the direction of travel (thick on a
//! downstroke, a hairline on a link or an upstroke) and tapers where the pen
//! lands and leaves. The exported SVG is the centre line of the same strokes,
//! built from [`strokes`], and a test holds the committed file to that.
//!
//! Coordinates are in wordmark units: `x` runs `0..`[`WIDTH`] left to right and
//! `y` runs `0` at the cap line to `1` at the baseline, as in screen space.

use crate::canvas::{rgb, Canvas, Rect};

/// A point in wordmark units.
pub type Pt = (f32, f32);

/// Samples per control-point segment when smoothing.
const SAMPLES: usize = 10;
/// How far the top of a letter leans right of its foot, in units.
const SLANT: f32 = 0.14;
/// The mark's full width in units: the last control point plus the lean.
pub const WIDTH: f32 = 5.30;
/// The mark's full height in units: cap line to baseline.
pub const HEIGHT: f32 = 1.0;

/// The nib's angle: a horizontal edge, so a stroke that runs sideways is a
/// hairline and one that runs straight down is full width.
const NIB: f32 = 0.14;
/// The width of a hairline as a share of a full stroke.
const HAIRLINE: f32 = 0.28;
/// Distance over which the pen lands, and over which it lifts, in units.
const LAND: f32 = 0.05;
const LIFT: f32 = 0.09;

/// The pen path, in the order it is written. Each stroke is drawn without
/// lifting the pen.
const PATH: &[&[Pt]] = &[
    // L I T: each stem is written up as a hairline and down at full width, and
    // the foot of one runs along the baseline into the next.
    &[
        (0.22, 0.00),
        (0.20, 0.40),
        (0.18, 0.80),
        (0.22, 0.95),
        (0.34, 1.00),
        (0.55, 0.99),
        (0.78, 0.97),
        (0.96, 0.98),
        (1.06, 0.60),
        (1.10, 0.08),
        (1.11, 0.00),
        (1.10, 0.30),
        (1.08, 0.70),
        (1.07, 0.97),
        (1.12, 1.00),
        (1.24, 0.99),
        (1.36, 0.98),
        (1.44, 0.70),
        (1.50, 0.08),
        (1.51, 0.00),
        (1.50, 0.40),
        (1.48, 0.85),
        (1.50, 1.00),
        (1.58, 0.99),
    ],
    // The bar of the T, the first lift.
    &[(1.28, 0.07), (1.52, 0.03), (1.78, 0.05)],
    // R I: the stem, the bowl written back up the stem, the leg, then the I.
    &[
        (1.62, 0.99),
        (1.78, 0.98),
        (1.90, 0.60),
        (1.98, 0.05),
        (1.99, 0.00),
        (1.97, 0.50),
        (1.95, 0.98),
        (1.98, 1.00),
        (1.98, 0.55),
        (1.99, 0.05),
        (2.18, 0.00),
        (2.38, 0.06),
        (2.46, 0.26),
        (2.34, 0.46),
        (2.12, 0.52),
        (2.03, 0.53),
        (2.20, 0.62),
        (2.34, 0.84),
        (2.46, 0.98),
        (2.60, 0.98),
        (2.74, 0.97),
        (2.84, 0.70),
        (2.90, 0.08),
        (2.91, 0.00),
        (2.90, 0.40),
        (2.88, 0.70),
        (2.87, 0.97),
        (2.92, 1.00),
        (3.02, 0.99),
    ],
    // C O: the C is written from its open end, and its foot runs along the
    // baseline into the O.
    &[
        (3.78, 0.16),
        (3.60, 0.02),
        (3.38, 0.00),
        (3.18, 0.14),
        (3.12, 0.50),
        (3.20, 0.86),
        (3.42, 1.00),
        (3.62, 0.99),
        (3.82, 0.99),
        (4.00, 1.00),
        (4.22, 0.98),
        (4.40, 0.74),
        (4.40, 0.34),
        (4.28, 0.06),
        (4.10, 0.00),
        (3.94, 0.12),
        (3.88, 0.48),
        (3.96, 0.84),
        (4.10, 0.99),
        (4.24, 0.98),
    ],
    // S, the last lift.
    &[
        (5.12, 0.14),
        (4.96, 0.02),
        (4.76, 0.02),
        (4.66, 0.20),
        (4.78, 0.40),
        (4.96, 0.52),
        (5.10, 0.70),
        (5.06, 0.92),
        (4.86, 1.00),
        (4.66, 0.94),
        (4.58, 0.80),
    ],
];

/// One Catmull-Rom segment between `p1` and `p2`, at `t` in `0..1`.
fn catmull(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Smooth one stroke's control points into a dense, slanted polyline.
fn smooth(points: &[Pt]) -> Vec<Pt> {
    let last = points.len().saturating_sub(1);
    let at = |k: usize| points.get(k.min(last)).copied().unwrap_or((0.0, 0.0));
    let lean = |(x, y): Pt| (x + SLANT * (1.0 - y), y);
    let mut out = Vec::with_capacity(last.saturating_mul(SAMPLES).saturating_add(1));
    for i in 0..last {
        let (p0, p1) = (at(i.saturating_sub(1)), at(i));
        let (p2, p3) = (at(i.saturating_add(1)), at(i.saturating_add(2)));
        for s in 0..SAMPLES {
            let t = s as f32 / SAMPLES as f32;
            out.push(lean((
                catmull(p0.0, p1.0, p2.0, p3.0, t),
                catmull(p0.1, p1.1, p2.1, p3.1, t),
            )));
        }
    }
    if let Some(&end) = points.last() {
        out.push(lean(end));
    }
    out
}

/// Every stroke of the wordmark as a dense polyline, in the order it is written.
#[must_use]
pub fn strokes() -> Vec<Vec<Pt>> {
    PATH.iter().map(|s| smooth(s)).collect()
}

fn dist(a: Pt, b: Pt) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}

/// Each consecutive pair of points in a stroke, as `(from, to)`.
fn segments(stroke: &[Pt]) -> impl Iterator<Item = (Pt, Pt)> + '_ {
    stroke.windows(2).filter_map(|w| match w {
        [a, b] => Some((*a, *b)),
        _ => None,
    })
}

fn length(stroke: &[Pt]) -> f32 {
    segments(stroke).map(|(a, b)| dist(a, b)).sum()
}

/// The part of `strokes` that is inked after `progress` (`0..=1`) of the total
/// pen path. Strokes are consumed in order; the last one is cut mid-curve.
#[must_use]
pub fn inked(strokes: &[Vec<Pt>], progress: f32) -> Vec<Vec<Pt>> {
    let total: f32 = strokes.iter().map(|s| length(s)).sum();
    let mut budget = total * progress.clamp(0.0, 1.0);
    let mut out = Vec::new();
    for stroke in strokes {
        if budget <= 0.0 {
            break;
        }
        let len = length(stroke);
        if len <= budget {
            out.push(stroke.clone());
            budget -= len;
            continue;
        }
        let mut part = Vec::new();
        if let Some(&first) = stroke.first() {
            part.push(first);
        }
        for (a, b) in segments(stroke) {
            let seg = dist(a, b);
            if seg >= budget {
                let f = budget / seg;
                part.push((a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f));
                break;
            }
            part.push(b);
            budget -= seg;
        }
        out.push(part);
        break;
    }
    out
}

/// The line width at each point of a stroke, as a share of a full stroke.
///
/// It follows the direction of travel through the nib, is averaged over its
/// neighbours so a corner does not make the line jump, and tapers where the
/// pen lands and where it lifts.
fn pen_widths(stroke: &[Pt]) -> Vec<f32> {
    let n = stroke.len();
    let at = |k: usize| stroke.get(k.min(n.saturating_sub(1))).copied().unwrap_or((0.0, 0.0));
    let raw: Vec<f32> = (0..n)
        .map(|i| {
            let (a, b) = (at(i.saturating_sub(1)), at(i.saturating_add(1)));
            let theta = (b.1 - a.1).atan2(b.0 - a.0);
            HAIRLINE + (1.0 - HAIRLINE) * (theta - NIB).sin().abs()
        })
        .collect();
    let total = length(stroke);
    let mut run = 0.0_f32;
    (0..n)
        .map(|i| {
            let lo = i.saturating_sub(3);
            let hi = i.saturating_add(3).min(n.saturating_sub(1));
            let window = raw.get(lo..=hi).unwrap_or(&[]);
            let mean = window.iter().sum::<f32>() / window.len().max(1) as f32;
            if i > 0 {
                run += dist(at(i.saturating_sub(1)), at(i));
            }
            let landing = (0.3 + 0.7 * run / LAND).min(1.0);
            let leaving = (0.3 + 0.7 * (total - run) / LIFT).clamp(0.3, 1.0);
            mean * landing * leaving
        })
        .collect()
}

/// Draw the wordmark onto `canvas`, with its origin at `(x, y)` and one unit
/// equal to `scale` pixels. `progress` is how much of the pen path is inked and
/// `pen_px` is the width of a full downstroke.
pub fn render(canvas: &mut Canvas, origin: Pt, scale: f32, progress: f32, pen_px: f32, color: u32) {
    let all = strokes();
    let total: f32 = all.iter().map(|s| length(s)).sum();
    let mut budget = total * progress.clamp(0.0, 1.0);
    for stroke in &all {
        let widths = pen_widths(stroke);
        let at = |p: Pt| (origin.0 + p.0 * scale, origin.1 + p.1 * scale);
        for (i, (a, b)) in segments(stroke).enumerate() {
            let seg = dist(a, b);
            if budget <= 0.0 {
                return;
            }
            let f = (budget / seg.max(f32::EPSILON)).min(1.0);
            let end = (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
            let w = widths.get(i).copied().unwrap_or(1.0);
            let w2 = widths.get(i.saturating_add(1)).copied().unwrap_or(w);
            let (x0, y0) = at(a);
            let (x1, y1) = at(end);
            canvas.stroke_line(x0, y0, x1, y1, pen_px * 0.5 * (w + w2), color);
            budget -= seg;
        }
    }
}

/// The colour of the boot surface: one opaque field, no gradient.
pub const FIELD: u32 = 0x11_13_18;

/// The pen's pace: it settles into the word and eases off at the end, rather
/// than moving at one speed like a plotter.
fn pace(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let smooth = t * t * (3.0 - 2.0 * t);
    0.4 * t + 0.6 * smooth
}

/// Draw the boot surface: the mark written across a plain Obsidian field,
/// centred on the screen. `progress` is how much of the write is done, `0..=1`.
///
/// Only the mark is on the surface. The note and progress bar this used to draw
/// told a story on a timer rather than the machine's state, and the brief for
/// the intro is a pen writing a word, not a loader.
pub fn draw_intro(canvas: &mut Canvas, progress: f32) {
    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
    canvas.fill_round_rect(Rect::new(0.0, 0.0, w, h), 0.0, rgb(FIELD));
    let scale = (w * 0.42 / WIDTH).min(h * 0.34 / HEIGHT);
    let origin = ((w - WIDTH * scale) / 2.0, (h - HEIGHT * scale) / 2.0);
    let pen = (scale * 0.082).max(2.0);
    render(canvas, origin, scale, pace(progress), pen, rgb(0xF4_F2_ED));
}

/// The static wordmark as an SVG document, in a `viewBox` of wordmark units.
///
/// `color` is an SVG colour such as `#F4F2ED`. Each stroke is one `path`, so a
/// viewer can animate it with `stroke-dasharray` the same way [`inked`] reveals it.
#[must_use]
pub fn svg(color: &str) -> String {
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"-0.05 -0.05 {:.3} {:.3}\">\n",
        WIDTH + 0.10,
        HEIGHT + 0.10
    );
    for stroke in strokes() {
        let mut d = String::new();
        for (i, (x, y)) in stroke.iter().enumerate() {
            let cmd = if i == 0 { "M" } else { "L" };
            d.push_str(&format!("{cmd}{x:.3} {y:.3} "));
        }
        s.push_str(&format!(
            "  <path d=\"{}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"0.045\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\n",
            d.trim_end()
        ));
    }
    s.push_str("</svg>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::rgb;

    /// The committed SVG asset, which must match what the code generates.
    const COMMITTED: &str = include_str!("../../../assets/brand/litric-wordmark.svg");

    fn all_points() -> impl Iterator<Item = Pt> {
        strokes().into_iter().flatten()
    }

    #[test]
    fn the_mark_stays_inside_its_box() {
        // Catmull-Rom bulges a little past its end points, so the mark may
        // overshoot the cap line and baseline by a hair. The SVG viewBox
        // leaves 0.05 units of margin for exactly that, and the test holds
        // the drawing to the same margin.
        const MARGIN: f32 = 0.05;
        for (x, y) in all_points() {
            assert!(
                (-MARGIN..=WIDTH + MARGIN).contains(&x),
                "x {x} outside the viewBox"
            );
            assert!(
                (-MARGIN..=HEIGHT + MARGIN).contains(&y),
                "y {y} outside the viewBox"
            );
        }
    }

    #[test]
    fn every_point_is_finite_and_no_stroke_is_empty() {
        for stroke in strokes() {
            assert!(stroke.len() >= 2, "a stroke has no length");
            assert!(stroke.iter().all(|(x, y)| x.is_finite() && y.is_finite()));
        }
    }

    #[test]
    fn the_pen_moves_left_to_right() {
        // The reveal is a left-to-right write, so each letter's first point
        // must start to the right of the one before it.
        let starts: Vec<f32> = strokes().iter().filter_map(|s| s.first().map(|p| p.0)).collect();
        let mut last = f32::MIN;
        for x in starts {
            assert!(x >= last - 0.2, "stroke starts {x} before {last}");
            last = x;
        }
    }

    #[test]
    fn nothing_is_inked_at_zero_and_everything_at_one() {
        let all = strokes();
        assert!(inked(&all, 0.0).is_empty());
        let full: f32 = inked(&all, 1.0).iter().map(|s| length(s)).sum();
        let total: f32 = all.iter().map(|s| length(s)).sum();
        assert!((full - total).abs() < 1e-3, "full {full} vs total {total}");
    }

    #[test]
    fn the_inked_length_grows_monotonically() {
        let all = strokes();
        let mut previous = 0.0_f32;
        for step in 0..=50 {
            let p = step as f32 / 50.0;
            let now: f32 = inked(&all, p).iter().map(|s| length(s)).sum();
            assert!(now + 1e-4 >= previous, "reveal went backwards at {p}");
            previous = now;
        }
    }

    #[test]
    fn the_reveal_is_a_prefix_of_the_mark() {
        // Half-way must not skip ahead: the first point of the ink is the
        // first point of the mark, so the write starts at the left.
        let all = strokes();
        let first = all.first().and_then(|s| s.first()).copied();
        let half_first = inked(&all, 0.5).first().and_then(|s| s.first()).copied();
        assert_eq!(half_first, first);
    }

    #[test]
    fn rendering_a_partial_mark_draws_fewer_pixels_than_the_full_one() {
        let ink = |progress: f32| {
            let mut c = Canvas::new(400, 120, rgb(0x111318));
            render(&mut c, (10.0, 10.0), 60.0, progress, 3.0, rgb(0xF4F2ED));
            c.pixels().iter().filter(|&&p| p != rgb(0x111318)).count()
        };
        let quarter = ink(0.25);
        let full = ink(1.0);
        assert!(quarter > 0, "a quarter of the write drew nothing");
        assert!(quarter < full, "quarter {quarter} is not less than full {full}");
    }

    fn intro_shell(reduce_motion: bool) -> crate::shell::Shell {
        let tokens = crate::ui::Tokens::new(
            litric_design::Theme::of(litric_design::ThemeKind::Dark),
            1.0,
        );
        let mut s = crate::shell::Shell::new(
            tokens,
            1280.0,
            800.0,
            litric_locale::Regional::fallback("test"),
        );
        s.mode = crate::shell::DesktopMode::Ready;
        let prefs = crate::settings::Preferences {
            reduce_motion,
            ..crate::settings::Preferences::default()
        };
        s.load_preferences(prefs);
        s
    }

    fn frame(s: &mut crate::shell::Shell) -> Canvas {
        let mut c = Canvas::new(1280, 800, rgb(0x000000));
        crate::views::draw(&mut c, s);
        c
    }

    #[test]
    fn the_intro_is_the_mark_then_the_desktop() {
        use crate::shell::BOOT_INTRO_MS;
        let field = rgb(0x11_13_18);
        let mut s = intro_shell(false);
        s.start_boot_intro(1_000);
        s.tick(1_000);

        // At the start the field is plain: the dock the desktop would draw
        // at this point is not on screen.
        assert_eq!(frame(&mut s).get(640, 740), Some(field), "the desktop showed through");

        // Mid-write, ink has reached the screen.
        // Half-way through the 1.9 s pen write.
        s.tick(1_000 + 950);
        let inked = frame(&mut s).pixels().iter().filter(|&&p| p != field).count();
        assert!(inked > 0, "nothing was written mid-intro");

        // Once the intro has run its course, the desktop is back.
        s.tick(1_000 + BOOT_INTRO_MS);
        assert_eq!(s.boot_intro_now(), None);
        assert_ne!(frame(&mut s).get(640, 740), Some(field), "the desktop did not return");
    }

    #[test]
    fn reduced_motion_holds_the_finished_mark_then_ends() {
        use crate::shell::BOOT_INTRO_MS;
        let mut s = intro_shell(true);
        s.start_boot_intro(0);
        s.tick(10);
        assert_eq!(s.boot_intro_now(), Some(1.0));
        s.tick(BOOT_INTRO_MS);
        assert_eq!(s.boot_intro_now(), None, "reduced motion never ended the intro");
    }

    #[test]
    fn the_loop_keeps_frames_coming_during_the_intro() {
        let mut s = intro_shell(false);
        s.start_boot_intro(0);
        s.tick(0);
        assert!(s.wake_after_ms(100) <= 33, "the intro would stall at idle pace");
    }

    #[test]
    fn the_committed_svg_is_the_generated_one() {
        assert_eq!(
            COMMITTED,
            svg("#F4F2ED"),
            "regenerate with: cargo run -p litric-shell --example wordmark_svg -- ../assets/brand/litric-wordmark.svg"
        );
    }
}
