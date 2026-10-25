//! The LITRICOS wordmark: a handwritten mark drawn left to right on first boot.
//!
//! The letters are authored as pen strokes, not set in a typeface. Each stroke
//! is a handful of control points smoothed with a Catmull-Rom spline into a
//! dense polyline, so the curves are continuous rather than faceted.
//!
//! The reveal is by arc length: `progress` is the fraction of the total inked
//! path that is visible. That is the same mechanism as `stroke-dasharray` with
//! `stroke-dashoffset` in an SVG, applied to the software canvas. The static
//! mark and the exported SVG are both built from [`strokes`], so they cannot
//! drift apart; a test holds the committed SVG to that.
//!
//! Coordinates are in wordmark units: `x` runs `0..`[`WIDTH`] left to right and
//! `y` runs `0` at the cap line to `1` at the baseline, as in screen space.

use crate::canvas::{rgb, Canvas, Rect};

/// A point in wordmark units.
pub type Pt = (f32, f32);

/// Distance between letter origins, in wordmark units.
const ADVANCE: f32 = 0.84;
/// Samples per control-point segment when smoothing.
const SAMPLES: usize = 12;
/// The mark's full width in units: seven advances plus the last letter's box.
pub const WIDTH: f32 = 7.0 * ADVANCE + 0.70;
/// The mark's full height in units: cap line to baseline.
pub const HEIGHT: f32 = 1.0;

// Each letter is a list of strokes; each stroke is drawn without lifting the
// pen. Letters are drawn in their box (x `0..0.7`); the advance moves them.
const GLYPH_L: &[&[Pt]] = &[&[
    (0.10, 0.00),
    (0.13, 0.45),
    (0.15, 0.90),
    (0.22, 0.99),
    (0.45, 1.00),
    (0.66, 0.99),
]];
const GLYPH_I: &[&[Pt]] = &[&[(0.22, 0.00), (0.21, 0.40), (0.19, 0.80), (0.20, 1.00)]];
const GLYPH_T: &[&[Pt]] = &[
    &[(0.02, 0.08), (0.30, 0.03), (0.62, 0.07)],
    &[(0.38, 0.06), (0.34, 0.50), (0.30, 0.99)],
];
const GLYPH_R: &[&[Pt]] = &[
    &[(0.10, 0.00), (0.09, 0.50), (0.10, 1.00)],
    &[
        (0.10, 0.02),
        (0.42, 0.00),
        (0.57, 0.20),
        (0.46, 0.46),
        (0.12, 0.50),
    ],
    &[(0.36, 0.50), (0.62, 1.00)],
];
const GLYPH_C: &[&[Pt]] = &[&[
    (0.64, 0.18),
    (0.52, 0.02),
    (0.30, 0.00),
    (0.08, 0.20),
    (0.06, 0.52),
    (0.14, 0.84),
    (0.34, 1.00),
    (0.56, 0.92),
    (0.66, 0.78),
]];
const GLYPH_O: &[&[Pt]] = &[&[
    (0.36, 0.00),
    (0.10, 0.14),
    (0.06, 0.50),
    (0.12, 0.86),
    (0.36, 1.00),
    (0.60, 0.86),
    (0.64, 0.50),
    (0.60, 0.14),
    (0.36, 0.00),
]];
const GLYPH_S: &[&[Pt]] = &[&[
    (0.60, 0.12),
    (0.46, 0.01),
    (0.22, 0.02),
    (0.12, 0.22),
    (0.26, 0.46),
    (0.50, 0.54),
    (0.62, 0.76),
    (0.48, 0.98),
    (0.22, 0.98),
    (0.06, 0.84),
]];

