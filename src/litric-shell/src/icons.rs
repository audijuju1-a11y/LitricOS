//! The LitricOS icon set.
//!
//! Icons are described the way the typeface is: as the path a pen takes over a
//! normalised box, drawn at any size by the same anti-aliased line renderer.
//! That is not a shortcut around not having images — it is what makes an icon
//! set consistent. One stroke weight, one corner radius, one optical size, all
//! enforced by the drawing code rather than by whoever exported the PNGs.
//!
//! ```text
//!   (0,0) ┌─────────┐
//!         │         │   Every icon is drawn in a unit box and scaled.
//!         │         │   y increases downward, as on the canvas.
//!         └─────────┘ (1,1)
//! ```
//!
//! **No emoji.** An emoji is somebody else's typeface, rendered at whatever
//! size and colour that font decided, and it changes between systems. An
//! operating system's own iconography cannot be borrowed from a font it does
//! not ship.

use crate::canvas::{arc_points, Canvas, Rect};

/// Every icon the shell can draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Icon {
    /// The LitricOS mark, used in the top bar.
    Litric,
    /// A folder: the file manager.
    Files,
    /// A globe: the browser.
    Browser,
    /// A prompt: the terminal.
    Terminal,
    /// A dial: settings.
    Settings,
    /// A sparkline: activity and the system monitor.
    Activity,
    /// A bin: the trash.
    Trash,
    /// Stacked panes: environments.
    Environment,
    /// A grid: workspaces and the launcher.
    Grid,
    /// A magnifier: search.
    Search,
    /// A bell: notifications.
    Notifications,
    /// Wi-Fi arcs.
    Network,
    /// The Bluetooth rune.
    Bluetooth,
    /// A speaker: audio.
    Audio,
    /// A speaker with a slash: muted.
    AudioMuted,
    /// A battery.
    Battery,
    /// A bolt: power and performance.
    Power,
    /// A chevron pointing down: a menu that opens.
    ChevronDown,
    /// A chevron pointing right.
    ChevronRight,
    /// A cross: close.
    Close,
    /// A minus: minimize.
    Minimize,
    /// A square: maximize.
    Maximize,
    /// A tick: a setting that is on.
    Check,
    /// A warning triangle.
    Warning,
    /// An information circle.
    Info,
    /// A padlock: locked, or secure.
    Lock,
    /// A person: the user menu.
    User,
    /// A window outline: the task switcher.
    Window,
    /// A CPU die.
    Cpu,
    /// Stacked discs: memory and storage.
    Memory,
    /// A shield: Litric Defender, scanning or at rest.
    Shield,
    /// A shield with a tick: the last scan found nothing.
    ShieldCheck,
}

impl Icon {
    /// A stable name, for diagnostics and for the settings application.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Litric => "litric",
            Self::Files => "files",
            Self::Browser => "browser",
            Self::Terminal => "terminal",
            Self::Settings => "settings",
            Self::Activity => "activity",
            Self::Trash => "trash",
            Self::Environment => "environment",
            Self::Grid => "grid",
            Self::Search => "search",
            Self::Notifications => "notifications",
            Self::Network => "network",
            Self::Bluetooth => "bluetooth",
            Self::Audio => "audio",
            Self::AudioMuted => "audio-muted",
            Self::Battery => "battery",
            Self::Power => "power",
            Self::ChevronDown => "chevron-down",
            Self::ChevronRight => "chevron-right",
            Self::Close => "close",
            Self::Minimize => "minimize",
            Self::Maximize => "maximize",
            Self::Check => "check",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Lock => "lock",
            Self::User => "user",
            Self::Window => "window",
            Self::Cpu => "cpu",
            Self::Memory => "memory",
            Self::Shield => "shield",
            Self::ShieldCheck => "shield-check",
        }
    }

    /// Every icon, for the specimen sheet and the tests.
    pub const ALL: [Self; 32] = [
        Self::Litric,
        Self::Files,
        Self::Browser,
        Self::Terminal,
        Self::Settings,
        Self::Activity,
        Self::Trash,
        Self::Environment,
        Self::Grid,
        Self::Search,
        Self::Notifications,
        Self::Network,
        Self::Bluetooth,
        Self::Audio,
        Self::AudioMuted,
        Self::Battery,
        Self::Power,
        Self::ChevronDown,
        Self::ChevronRight,
        Self::Close,
        Self::Minimize,
        Self::Maximize,
        Self::Check,
        Self::Warning,
        Self::Info,
        Self::Lock,
        Self::User,
        Self::Window,
        Self::Cpu,
        Self::Memory,
        Self::Shield,
        Self::ShieldCheck,
    ];
}

