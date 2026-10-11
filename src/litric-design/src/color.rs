//! Colour, luminance and contrast.
//!
//! Everything here exists to answer one question the design system must be able
//! to answer mechanically: **can a person read this text on that surface?**
//!
//! Contrast is computed to WCAG 2.1, which defines relative luminance with a
//! gamma decode and a fixed set of channel weights. It is not a matter of taste
//! and it is not something to eyeball on a good monitor in a bright room.

use core::fmt;

/// An 8-bit sRGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Rgb {
    /// Construct from channels.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Construct from `0xRRGGBB`.
    #[must_use]
    pub const fn hex(v: u32) -> Self {
        Self {
            r: ((v >> 16) & 0xFF) as u8,
            g: ((v >> 8) & 0xFF) as u8,
            b: (v & 0xFF) as u8,
        }
    }

    /// The `#RRGGBB` form, for stylesheets and for the mockups.
    #[must_use]
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// WCAG 2.1 relative luminance, 0.0 (black) to 1.0 (white).
    ///
    /// The gamma decode matters: averaging raw 8-bit channels gives a number
    /// that looks reasonable and is wrong, because sRGB is not linear in
    /// perceived brightness.
    #[must_use]
    pub fn luminance(self) -> f64 {
        fn channel(c: u8) -> f64 {
            let c = f64::from(c) / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(self.r) + 0.7152 * channel(self.g) + 0.0722 * channel(self.b)
    }

    /// WCAG contrast ratio against another colour, from 1.0 to 21.0.
    ///
    /// Symmetric: the order of the two colours does not matter.
    #[must_use]
    pub fn contrast(self, other: Self) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Composite `self` over `backdrop` at `alpha` (0.0–1.0).
    ///
    /// Straight alpha in sRGB space. This is what a compositor actually does for
    /// a translucent surface, so it is what the contrast checks must model —
    /// checking the tint colour alone would pass a glass panel that is
    /// unreadable in practice.
    #[must_use]
    pub fn over(self, backdrop: Self, alpha: f64) -> Self {
        let a = alpha.clamp(0.0, 1.0);
        let mix = |fg: u8, bg: u8| -> u8 {
            let v = f64::from(fg) * a + f64::from(bg) * (1.0 - a);
            v.round().clamp(0.0, 255.0) as u8
        };
        Self {
            r: mix(self.r, backdrop.r),
            g: mix(self.g, backdrop.g),
            b: mix(self.b, backdrop.b),
        }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// WCAG conformance level for a text/background pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Conformance {
    /// Below 3.0. Not acceptable for any text.
    Fail,
    /// 3.0+. Large text only, and non-text UI elements.
    AaLarge,
    /// 4.5+. The minimum for body text.
    Aa,
    /// 7.0+. Required by the high-contrast theme.
    Aaa,
}

impl Conformance {
    /// Classify a contrast ratio.
    #[must_use]
    pub fn of(ratio: f64) -> Self {
        if ratio >= 7.0 {
            Self::Aaa
        } else if ratio >= 4.5 {
            Self::Aa
        } else if ratio >= 3.0 {
            Self::AaLarge
        } else {
            Self::Fail
        }
    }

    /// Whether body text at this level is acceptable.
    #[must_use]
    pub fn passes_body_text(self) -> bool {
        self >= Self::Aa
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips() {
        for v in [0x000000u32, 0xFFFFFF, 0x0A0B0D, 0x7F3FBF] {
            assert_eq!(Rgb::hex(v).to_hex(), format!("#{v:06x}"));
        }
    }

    #[test]
    fn luminance_matches_the_wcag_reference_points() {
        // Black and white are the anchors; if the gamma decode is wrong these
        // still pass, which is why the mid-grey case below matters.
        assert!((Rgb::hex(0x000000).luminance() - 0.0).abs() < 1e-9);
        assert!((Rgb::hex(0xFFFFFF).luminance() - 1.0).abs() < 1e-9);

        // 50% grey (#808080) has luminance ~0.2159, NOT 0.5. A linear average
        // would give 0.502 and every contrast decision downstream would be wrong.
        let mid = Rgb::hex(0x808080).luminance();
        assert!((mid - 0.2159).abs() < 0.001, "mid grey luminance was {mid}");
    }

    #[test]
    fn contrast_black_on_white_is_the_maximum() {
        let r = Rgb::hex(0x000000).contrast(Rgb::hex(0xFFFFFF));
        assert!((r - 21.0).abs() < 1e-6, "expected 21.0, got {r}");
    }

    #[test]
    fn contrast_is_symmetric_and_self_is_one() {
        let a = Rgb::hex(0x1A2B3C);
        let b = Rgb::hex(0xEEDDCC);
        assert!((a.contrast(b) - b.contrast(a)).abs() < 1e-12);
        assert!((a.contrast(a) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn conformance_thresholds_are_the_wcag_ones() {
        assert_eq!(Conformance::of(21.0), Conformance::Aaa);
        assert_eq!(Conformance::of(7.0), Conformance::Aaa);
        assert_eq!(Conformance::of(6.99), Conformance::Aa);
        assert_eq!(Conformance::of(4.5), Conformance::Aa);
        assert_eq!(Conformance::of(4.49), Conformance::AaLarge);
        assert_eq!(Conformance::of(3.0), Conformance::AaLarge);
        assert_eq!(Conformance::of(2.99), Conformance::Fail);

        assert!(Conformance::of(4.5).passes_body_text());
        assert!(!Conformance::of(4.4).passes_body_text());
    }

    #[test]
    fn compositing_at_the_extremes_is_exact() {
        let fg = Rgb::hex(0xFF0000);
        let bg = Rgb::hex(0x0000FF);
        assert_eq!(fg.over(bg, 1.0), fg, "alpha 1 is the foreground");
        assert_eq!(fg.over(bg, 0.0), bg, "alpha 0 is the backdrop");
    }

    #[test]
    fn compositing_is_clamped_against_out_of_range_alpha() {
        // Untrusted or computed alpha must not produce a wrapped channel.
        let fg = Rgb::hex(0xFFFFFF);
        let bg = Rgb::hex(0x000000);
        assert_eq!(fg.over(bg, 2.0), fg);
        assert_eq!(fg.over(bg, -1.0), bg);
        assert_eq!(fg.over(bg, f64::NAN.max(0.0)), bg);
    }

    #[test]
    fn compositing_moves_luminance_monotonically() {
        // A property rather than an example: as alpha rises, the result moves
        // steadily from backdrop toward foreground and never overshoots.
        let fg = Rgb::hex(0xFFFFFF);
        let bg = Rgb::hex(0x101014);
        let mut previous = bg.luminance();
        for step in 1..=10 {
            let a = f64::from(step) / 10.0;
            let l = fg.over(bg, a).luminance();
            assert!(
                l >= previous - 1e-9,
                "luminance went backwards at alpha {a}"
            );
            assert!(l <= fg.luminance() + 1e-9);
            previous = l;
        }
    }
}