/// The letters in order: L I T R I C O S.
const LETTERS: [&[&[Pt]]; 8] = [
    GLYPH_L, GLYPH_I, GLYPH_T, GLYPH_R, GLYPH_I, GLYPH_C, GLYPH_O, GLYPH_S,
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

/// Smooth one stroke's control points into a dense polyline shifted by `dx`.
fn smooth(points: &[Pt], dx: f32) -> Vec<Pt> {
    let last = points.len().saturating_sub(1);
    let at = |k: usize| points.get(k.min(last)).copied().unwrap_or((0.0, 0.0));
    let mut out = Vec::with_capacity(last.saturating_mul(SAMPLES).saturating_add(1));
    for i in 0..last {
        let (p0, p1) = (at(i.saturating_sub(1)), at(i));
        let (p2, p3) = (at(i.saturating_add(1)), at(i.saturating_add(2)));
        for s in 0..SAMPLES {
            let t = s as f32 / SAMPLES as f32;
            out.push((
                catmull(p0.0, p1.0, p2.0, p3.0, t) + dx,
                catmull(p0.1, p1.1, p2.1, p3.1, t),
            ));
        }
    }
    if let Some(&end) = points.last() {
        out.push((end.0 + dx, end.1));
    }
    out
}

/// Every stroke of the wordmark as a dense polyline, left to right.
#[must_use]
pub fn strokes() -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    for (index, letter) in LETTERS.iter().enumerate() {
        let dx = index as f32 * ADVANCE;
        for stroke in letter.iter() {
            out.push(smooth(stroke, dx));
        }
    }
    out
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

/// Draw the wordmark onto `canvas`, with its origin at `(x, y)` and one unit
/// equal to `scale` pixels. `progress` is how much of the pen path is inked.
pub fn render(canvas: &mut Canvas, origin: Pt, scale: f32, progress: f32, pen_px: f32, color: u32) {
    let all = strokes();
    for stroke in inked(&all, progress) {
        for (a, b) in segments(&stroke) {
            canvas.stroke_line(
                origin.0 + a.0 * scale,
                origin.1 + a.1 * scale,
                origin.0 + b.0 * scale,
                origin.1 + b.1 * scale,
                pen_px,
                color,
            );
        }
    }
}

/// Draw the boot intro: the mark written across a plain Obsidian field, centred
/// at about 40% of the screen. `progress` is how much of the pen path is inked,
/// and `note` is the line under it that says what the machine is doing -- the
/// "Getting things ready" the surface exists to say out loud.
///
/// The field is one opaque colour, so nothing from the desktop underneath shows
/// through and no gradient is involved. The note and its track are drawn from
/// the very first frame, so a machine that is slow to bring the session up says
/// so instead of looking stuck.
pub fn draw_intro(canvas: &mut Canvas, progress: f32, note: &str) {
    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
    canvas.fill_round_rect(Rect::new(0.0, 0.0, w, h), 0.0, rgb(0x11_13_18));
    let scale = (w * 0.4 / WIDTH).min(h * 0.4 / HEIGHT);
    let origin = ((w - WIDTH * scale) / 2.0, (h - HEIGHT * scale) / 2.0);
    let pen = (scale * 0.045).max(1.5);
    render(canvas, origin, scale, progress, pen, rgb(0xF4_F2_ED));

    // The note and the track sit under the mark, and the track is the pen's own
    // progress rather than a second, unrelated number: one animation, two
    // readings of it.
    let note_px = (w * 0.014).clamp(11.0, 20.0);
    let style = crate::font::Style::new(note_px, rgb(0xB9_BD_C7));
    if !note.is_empty() {
        let nw = crate::font::measure(note, &style);
        crate::font::draw(
            canvas,
            note,
            (w - nw) / 2.0,
            origin.1 + HEIGHT * scale + note_px * 2.4,
            &style,
        );
    }
    let track_w = (w * 0.18).clamp(120.0, 320.0);
    let track_h = 3.0_f32.max(note_px * 0.16);
    let track_y = origin.1 + HEIGHT * scale + note_px * 3.4;
    let track = Rect::new((w - track_w) / 2.0, track_y, track_w, track_h);
    canvas.fill_round_rect(track, track_h / 2.0, rgb(0x2A_2E_38));
    let filled = (track_w * progress.clamp(0.0, 1.0)).max(track_h);
    canvas.fill_round_rect(
        Rect::new(track.x, track.y, filled, track_h),
        track_h / 2.0,
        rgb(0x6F_8B_FF),
    );
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
        let starts: Vec<f32> = strokes()
            .iter()
            .filter_map(|s| s.first().map(|p| p.0))
            .collect();
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
        assert!(
            quarter < full,
            "quarter {quarter} is not less than full {full}"
        );
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
        assert_eq!(
            frame(&mut s).get(640, 740),
            Some(field),
            "the desktop showed through"
        );

        // Mid-write, ink has reached the screen.
        // Half-way through the 1.9 s pen write.
        s.tick(1_000 + 950);
        let inked = frame(&mut s)
            .pixels()
            .iter()
            .filter(|&&p| p != field)
            .count();
        assert!(inked > 0, "nothing was written mid-intro");

        // Once the intro has run its course, the desktop is back.
        s.tick(1_000 + BOOT_INTRO_MS);
        assert_eq!(s.boot_intro_now(), None);
        assert_ne!(
            frame(&mut s).get(640, 740),
            Some(field),
            "the desktop did not return"
        );
    }

    #[test]
    fn reduced_motion_holds_the_finished_mark_then_ends() {
        use crate::shell::BOOT_INTRO_MS;
        let mut s = intro_shell(true);
        s.start_boot_intro(0);
        s.tick(10);
        assert_eq!(s.boot_intro_now(), Some(1.0));
        s.tick(BOOT_INTRO_MS);
        assert_eq!(
            s.boot_intro_now(),
            None,
            "reduced motion never ended the intro"
        );
    }

    #[test]
    fn the_loop_keeps_frames_coming_during_the_intro() {
        let mut s = intro_shell(false);
        s.start_boot_intro(0);
        s.tick(0);
        assert!(
            s.wake_after_ms(100) <= 33,
            "the intro would stall at idle pace"
        );
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