/// The stroke weight of an icon as a fraction of its box, matching the
/// typeface's regular weight so text and icons sit together.
pub const STROKE: f32 = 0.085;

/// Draw an icon inside a box.
///
/// `weight` is a multiplier on [`STROKE`]; 1.0 is regular. The line width never
/// falls below one pixel, because a half-pixel icon is a grey smudge.
pub fn draw(canvas: &mut Canvas, icon: Icon, box_: Rect, color: u32, weight: f32) {
    let s = box_.w.min(box_.h);
    let lw = (s * STROKE * weight).max(1.0);
    // Everything is authored in a unit box; this maps it to the real one and
    // centres it when the box is not square.
    let ox = box_.x + (box_.w - s) / 2.0;
    let oy = box_.y + (box_.h - s) / 2.0;
    let p = |x: f32, y: f32| -> (f32, f32) { (ox + x * s, oy + y * s) };

    let line = |canvas: &mut Canvas, a: (f32, f32), b: (f32, f32)| {
        let (x0, y0) = p(a.0, a.1);
        let (x1, y1) = p(b.0, b.1);
        canvas.stroke_line(x0, y0, x1, y1, lw, color);
    };
    let poly = |canvas: &mut Canvas, pts: &[(f32, f32)]| {
        let mut prev: Option<(f32, f32)> = None;
        for q in pts {
            let cur = p(q.0, q.1);
            if let Some(a) = prev {
                canvas.stroke_line(a.0, a.1, cur.0, cur.1, lw, color);
            }
            prev = Some(cur);
        }
    };
    let circle = |canvas: &mut Canvas, cx: f32, cy: f32, r: f32| {
        let pts = arc_points(cx, cy, r, r, 0.0, 360.0, 40);
        let mut prev: Option<(f32, f32)> = None;
        for q in &pts {
            let cur = p(q.0, q.1);
            if let Some(a) = prev {
                canvas.stroke_line(a.0, a.1, cur.0, cur.1, lw, color);
            }
            prev = Some(cur);
        }
    };
    let arc = |canvas: &mut Canvas, cx: f32, cy: f32, r: f32, a0: f32, a1: f32| {
        let pts = arc_points(cx, cy, r, r, a0, a1, 24);
        let mut prev: Option<(f32, f32)> = None;
        for q in &pts {
            let cur = p(q.0, q.1);
            if let Some(a) = prev {
                canvas.stroke_line(a.0, a.1, cur.0, cur.1, lw, color);
            }
            prev = Some(cur);
        }
    };
    let rounded = |canvas: &mut Canvas, x: f32, y: f32, w: f32, h: f32, r: f32| {
        let (px, py) = p(x, y);
        canvas.stroke_round_rect(Rect::new(px, py, w * s, h * s), r * s, lw, color);
    };
    let dot = |canvas: &mut Canvas, x: f32, y: f32, r: f32| {
        let (px, py) = p(x, y);
        canvas.fill_circle(px, py, r * s, color);
    };

    match icon {
        // The mark: a rounded square with a diagonal cut, echoing the corner
        // radius used everywhere else in the interface.
        Icon::Litric => {
            rounded(canvas, 0.14, 0.14, 0.72, 0.72, 0.22);
            poly(canvas, &[(0.34, 0.70), (0.34, 0.32), (0.62, 0.32)]);
        }
        Icon::Files => {
            poly(
                canvas,
                &[
                    (0.12, 0.76),
                    (0.12, 0.26),
                    (0.40, 0.26),
                    (0.48, 0.36),
                    (0.88, 0.36),
                    (0.88, 0.76),
                    (0.12, 0.76),
                ],
            );
        }
        Icon::Browser => {
            circle(canvas, 0.5, 0.5, 0.38);
            line(canvas, (0.12, 0.5), (0.88, 0.5));
            // The meridian: an ellipse drawn as two arcs so it reads as a
            // sphere rather than as a circle with a line through it.
            let pts = arc_points(0.5, 0.5, 0.19, 0.38, -90.0, 90.0, 24);
            let mut prev: Option<(f32, f32)> = None;
            for q in &pts {
                let cur = p(q.0, q.1);
                if let Some(a) = prev {
                    canvas.stroke_line(a.0, a.1, cur.0, cur.1, lw, color);
                }
                prev = Some(cur);
            }
            let pts2 = arc_points(0.5, 0.5, 0.19, 0.38, 90.0, 270.0, 24);
            let mut prev2: Option<(f32, f32)> = None;
            for q in &pts2 {
                let cur = p(q.0, q.1);
                if let Some(a) = prev2 {
                    canvas.stroke_line(a.0, a.1, cur.0, cur.1, lw, color);
                }
                prev2 = Some(cur);
            }
        }
        Icon::Terminal => {
            poly(canvas, &[(0.20, 0.30), (0.44, 0.50), (0.20, 0.70)]);
            line(canvas, (0.52, 0.70), (0.82, 0.70));
        }
        Icon::Settings => {
            circle(canvas, 0.5, 0.5, 0.20);
            for (dx, dy) in [(0.0f32, -1.0f32), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
                line(
                    canvas,
                    (0.5 + dx * 0.28, 0.5 + dy * 0.28),
                    (0.5 + dx * 0.42, 0.5 + dy * 0.42),
                );
            }
            for (dx, dy) in [(0.7f32, -0.7f32), (-0.7, 0.7), (0.7, 0.7), (-0.7, -0.7)] {
                line(
                    canvas,
                    (0.5 + dx * 0.28, 0.5 + dy * 0.28),
                    (0.5 + dx * 0.40, 0.5 + dy * 0.40),
                );
            }
        }
        Icon::Activity => {
            poly(
                canvas,
                &[
                    (0.12, 0.62),
                    (0.30, 0.62),
                    (0.40, 0.32),
                    (0.54, 0.72),
                    (0.64, 0.48),
                    (0.72, 0.56),
                    (0.88, 0.56),
                ],
            );
        }
        Icon::Trash => {
            line(canvas, (0.16, 0.28), (0.84, 0.28));
            poly(
                canvas,
                &[(0.38, 0.28), (0.40, 0.16), (0.60, 0.16), (0.62, 0.28)],
            );
            poly(
                canvas,
                &[(0.24, 0.34), (0.30, 0.86), (0.70, 0.86), (0.76, 0.34)],
            );
            line(canvas, (0.42, 0.44), (0.44, 0.76));
            line(canvas, (0.58, 0.44), (0.56, 0.76));
        }
        Icon::Environment => {
            rounded(canvas, 0.10, 0.24, 0.52, 0.52, 0.10);
            rounded(canvas, 0.38, 0.10, 0.52, 0.52, 0.10);
        }
        Icon::Grid => {
            for (cx, cy) in [(0.28f32, 0.28f32), (0.72, 0.28), (0.28, 0.72), (0.72, 0.72)] {
                rounded(canvas, cx - 0.16, cy - 0.16, 0.32, 0.32, 0.09);
            }
        }
        Icon::Search => {
            circle(canvas, 0.44, 0.44, 0.26);
            line(canvas, (0.63, 0.63), (0.86, 0.86));
        }
        Icon::Notifications => {
            poly(
                canvas,
                &[
                    (0.24, 0.66),
                    (0.30, 0.56),
                    (0.30, 0.42),
                    (0.50, 0.20),
                    (0.70, 0.42),
                    (0.70, 0.56),
                    (0.76, 0.66),
                    (0.24, 0.66),
                ],
            );
            arc(canvas, 0.5, 0.66, 0.11, 0.0, 180.0);
        }
        Icon::Network => {
            // Three arcs and a dot: the shape everybody already reads as
            // wireless, drawn at one weight rather than as filled wedges.
            for r in [0.44f32, 0.30, 0.16] {
                arc(canvas, 0.5, 0.74, r, 200.0, 340.0);
            }
            dot(canvas, 0.5, 0.76, 0.055);
        }
        Icon::Bluetooth => {
            // The rune: a spine with two fins, each meeting it at the same
            // angle. Two polylines rather than one closed path so the crossing
            // stays a crossing instead of collapsing into a triangle.
            line(canvas, (0.5, 0.10), (0.5, 0.90));
            poly(canvas, &[(0.5, 0.10), (0.76, 0.32), (0.26, 0.70)]);
            poly(canvas, &[(0.5, 0.90), (0.76, 0.68), (0.26, 0.30)]);
        }
        Icon::Audio | Icon::AudioMuted => {
            poly(
                canvas,
                &[
                    (0.16, 0.38),
                    (0.30, 0.38),
                    (0.48, 0.20),
                    (0.48, 0.80),
                    (0.30, 0.62),
                    (0.16, 0.62),
                    (0.16, 0.38),
                ],
            );
            if icon == Icon::Audio {
                arc(canvas, 0.52, 0.5, 0.18, -60.0, 60.0);
                arc(canvas, 0.52, 0.5, 0.30, -55.0, 55.0);
            } else {
                line(canvas, (0.62, 0.38), (0.86, 0.62));
                line(canvas, (0.86, 0.38), (0.62, 0.62));
            }
        }
        Icon::Battery => {
            rounded(canvas, 0.10, 0.34, 0.68, 0.32, 0.08);
            line(canvas, (0.84, 0.44), (0.84, 0.56));
            rounded(canvas, 0.16, 0.40, 0.34, 0.20, 0.04);
        }
        Icon::Power => {
            poly(
                canvas,
                &[
                    (0.56, 0.10),
                    (0.30, 0.54),
                    (0.50, 0.54),
                    (0.44, 0.90),
                    (0.70, 0.46),
                    (0.50, 0.46),
                    (0.56, 0.10),
                ],
            );
        }
        Icon::ChevronDown => poly(canvas, &[(0.18, 0.36), (0.50, 0.66), (0.82, 0.36)]),
        Icon::ChevronRight => poly(canvas, &[(0.36, 0.18), (0.66, 0.50), (0.36, 0.82)]),
        Icon::Close => {
            line(canvas, (0.28, 0.28), (0.72, 0.72));
            line(canvas, (0.72, 0.28), (0.28, 0.72));
        }
        Icon::Minimize => line(canvas, (0.16, 0.52), (0.84, 0.52)),
        Icon::Maximize => rounded(canvas, 0.26, 0.26, 0.48, 0.48, 0.08),
        Icon::Check => poly(canvas, &[(0.22, 0.52), (0.42, 0.72), (0.78, 0.30)]),
        Icon::Warning => {
            poly(
                canvas,
                &[(0.50, 0.16), (0.90, 0.82), (0.10, 0.82), (0.50, 0.16)],
            );
            line(canvas, (0.50, 0.40), (0.50, 0.60));
            dot(canvas, 0.50, 0.71, 0.045);
        }
        Icon::Info => {
            circle(canvas, 0.5, 0.5, 0.38);
            line(canvas, (0.50, 0.46), (0.50, 0.70));
            dot(canvas, 0.50, 0.33, 0.045);
        }
        Icon::Lock => {
            rounded(canvas, 0.22, 0.46, 0.56, 0.38, 0.09);
            arc(canvas, 0.50, 0.46, 0.18, 180.0, 360.0);
        }
        Icon::User => {
            circle(canvas, 0.5, 0.34, 0.17);
            arc(canvas, 0.5, 0.90, 0.30, 180.0, 360.0);
        }
        Icon::Window => {
            rounded(canvas, 0.12, 0.20, 0.76, 0.60, 0.09);
            line(canvas, (0.12, 0.36), (0.88, 0.36));
        }
        Icon::Cpu => {
            rounded(canvas, 0.26, 0.26, 0.48, 0.48, 0.07);
            rounded(canvas, 0.40, 0.40, 0.20, 0.20, 0.04);
            for t in [0.36f32, 0.50, 0.64] {
                line(canvas, (t, 0.26), (t, 0.14));
                line(canvas, (t, 0.74), (t, 0.86));
                line(canvas, (0.26, t), (0.14, t));
                line(canvas, (0.74, t), (0.86, t));
            }
        }
        Icon::Memory => {
            for y in [0.30f32, 0.50, 0.70] {
                rounded(canvas, 0.14, y - 0.09, 0.72, 0.18, 0.05);
            }
        }
        Icon::Shield => {
            poly(
                canvas,
                &[
                    (0.50, 0.12),
                    (0.84, 0.26),
                    (0.80, 0.58),
                    (0.50, 0.88),
                    (0.20, 0.58),
                    (0.16, 0.26),
                    (0.50, 0.12),
                ],
            );
        }
        Icon::ShieldCheck => {
            poly(
                canvas,
                &[
                    (0.50, 0.12),
                    (0.84, 0.26),
                    (0.80, 0.58),
                    (0.50, 0.88),
                    (0.20, 0.58),
                    (0.16, 0.26),
                    (0.50, 0.12),
                ],
            );
            poly(canvas, &[(0.34, 0.48), (0.46, 0.60), (0.68, 0.34)]);
        }
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::canvas::{luminance, rgb};

    fn ink(icon: Icon, size: u32) -> (usize, (i64, i64, i64, i64)) {
        let mut c = Canvas::new(size, size, rgb(0x000000));
        draw(
            &mut c,
            icon,
            Rect::new(0.0, 0.0, size as f32, size as f32),
            rgb(0xFFFFFF),
            1.0,
        );
        let (mut minx, mut miny, mut maxx, mut maxy) = (i64::MAX, i64::MAX, -1i64, -1i64);
        let mut n = 0usize;
        for y in 0..i64::from(size) {
            for x in 0..i64::from(size) {
                if luminance(c.get(x, y).unwrap_or(0)) > 0.15 {
                    n += 1;
                    minx = minx.min(x);
                    miny = miny.min(y);
                    maxx = maxx.max(x);
                    maxy = maxy.max(y);
                }
            }
        }
        (n, (minx, miny, maxx, maxy))
    }

    #[test]
    fn every_icon_draws_something() {
        for i in Icon::ALL {
            let (n, _) = ink(i, 48);
            assert!(n > 20, "{} drew only {n} pixels", i.name());
        }
    }

    #[test]
    fn every_icon_stays_inside_its_box() {
        // An icon that overruns its box collides with its neighbour in the top
        // bar, which looks like a layout bug and is an icon bug.
        for i in Icon::ALL {
            let (_, (minx, miny, maxx, maxy)) = ink(i, 64);
            assert!(minx >= 0 && miny >= 0, "{} starts outside", i.name());
            assert!(
                maxx <= 63 && maxy <= 63,
                "{} runs to {maxx},{maxy} in a 64px box",
                i.name()
            );
        }
    }

    #[test]
    fn every_icon_fills_a_reasonable_share_of_its_box() {
        // Optical sizing: an icon that occupies a quarter of its box looks
        // tiny beside one that fills it, and the top bar looks ragged.
        for i in Icon::ALL {
            let (_, (minx, miny, maxx, maxy)) = ink(i, 64);
            let w = maxx - minx;
            let h = maxy - miny;
            // The long axis has to fill the box; the short one only has to be
            // substantial. A chevron and a minus are legitimately wide and
            // flat, and demanding a square from them would make them look
            // wrong rather than consistent.
            // 32 of 64 rather than something larger: the window-control
            // glyphs (close, minimize, maximize) are deliberately inset,
            // because they are drawn inside a small button rather than
            // standing alone in a launcher tile.
            assert!(
                w.max(h) >= 32,
                "{} spans only {w}x{h} in a 64px box",
                i.name()
            );
            // No lower bound on the short axis: a minus sign is one stroke
            // tall and that is what a minus sign is. The long axis is the
            // optical size, and it is the one that has to be consistent.
            assert!(w.min(h) >= 1, "{} drew a zero-height mark", i.name());
        }
    }

    #[test]
    fn icons_are_legible_at_top_bar_size() {
        // 16 logical pixels is what the status area uses. An icon that becomes
        // a smudge there is an icon that cannot be used where it is needed.
        for i in Icon::ALL {
            let (n, _) = ink(i, 16);
            assert!(n >= 6, "{} is {n} pixels at 16px", i.name());
        }
    }

    #[test]
    fn a_heavier_weight_puts_down_more_ink() {
        let count = |w: f32| -> usize {
            let mut c = Canvas::new(48, 48, rgb(0x000000));
            draw(
                &mut c,
                Icon::Settings,
                Rect::new(0.0, 0.0, 48.0, 48.0),
                rgb(0xFFFFFF),
                w,
            );
            (0..48i64)
                .flat_map(|y| (0..48i64).map(move |x| (x, y)))
                .filter(|(x, y)| luminance(c.get(*x, *y).unwrap_or(0)) > 0.5)
                .count()
        };
        assert!(count(2.0) > count(1.0));
    }

    #[test]
    fn a_non_square_box_centres_the_icon_rather_than_stretching_it() {
        let mut c = Canvas::new(120, 40, rgb(0x000000));
        draw(
            &mut c,
            Icon::Close,
            Rect::new(0.0, 0.0, 120.0, 40.0),
            rgb(0xFFFFFF),
            1.0,
        );
        let mut minx = i64::MAX;
        let mut maxx = -1i64;
        for y in 0..40i64 {
            for x in 0..120i64 {
                if luminance(c.get(x, y).unwrap_or(0)) > 0.15 {
                    minx = minx.min(x);
                    maxx = maxx.max(x);
                }
            }
        }
        let centre = (minx + maxx).checked_div(2).unwrap_or(0);
        assert!((centre - 60).abs() <= 2, "icon centred at {centre}, not 60");
        assert!(maxx - minx <= 40, "icon was stretched to the box width");
    }

    #[test]
    fn every_icon_has_a_distinct_name() {
        let mut names: Vec<&str> = Icon::ALL.iter().map(|i| i.name()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "two icons share a name");
        // The literal is deliberate: adding an icon means looking at this line
        // and deciding where it belongs in the set, not only appending it.
        assert_eq!(before, 32);
    }

    #[test]
    fn muted_audio_differs_from_audio() {
        // They share a body; if the slash were missing they would be identical
        // and the status area would be unable to show the difference.
        let (a, _) = ink(Icon::Audio, 48);
        let (m, _) = ink(Icon::AudioMuted, 48);
        assert_ne!(a, m);
    }
}
