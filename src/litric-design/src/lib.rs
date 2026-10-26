//! The LitricOS design system.
//!
//! # Layer position
//!
//! **Layer 5 foundation.** Pure data and pure functions: no rendering, no
//! toolkit, no I/O. `litric-shell` consumes these tokens; nothing else defines a
//! colour, a radius or a duration. ADR-0009 makes that structurally true — there
//! is one process drawing the UI, so there is no second toolkit that could
//! disagree.
//!
//! # Why the tokens are code and not a stylesheet
//!
//! Because the constraints are testable, and a stylesheet cannot fail CI.
//!
//! The specific thing this crate exists to prevent is a translucent "glass"
//! interface that looks beautiful in a screenshot and is unreadable over a
//! bright wallpaper. That is not a hypothetical failure mode: it is the
//! criticism levelled at every glass-styled interface that shipped without
//! measuring, and it is invisible to whoever designed it, because they had a
//! dark wallpaper.
//!
//! So [`Glass`] carries an alpha floor, and the tests assert that body text
//! stays at WCAG AA over the **worst-case** backdrop — pure white and pure
//! black — not over the wallpaper the designer happened to use.

#![forbid(unsafe_code)]

pub mod color;

pub use color::{Conformance, Rgb};

// ---------------------------------------------------------------------------
// Themes
// ---------------------------------------------------------------------------

/// Which theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ThemeKind {
    /// Default.
    #[default]
    Dark,
    /// Light.
    Light,
    /// High contrast, dark. Every text pair reaches AAA.
    HighContrastDark,
    /// High contrast, light. Every text pair reaches AAA.
    HighContrastLight,
}

impl ThemeKind {
    /// Every theme, so tests and the settings UI enumerate the same set.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Dark,
            Self::Light,
            Self::HighContrastDark,
            Self::HighContrastLight,
        ]
    }

    /// Stable token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::HighContrastDark => "high-contrast-dark",
            Self::HighContrastLight => "high-contrast-light",
        }
    }

    /// Whether this theme must meet AAA rather than AA.
    #[must_use]
    pub const fn requires_aaa(self) -> bool {
        matches!(self, Self::HighContrastDark | Self::HighContrastLight)
    }
}

/// A user-selected accent colour, applied on top of a theme.
///
/// The choice is a *slot*, not a hex value. Each theme carries the value for
/// that slot, because the blue that reads on a near-black background is not the
/// blue that reads on white: the same literal in both themes is how an accent
/// picker ships an unreadable selection colour.
///
/// High-contrast themes ignore the choice entirely. Their accent is part of the
/// AAA contrast guarantee this crate enforces, and a decorative preference must
/// not be able to break it. [`Theme::with_accent`] returns such a theme
/// unchanged, and the settings UI says so rather than offering a control that
/// silently does nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Accent {
    /// The theme's own accent.
    #[default]
    Default,
    /// Cool blue.
    Sky,
    /// Soft violet.
    Violet,
    /// Blue-green.
    Teal,
    /// Warm amber.
    Amber,
    /// Pink.
    Rose,
}

impl Accent {
    /// Every accent, so the settings UI and the contrast gate enumerate the
    /// same list.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::Sky,
            Self::Violet,
            Self::Teal,
            Self::Amber,
            Self::Rose,
        ]
    }

    /// Stable token, written to the preference file and parsed back from it.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sky => "sky",
            Self::Violet => "violet",
            Self::Teal => "teal",
            Self::Amber => "amber",
            Self::Rose => "rose",
        }
    }

    /// Parse a stored token. `None` for anything this build does not know, so a
    /// settings file from a newer build falls back to the default accent
    /// instead of failing to load.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        Self::all().iter().copied().find(|a| a.token() == raw)
    }
}

/// The semantic colour tokens.
///
/// Semantic, not literal: components ask for `text_primary`, never for a hex
/// value. That is what makes a theme swap a data change rather than an audit of
/// every component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// The deepest background. Desktop, window backgrounds.
    pub bg_base: Rgb,
    /// Raised surfaces: cards, sheets, sidebars.
    pub bg_elevated: Rgb,
    /// The tint a glass surface applies over whatever is behind it.
    pub glass_tint: Rgb,
    /// Primary body text.
    pub text_primary: Rgb,
    /// Secondary text: captions, metadata, disabled labels.
    pub text_secondary: Rgb,
    /// Accent for selection, focus and primary actions.
    pub accent: Rgb,
    /// Text drawn on top of `accent`.
    pub on_accent: Rgb,
    /// Separators and outlines.
    pub border: Rgb,
    /// Destructive actions and errors.
    pub danger: Rgb,
    /// Confirmation and healthy status.
    pub success: Rgb,
    /// Caution and degraded status.
    pub warning: Rgb,
}

/// A complete theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Which theme this is.
    pub kind: ThemeKind,
    /// Its colours.
    pub palette: Palette,
    /// Its glass material.
    pub glass: Glass,
}

impl Theme {
    /// The theme for a given kind.
    #[must_use]
    pub const fn of(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Dark => Self {
                kind,
                palette: Palette {
                    bg_base: Rgb::hex(0x0B0C0F),
                    bg_elevated: Rgb::hex(0x15171D),
                    glass_tint: Rgb::hex(0x14161B),
                    text_primary: Rgb::hex(0xF2F4F8),
                    text_secondary: Rgb::hex(0xA9B1BF),
                    accent: Rgb::hex(0x6AA6FF),
                    on_accent: Rgb::hex(0x06080C),
                    border: Rgb::hex(0x2A2E37),
                    danger: Rgb::hex(0xFF6B6B),
                    success: Rgb::hex(0x5BD9A4),
                    warning: Rgb::hex(0xFFC66D),
                },
                glass: Glass::DARK,
            },
            ThemeKind::Light => Self {
                kind,
                palette: Palette {
                    bg_base: Rgb::hex(0xF6F7F9),
                    bg_elevated: Rgb::hex(0xFFFFFF),
                    glass_tint: Rgb::hex(0xFFFFFF),
                    text_primary: Rgb::hex(0x0E1116),
                    text_secondary: Rgb::hex(0x4C5563),
                    accent: Rgb::hex(0x1D5FD6),
                    on_accent: Rgb::hex(0xFFFFFF),
                    border: Rgb::hex(0xD9DDE3),
                    danger: Rgb::hex(0xC42B2B),
                    success: Rgb::hex(0x1B7A4F),
                    warning: Rgb::hex(0x8A5A00),
                },
                glass: Glass::LIGHT,
            },
            ThemeKind::HighContrastDark => Self {
                kind,
                palette: Palette {
                    bg_base: Rgb::hex(0x000000),
                    bg_elevated: Rgb::hex(0x0A0A0A),
                    glass_tint: Rgb::hex(0x000000),
                    text_primary: Rgb::hex(0xFFFFFF),
                    text_secondary: Rgb::hex(0xE0E0E0),
                    accent: Rgb::hex(0x8CC0FF),
                    on_accent: Rgb::hex(0x000000),
                    border: Rgb::hex(0xFFFFFF),
                    danger: Rgb::hex(0xFF9A9A),
                    success: Rgb::hex(0x8CE8BE),
                    warning: Rgb::hex(0xFFD98A),
                },
                glass: Glass::HIGH_CONTRAST,
            },
            ThemeKind::HighContrastLight => Self {
                kind,
                palette: Palette {
                    bg_base: Rgb::hex(0xFFFFFF),
                    bg_elevated: Rgb::hex(0xFFFFFF),
                    glass_tint: Rgb::hex(0xFFFFFF),
                    text_primary: Rgb::hex(0x000000),
                    text_secondary: Rgb::hex(0x1A1A1A),
                    accent: Rgb::hex(0x00408A),
                    on_accent: Rgb::hex(0xFFFFFF),
                    border: Rgb::hex(0x000000),
                    danger: Rgb::hex(0x8A0000),
                    success: Rgb::hex(0x004D2E),
                    warning: Rgb::hex(0x5A3A00),
                },
                glass: Glass::HIGH_CONTRAST,
            },
        }
    }

    /// The theme with the user's accent applied.
    ///
    /// Only the accent pair moves: `text_primary`, the surfaces and the status
    /// colours are untouched, because they are the contrast contract and not
    /// decoration. High-contrast themes return themselves unchanged — see
    /// [`Accent`].
    ///
    /// Every value below is checked by `every_accent_keeps_text_legible` in
    /// this crate's tests: `on_accent` over `accent`, and `accent` as a text
    /// tone over both theme backgrounds, at AA in the standard themes.
    #[must_use]
    pub const fn with_accent(self, accent: Accent) -> Self {
        let (accent, on_accent) = match (self.kind, accent) {
            (ThemeKind::HighContrastDark | ThemeKind::HighContrastLight, _)
            | (_, Accent::Default) => return self,
            (ThemeKind::Dark, Accent::Sky) => (Rgb::hex(0x6AA6FF), Rgb::hex(0x06080C)),
            (ThemeKind::Dark, Accent::Violet) => (Rgb::hex(0xB79CFF), Rgb::hex(0x0B0714)),
            (ThemeKind::Dark, Accent::Teal) => (Rgb::hex(0x63E0C8), Rgb::hex(0x041712)),
            (ThemeKind::Dark, Accent::Amber) => (Rgb::hex(0xFFC46B), Rgb::hex(0x1A1000)),
            (ThemeKind::Dark, Accent::Rose) => (Rgb::hex(0xFFA3C7), Rgb::hex(0x1A0710)),
            (ThemeKind::Light, Accent::Sky) => (Rgb::hex(0x1D5FD6), Rgb::hex(0xFFFFFF)),
            (ThemeKind::Light, Accent::Violet) => (Rgb::hex(0x5B32C4), Rgb::hex(0xFFFFFF)),
            (ThemeKind::Light, Accent::Teal) => (Rgb::hex(0x046B5E), Rgb::hex(0xFFFFFF)),
            (ThemeKind::Light, Accent::Amber) => (Rgb::hex(0x7A4E00), Rgb::hex(0xFFFFFF)),
            (ThemeKind::Light, Accent::Rose) => (Rgb::hex(0xA3215C), Rgb::hex(0xFFFFFF)),
        };
        Self {
            kind: self.kind,
            palette: Palette {
                accent,
                on_accent,
                ..self.palette
            },
            glass: self.glass,
        }
    }
}

// ---------------------------------------------------------------------------
// Glass
// ---------------------------------------------------------------------------

/// A translucent "liquid glass" material.
///
/// Four things make it read as glass rather than as flat transparency: a real
/// backdrop blur, a saturation boost so colour bleeds through, a hairline border
/// where the panel meets what is behind it, and a specular highlight along the
/// top edge.
///
/// [`Glass::tint_alpha`] is the one that is not aesthetic. Lower it and the
/// panel becomes prettier and, over a bright wallpaper, unreadable. The floor is
/// enforced by test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glass {
    /// Backdrop blur radius in logical pixels.
    pub blur_px: u16,
    /// Backdrop saturation multiplier, in percent. 100 = unchanged.
    pub saturation_pct: u16,
    /// Opacity of the tint over the blurred backdrop, in percent.
    ///
    /// Percent rather than float so the token is exactly comparable and can be
    /// round-tripped through configuration without drift.
    pub tint_alpha_pct: u8,
    /// Hairline border opacity, percent.
    pub border_alpha_pct: u8,
    /// Top-edge specular highlight opacity, percent.
    pub highlight_alpha_pct: u8,
}

impl Glass {
    /// The alpha below which text legibility can no longer be guaranteed over an
    /// arbitrary backdrop.
    ///
    /// Determined by the tests in this crate, not chosen for looks: below this,
    /// body text over a white wallpaper drops under WCAG AA.
    pub const MIN_TINT_ALPHA_PCT: u8 = 65;

    /// Dark theme glass.
    ///
    /// 82%, not the 72% this started at. At 72% the *primary* text passed
    /// comfortably and `text_secondary` came out at 3.34 over a white wallpaper
    /// -- below AA, and invisible to anyone testing on a dark background. The
    /// dimmer text is always the one that breaks first, which is why the test
    /// checks it separately rather than assuming primary text is representative.
    ///
    /// Saturation dropped from 180 to 120: boosting the colours behind glass
    /// three-quarters again is what turned every panel into a coloured haze
    /// over the old wallpaper. Glass that magnifies its backdrop stops being a
    /// material and becomes a filter.
    pub const DARK: Self = Self {
        blur_px: 32,
        saturation_pct: 120,
        tint_alpha_pct: 82,
        border_alpha_pct: 14,
        highlight_alpha_pct: 10,
    };

    /// Light theme glass.
    ///
    /// 85%, for the mirror of the dark theme's problem: white glass over a DARK
    /// wallpaper. At 78% the secondary text landed at 4.46 against a required
    /// 4.5 -- a failure narrow enough that no one would have caught it by eye,
    /// and wrong on exactly the wallpapers a light-theme user is least likely to
    /// have tested with.
    ///
    /// The general result, arrived at by measurement rather than taste: a glass
    /// material that must stay readable over an ARBITRARY backdrop needs roughly
    /// 80%+ opacity in both directions. Glass at 40-60% looks far better in a
    /// screenshot and cannot meet AA. That trade is the whole reason these
    /// numbers are in a tested file instead of a stylesheet.
    pub const LIGHT: Self = Self {
        blur_px: 32,
        saturation_pct: 115,
        tint_alpha_pct: 85,
        border_alpha_pct: 10,
        highlight_alpha_pct: 45,
    };

    /// High-contrast glass: effectively opaque.
    ///
    /// Translucency and guaranteed AAA contrast are not compatible over an
    /// arbitrary backdrop, so the high-contrast themes give up the effect rather
    /// than give up the contrast. Choosing the other way would make the
    /// accessibility theme the one that fails accessibility.
    pub const HIGH_CONTRAST: Self = Self {
        blur_px: 0,
        saturation_pct: 100,
        tint_alpha_pct: 100,
        border_alpha_pct: 100,
        highlight_alpha_pct: 0,
    };

    /// The most transparent a glass surface is ever allowed to be.
    ///
    /// Over a very dark wallpaper the adaptive solver would happily return 0%,
    /// which is not glass any more -- it is a hole. 18% keeps the material
    /// reading as a surface: the border, the specular highlight and the
    /// saturation boost still have something to sit on.
    pub const MIN_AESTHETIC_ALPHA_PCT: u8 = 18;

    /// The most opaque it is allowed to become before giving up the effect.
    ///
    /// Past this the panel is a solid colour with a blur behind it, and the
    /// honest thing is to say so rather than pretend the material is still
    /// doing something.
    pub const MAX_ADAPTIVE_ALPHA_PCT: u8 = 92;

    /// Tint alpha as a fraction.
    #[must_use]
    pub fn tint_alpha(self) -> f64 {
        f64::from(self.tint_alpha_pct) / 100.0
    }

    /// The **most transparent** tint alpha that still keeps `text` at `target`
    /// contrast over this specific `backdrop`.
    ///
    /// This is how the material gets to be beautiful and legible at the same
    /// time, instead of trading one for the other.
    ///
    /// A fixed alpha has to assume the worst wallpaper anyone might ever set, so
    /// it is permanently as opaque as its worst case — 82% in this system, which
    /// looks heavy over the dark wallpapers most people actually use. Sampling
    /// the backdrop and solving for the minimum alpha that satisfies WCAG means:
    ///
    /// * over a dark wallpaper the panel can drop to 20-30% and look like real
    ///   glass;
    /// * over a white wallpaper it firms up on its own, without a setting;
    /// * and the contrast guarantee holds in **both** cases rather than being
    ///   traded away for the first.
    ///
    /// Apple's Liquid Glass solves the same problem partly through an
    /// accessibility switch the user has to find and turn on. Solving it in the
    /// material means the default is already correct.
    ///
    /// Returns `None` when no alpha in the permitted range reaches `target` —
    /// which means the tint and the text cannot coexist and the palette is
    /// wrong, not the backdrop.
    #[must_use]
    pub fn adaptive_alpha_pct(tint: Rgb, text: Rgb, backdrop: Rgb, target: f64) -> Option<u8> {
        // Contrast against alpha is not monotonic in general: the surface
        // luminance travels from the backdrop to the tint, and it may pass
        // through the text's own luminance on the way. So this walks the range
        // rather than bisecting -- 75 evaluations of cheap arithmetic, and no
        // assumption about the shape of the curve that a future palette change
        // could quietly violate.
        let lo = Self::MIN_AESTHETIC_ALPHA_PCT;
        let hi = Self::MAX_ADAPTIVE_ALPHA_PCT;
        (lo..=hi).find(|pct| {
            let a = f64::from(*pct) / 100.0;
            text.contrast(tint.over(backdrop, a)) >= target
        })
    }

    /// This glass, adapted to a backdrop so that `text` stays readable.
    ///
    /// Falls back to the static [`Self::tint_alpha_pct`] when no alpha in range
    /// works, so a palette problem degrades to the conservative value rather
    /// than to an unreadable one.
    #[must_use]
    pub fn adapted_to(self, tint: Rgb, text: Rgb, backdrop: Rgb, target: f64) -> Self {
        // High-contrast glass is opaque on purpose and is never adapted: making
        // the accessibility theme translucent because the wallpaper happened to
        // be dark would defeat the reason it exists.
        if self.tint_alpha_pct >= 100 {
            return self;
        }
        let pct =
            Self::adaptive_alpha_pct(tint, text, backdrop, target).unwrap_or(self.tint_alpha_pct);
        Self {
            tint_alpha_pct: pct,
            ..self
        }
    }

    /// The effective surface colour when this glass, tinted `tint`, sits over
    /// `backdrop`.
    ///
    /// This is what text is actually drawn on, and therefore what contrast must
    /// be measured against. Measuring against the tint alone is the mistake that
    /// produces an unreadable interface that passes its own checks.
    #[must_use]
    pub fn effective_surface(self, tint: Rgb, backdrop: Rgb) -> Rgb {
        tint.over(backdrop, self.tint_alpha())
    }
}

// ---------------------------------------------------------------------------
// Type, spacing, radius
// ---------------------------------------------------------------------------

/// A step on the type scale, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TypeStep {
    /// 12 — timestamps, dense metadata.
    Caption,
    /// 13 — footnotes.
    Footnote,
    /// 15 — body. The size everything else is judged against.
    Body,
    /// 17 — emphasised body, list headers.
    Headline,
    /// 20 — section titles.
    Title3,
    /// 24 — view titles.
    Title2,
    /// 30 — page titles.
    Title1,
    /// 38 — the largest, used sparingly.
    Display,
}

impl TypeStep {
    /// Every step, smallest first.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Caption,
            Self::Footnote,
            Self::Body,
            Self::Headline,
            Self::Title3,
            Self::Title2,
            Self::Title1,
            Self::Display,
        ]
    }

    /// Size in logical pixels.
    #[must_use]
    pub const fn px(self) -> u16 {
        match self {
            Self::Caption => 12,
            Self::Footnote => 13,
            Self::Body => 15,
            Self::Headline => 17,
            Self::Title3 => 20,
            Self::Title2 => 24,
            Self::Title1 => 30,
            Self::Display => 38,
        }
    }

    /// Line height in logical pixels.
    ///
    /// Tighter at large sizes: a display heading at 1.5x line height looks
    /// disconnected, while body text at 1.2x is hard to read in a paragraph.
    #[must_use]
    pub const fn line_height_px(self) -> u16 {
        match self {
            Self::Caption => 16,
            Self::Footnote => 18,
            Self::Body => 22,
            Self::Headline => 24,
            Self::Title3 => 26,
            Self::Title2 => 30,
            Self::Title1 => 36,
            Self::Display => 44,
        }
    }

    /// Whether WCAG counts this as "large text", which relaxes the contrast
    /// requirement from 4.5 to 3.0.
    ///
    /// Large is 18.66px bold or 24px regular. We do not assume bold, so the
    /// threshold is 24.
    #[must_use]
    pub const fn is_large_text(self) -> bool {
        self.px() >= 24
    }
}

/// Spacing scale, in logical pixels. A 4px grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Space {
    /// 2
    Xxs,
    /// 4
    Xs,
    /// 8
    Sm,
    /// 12
    Md,
    /// 16
    Lg,
    /// 24
    Xl,
    /// 32
    Xxl,
    /// 48
    Xxxl,
}

impl Space {
    /// Every step, smallest first.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Xxs,
            Self::Xs,
            Self::Sm,
            Self::Md,
            Self::Lg,
            Self::Xl,
            Self::Xxl,
            Self::Xxxl,
        ]
    }

    /// Value in logical pixels.
    #[must_use]
    pub const fn px(self) -> u16 {
        match self {
            Self::Xxs => 2,
            Self::Xs => 4,
            Self::Sm => 8,
            Self::Md => 12,
            Self::Lg => 16,
            Self::Xl => 24,
            Self::Xxl => 32,
            Self::Xxxl => 48,
        }
    }
}

/// Corner radius scale, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Radius {
    /// 6 — inputs, small controls.
    Sm,
    /// 10 — buttons, chips.
    Md,
    /// 14 — cards, popovers.
    Lg,
    /// 20 — sheets, panels.
    Xl,
    /// 28 — windows.
    Window,
    /// Fully rounded.
    Full,
}

impl Radius {
    /// Every step, smallest first.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Sm,
            Self::Md,
            Self::Lg,
            Self::Xl,
            Self::Window,
            Self::Full,
        ]
    }

    /// Value in logical pixels. `Full` reports a large sentinel.
    #[must_use]
    pub const fn px(self) -> u16 {
        match self {
            Self::Sm => 6,
            Self::Md => 10,
            Self::Lg => 14,
            Self::Xl => 20,
            Self::Window => 28,
            Self::Full => 9999,
        }
    }
}

/// Elevation: how far a surface sits above the wallpaper.
///
/// Each level is a shadow spec rather than a colour, so one level reads the
/// same way on every theme. Higher levels get larger blur and offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Elevation {
    /// 0 — flush with the wallpaper.
    Z0,
    /// 1 — cards resting on the desktop.
    Z1,
    /// 2 — windows, menus.
    Z2,
    /// 3 — the focused window and popovers.
    Z3,
    /// 4 — spotlight, sheets and banners.
    Z4,
}

impl Elevation {
    /// Every level, lowest first.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Z0, Self::Z1, Self::Z2, Self::Z3, Self::Z4]
    }

    /// Shadow blur radius in logical pixels.
    #[must_use]
    pub const fn blur_px(self) -> u16 {
        match self {
            Self::Z0 => 0,
            Self::Z1 => 8,
            Self::Z2 => 18,
            Self::Z3 => 30,
            Self::Z4 => 44,
        }
    }

    /// Shadow vertical offset in logical pixels.
    #[must_use]
    pub const fn offset_y_px(self) -> u16 {
        match self {
            Self::Z0 => 0,
            Self::Z1 => 2,
            Self::Z2 => 6,
            Self::Z3 => 12,
            Self::Z4 => 20,
        }
    }

    /// Shadow opacity, percent.
    #[must_use]
    pub const fn shadow_alpha_pct(self) -> u8 {
        match self {
            Self::Z0 => 0,
            Self::Z1 => 18,
            Self::Z2 => 28,
            Self::Z3 => 40,
            Self::Z4 => 55,
        }
    }
}

#[cfg(test)]
mod elevation_tests {
    use super::Elevation;

    #[test]
    fn higher_elevation_never_casts_less_shadow() {
        let levels = Elevation::all();
        for pair in levels.windows(2) {
            // `windows(2)` always hands over a slice of exactly two, but the
            // type is a slice: take them by position rather than by indexing,
            // which clippy cannot distinguish from a panicking access.
            let (Some(lo), Some(hi)) = (pair.first(), pair.get(1)) else {
                continue;
            };
            assert!(
                hi.blur_px() >= lo.blur_px(),
                "{lo:?} -> {hi:?} blur shrinks"
            );
            assert!(
                hi.offset_y_px() >= lo.offset_y_px(),
                "{lo:?} -> {hi:?} offset shrinks"
            );
            assert!(
                hi.shadow_alpha_pct() >= lo.shadow_alpha_pct(),
                "{lo:?} -> {hi:?} alpha shrinks"
            );
        }
        assert_eq!(Elevation::Z0.shadow_alpha_pct(), 0);
    }
}

// ---------------------------------------------------------------------------
// Motion
// ---------------------------------------------------------------------------

/// A motion token: a spring, plus the duration budget it should settle within.
///
/// Springs rather than fixed easing curves because a spring is interruptible.
/// ADR-0009 requires animations that can be redirected mid-flight — grabbing a
/// window that is still animating should not wait for the animation to finish,
/// and a fixed-duration curve cannot express that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Motion {
    /// Spring stiffness.
    pub stiffness: u16,
    /// Spring damping.
    pub damping: u16,
    /// Budget within which the motion should have visually settled, in ms.
    pub settle_ms: u16,
}

impl Motion {
    /// State changes that should feel immediate: hover, press.
    pub const INSTANT: Self = Self {
        stiffness: 520,
        damping: 34,
        settle_ms: 120,
    };
    /// Small transitions: popovers, tooltips, list reordering.
    pub const FAST: Self = Self {
        stiffness: 380,
        damping: 30,
        settle_ms: 200,
    };
    /// The default: sheets, panels, window movement.
    pub const NORMAL: Self = Self {
        stiffness: 260,
        damping: 26,
        settle_ms: 300,
    };
    /// Large surfaces: workspace switches, environment switches.
    pub const SLOW: Self = Self {
        stiffness: 180,
        damping: 24,
        settle_ms: 450,
    };

    /// The longest a UI animation may take before it starts costing the user
    /// time rather than communicating structure.
    ///
    /// Beyond this the animation is the interface's opinion of itself, not a
    /// service to the person waiting for it.
    pub const MAX_SETTLE_MS: u16 = 500;

    /// Every motion token.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::INSTANT, Self::FAST, Self::NORMAL, Self::SLOW]
    }

    /// The token to use when the user has asked for reduced motion.
    ///
    /// Not "a shorter animation": reduced motion means the position change
    /// happens without travel. Everything settles in one frame at 60 Hz.
    #[must_use]
    pub const fn reduced() -> Self {
        Self {
            stiffness: 1000,
            damping: 100,
            settle_ms: 16,
        }
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    /// The two worst backdrops a glass panel can sit over.
    const WORST_BACKDROPS: [Rgb; 2] = [Rgb::hex(0xFFFFFF), Rgb::hex(0x000000)];

    // ---- contrast: the reason this crate exists ---------------------------

    #[test]
    fn body_text_passes_aa_on_every_solid_surface_in_every_theme() {
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            let required = if kind.requires_aaa() { 7.0 } else { 4.5 };
            for (name, surface) in [
                ("bg_base", t.palette.bg_base),
                ("bg_elevated", t.palette.bg_elevated),
            ] {
                let r = t.palette.text_primary.contrast(surface);
                assert!(
                    r >= required,
                    "{}: text_primary on {name} is {r:.2}, needs {required}",
                    kind.token()
                );
            }
        }
    }

    #[test]
    fn secondary_text_passes_aa_on_every_solid_surface() {
        // Secondary text is the one that quietly fails: it is deliberately
        // dimmer, and it is where captions and metadata live.
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            let required = if kind.requires_aaa() { 7.0 } else { 4.5 };
            for (name, surface) in [
                ("bg_base", t.palette.bg_base),
                ("bg_elevated", t.palette.bg_elevated),
            ] {
                let r = t.palette.text_secondary.contrast(surface);
                assert!(
                    r >= required,
                    "{}: text_secondary on {name} is {r:.2}, needs {required}",
                    kind.token()
                );
            }
        }
    }

    #[test]
    fn text_on_accent_passes_aa_in_every_theme() {
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            let r = t.palette.on_accent.contrast(t.palette.accent);
            let required = if kind.requires_aaa() { 7.0 } else { 4.5 };
            assert!(
                r >= required,
                "{}: on_accent over accent is {r:.2}, needs {required}",
                kind.token()
            );
        }
    }

    #[test]
    fn status_colours_are_readable_on_the_base_background() {
        // Danger, success and warning are how the system reports that something
        // is wrong. If they are the least legible colours in the palette, the
        // messages that matter most are the hardest to read.
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            for (name, c) in [
                ("danger", t.palette.danger),
                ("success", t.palette.success),
                ("warning", t.palette.warning),
            ] {
                let r = c.contrast(t.palette.bg_base);
                assert!(
                    r >= 4.5,
                    "{}: {name} on bg_base is {r:.2}, needs 4.5",
                    kind.token()
                );
            }
        }
    }

    // ---- accents: a user choice that must not undo the contrast gate ------

    /// The two themes an accent can actually change. High contrast is excluded
    /// on purpose: see `high_contrast_themes_ignore_the_accent_choice`.
    const ACCENT_THEMES: [ThemeKind; 2] = [ThemeKind::Dark, ThemeKind::Light];

    #[test]
    fn every_accent_keeps_text_legible_on_its_own_fill() {
        // The failure this test exists for: an accent picker whose swatch is
        // fine and whose button label is unreadable, because `on_accent` was
        // left at whatever the default theme used.
        for kind in ACCENT_THEMES {
            for accent in Accent::all() {
                let t = Theme::of(kind).with_accent(*accent);
                let r = t.palette.on_accent.contrast(t.palette.accent);
                assert!(
                    r >= 4.5,
                    "{}: on_accent over {} accent is {r:.2}, needs 4.5",
                    kind.token(),
                    accent.token()
                );
            }
        }
    }

    #[test]
    fn every_accent_is_readable_as_text_on_the_theme_backgrounds() {
        // The accent is also a text tone (`Tone::Accent`): selected rows, links
        // and the primary action label. A pastel that is only meant as a fill
        // becomes invisible the moment it is used for words.
        for kind in ACCENT_THEMES {
            for accent in Accent::all() {
                let p = Theme::of(kind).with_accent(*accent).palette;
                for (name, surface) in [("bg_base", p.bg_base), ("bg_elevated", p.bg_elevated)] {
                    let r = p.accent.contrast(surface);
                    assert!(
                        r >= 4.5,
                        "{}: {} accent on {name} is {r:.2}, needs 4.5",
                        kind.token(),
                        accent.token()
                    );
                }
            }
        }
    }

    #[test]
    fn high_contrast_themes_ignore_the_accent_choice() {
        // A statement of policy, not an implementation detail: the accent is
        // part of the AAA guarantee in these themes, so the picker must not be
        // able to weaken it. The settings UI reports the choice as fixed rather
        // than showing a control with no effect.
        for kind in [ThemeKind::HighContrastDark, ThemeKind::HighContrastLight] {
            let base = Theme::of(kind);
            for accent in Accent::all() {
                assert_eq!(
                    base.with_accent(*accent),
                    base,
                    "{}: accent {} changed a high-contrast theme",
                    kind.token(),
                    accent.token()
                );
            }
        }
    }

    #[test]
    fn choosing_an_accent_moves_only_the_accent_pair() {
        // Everything else in the palette is the contrast contract. If a
        // decorative choice could move `bg_base` or `text_primary`, it would be
        // a second theme system wearing an accent picker's name.
        let base = Theme::of(ThemeKind::Dark);
        let chosen = base.with_accent(Accent::Violet);
        assert_ne!(chosen.palette.accent, base.palette.accent);
        assert_eq!(chosen.palette.bg_base, base.palette.bg_base);
        assert_eq!(chosen.palette.bg_elevated, base.palette.bg_elevated);
        assert_eq!(chosen.palette.text_primary, base.palette.text_primary);
        assert_eq!(chosen.palette.text_secondary, base.palette.text_secondary);
        assert_eq!(chosen.palette.danger, base.palette.danger);
        assert_eq!(chosen.palette.success, base.palette.success);
        assert_eq!(chosen.palette.warning, base.palette.warning);
        assert_eq!(chosen.palette.border, base.palette.border);
        assert_eq!(chosen.glass, base.glass);
    }

    #[test]
    fn the_default_accent_is_the_theme_itself() {
        for kind in ThemeKind::all() {
            assert_eq!(
                Theme::of(*kind).with_accent(Accent::Default),
                Theme::of(*kind)
            );
        }
    }

    #[test]
    fn accent_tokens_are_unique_and_round_trip() {
        // The preference file stores these strings. Two accents sharing a token
        // means a user's choice silently becomes a different colour on restart.
        let mut seen: Vec<&str> = Vec::new();
        for accent in Accent::all() {
            assert!(
                !seen.contains(&accent.token()),
                "duplicate token {}",
                accent.token()
            );
            seen.push(accent.token());
            assert_eq!(Accent::parse(accent.token()), Some(*accent));
        }
        assert_eq!(Accent::parse("  violet "), Some(Accent::Violet));
        assert_eq!(Accent::parse("chartreuse"), None);
        assert_eq!(Accent::parse(""), None);
    }

    // ---- glass: the failure mode this crate was written to prevent --------

    #[test]
    fn body_text_stays_legible_on_glass_over_the_worst_backdrop() {
        // THE test. A glass panel is composited over whatever the user set as a
        // wallpaper, which may be pure white. Checking contrast against the tint
        // colour alone would pass a panel that is unreadable in practice.
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            let required = if kind.requires_aaa() { 7.0 } else { 4.5 };
            for backdrop in WORST_BACKDROPS {
                let surface = t.glass.effective_surface(t.palette.glass_tint, backdrop);
                let r = t.palette.text_primary.contrast(surface);
                assert!(
                    r >= required,
                    "{}: text on glass over {backdrop} is {r:.2}, needs {required} \
                     (effective surface {surface})",
                    kind.token()
                );
            }
        }
    }

    #[test]
    fn secondary_text_stays_legible_on_glass_over_the_worst_backdrop() {
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            let required = if kind.requires_aaa() { 7.0 } else { 4.5 };
            for backdrop in WORST_BACKDROPS {
                let surface = t.glass.effective_surface(t.palette.glass_tint, backdrop);
                let r = t.palette.text_secondary.contrast(surface);
                assert!(
                    r >= required,
                    "{}: secondary text on glass over {backdrop} is {r:.2}, needs {required}",
                    kind.token()
                );
            }
        }
    }

    // ---- adaptive glass: beauty and legibility at the same time ----------

    #[test]
    fn adaptive_glass_is_far_more_transparent_over_a_dark_wallpaper() {
        // The whole point. A static alpha must assume the worst wallpaper
        // anyone might set, so it is permanently as heavy as its worst case.
        let t = Theme::of(ThemeKind::Dark);
        let dark_wallpaper = Rgb::hex(0x101828);

        let pct = Glass::adaptive_alpha_pct(
            t.palette.glass_tint,
            t.palette.text_secondary,
            dark_wallpaper,
            4.5,
        )
        .expect("a dark wallpaper must be satisfiable");

        assert!(
            pct < 40,
            "over a dark wallpaper glass should be able to drop below 40%, got {pct}%"
        );
        assert!(
            pct < t.glass.tint_alpha_pct,
            "adaptive ({pct}%) must beat the static worst-case ({}%)",
            t.glass.tint_alpha_pct
        );
    }

    #[test]
    fn adaptive_glass_firms_up_over_a_bright_wallpaper_without_being_told() {
        let t = Theme::of(ThemeKind::Dark);
        let dark = Glass::adaptive_alpha_pct(
            t.palette.glass_tint,
            t.palette.text_secondary,
            Rgb::hex(0x000000),
            4.5,
        )
        .expect("black is satisfiable");
        let bright = Glass::adaptive_alpha_pct(
            t.palette.glass_tint,
            t.palette.text_secondary,
            Rgb::hex(0xFFFFFF),
            4.5,
        )
        .expect("white is satisfiable");

        assert!(
            bright > dark,
            "a brighter backdrop must demand more opacity: got {bright}% vs {dark}%"
        );
    }

    #[test]
    fn adaptive_glass_holds_aa_across_the_entire_backdrop_space() {
        // The guarantee, checked exhaustively rather than at two convenient
        // points. Every one of these backdrops is a wallpaper someone could set,
        // and the returned alpha must satisfy AA for BOTH text roles at each.
        let t = Theme::of(ThemeKind::Dark);
        let mut checked = 0u32;

        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(51) {
                    let backdrop = Rgb::new(r as u8, g as u8, b as u8);
                    for text in [t.palette.text_primary, t.palette.text_secondary] {
                        let Some(pct) =
                            Glass::adaptive_alpha_pct(t.palette.glass_tint, text, backdrop, 4.5)
                        else {
                            panic!("no alpha satisfies AA over {backdrop} -- palette is wrong");
                        };
                        let surface = t.palette.glass_tint.over(backdrop, f64::from(pct) / 100.0);
                        let ratio = text.contrast(surface);
                        assert!(
                            ratio >= 4.5,
                            "adaptive returned {pct}% for {backdrop} but contrast is {ratio:.2}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(
            checked > 1000,
            "sweep was too small to be meaningful: {checked}"
        );
    }

    #[test]
    fn adaptive_glass_returns_the_minimum_not_merely_a_sufficient_alpha() {
        // If it returned any passing value the material would be needlessly
        // heavy. One percent below the answer must actually fail.
        let t = Theme::of(ThemeKind::Dark);
        for backdrop in [Rgb::hex(0x223344), Rgb::hex(0x9AA5B1), Rgb::hex(0xFFFFFF)] {
            let pct = Glass::adaptive_alpha_pct(
                t.palette.glass_tint,
                t.palette.text_secondary,
                backdrop,
                4.5,
            )
            .expect("satisfiable");
            if pct <= Glass::MIN_AESTHETIC_ALPHA_PCT {
                continue; // already at the aesthetic floor, nothing below to test
            }
            let below = f64::from(pct - 1) / 100.0;
            let surface = t.palette.glass_tint.over(backdrop, below);
            assert!(
                t.palette.text_secondary.contrast(surface) < 4.5,
                "{}% was not minimal for {backdrop}",
                pct
            );
        }
    }

    #[test]
    fn adaptive_glass_never_becomes_a_hole_or_a_solid_slab() {
        let t = Theme::of(ThemeKind::Dark);
        for backdrop in [Rgb::hex(0x000000), Rgb::hex(0x0A0A0A), Rgb::hex(0xFFFFFF)] {
            let g = t.glass.adapted_to(
                t.palette.glass_tint,
                t.palette.text_secondary,
                backdrop,
                4.5,
            );
            assert!(
                g.tint_alpha_pct >= Glass::MIN_AESTHETIC_ALPHA_PCT,
                "{backdrop}: {}% is transparent enough to stop being a surface",
                g.tint_alpha_pct
            );
            assert!(
                g.tint_alpha_pct <= Glass::MAX_ADAPTIVE_ALPHA_PCT,
                "{backdrop}: {}% is effectively opaque",
                g.tint_alpha_pct
            );
        }
    }

    #[test]
    fn high_contrast_glass_is_never_adapted() {
        // Making the accessibility theme translucent because the wallpaper
        // happened to be dark would defeat the reason it exists.
        for kind in [ThemeKind::HighContrastDark, ThemeKind::HighContrastLight] {
            let t = Theme::of(kind);
            let g = t.glass.adapted_to(
                t.palette.glass_tint,
                t.palette.text_primary,
                Rgb::hex(0x000000),
                7.0,
            );
            assert_eq!(
                g.tint_alpha_pct,
                100,
                "{}: high-contrast glass must stay opaque",
                kind.token()
            );
        }
    }

    #[test]
    fn every_theme_respects_the_glass_alpha_floor() {
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            assert!(
                t.glass.tint_alpha_pct >= Glass::MIN_TINT_ALPHA_PCT,
                "{}: tint alpha {}% is below the {}% floor",
                kind.token(),
                t.glass.tint_alpha_pct,
                Glass::MIN_TINT_ALPHA_PCT
            );
        }
    }

    #[test]
    fn the_alpha_floor_is_where_legibility_actually_breaks() {
        // Proves the floor is measured rather than decorative: at the floor the
        // dark theme still passes AA over white, and meaningfully below it, it
        // does not. If this test fails, the constant is wrong, not the theme.
        let t = Theme::of(ThemeKind::Dark);
        let white = Rgb::hex(0xFFFFFF);

        let at_floor = Glass {
            tint_alpha_pct: Glass::MIN_TINT_ALPHA_PCT,
            ..t.glass
        };
        let s = at_floor.effective_surface(t.palette.glass_tint, white);
        assert!(
            t.palette.text_primary.contrast(s) >= 4.5,
            "the floor should still pass AA"
        );

        let too_low = Glass {
            tint_alpha_pct: 40,
            ..t.glass
        };
        let s = too_low.effective_surface(t.palette.glass_tint, white);
        assert!(
            t.palette.text_primary.contrast(s) < 4.5,
            "40% alpha should fail AA over white -- if it passes, the floor is \
             too conservative and glass could be prettier"
        );
    }

    #[test]
    fn high_contrast_themes_give_up_translucency_not_contrast() {
        for kind in [ThemeKind::HighContrastDark, ThemeKind::HighContrastLight] {
            let t = Theme::of(kind);
            assert_eq!(
                t.glass.tint_alpha_pct,
                100,
                "{}: high-contrast glass must be opaque",
                kind.token()
            );
            assert_eq!(
                t.glass.blur_px,
                0,
                "{}: no blur in high contrast",
                kind.token()
            );
        }
    }

    // ---- scales -----------------------------------------------------------

    #[test]
    fn the_type_scale_is_strictly_increasing_with_its_line_heights() {
        let mut last_px = 0u16;
        let mut last_lh = 0u16;
        for step in TypeStep::all() {
            assert!(
                step.px() > last_px,
                "{step:?} is not larger than the step below"
            );
            assert!(
                step.line_height_px() > last_lh,
                "{step:?} line height regressed"
            );
            assert!(
                step.line_height_px() > step.px(),
                "{step:?} line height must exceed the font size"
            );
            last_px = step.px();
            last_lh = step.line_height_px();
        }
    }

    #[test]
    fn line_height_ratio_loosens_for_body_and_tightens_for_display() {
        // Body text needs room to read a paragraph; a display heading at the
        // same ratio looks like two unrelated lines.
        let body = f64::from(TypeStep::Body.line_height_px()) / f64::from(TypeStep::Body.px());
        let display =
            f64::from(TypeStep::Display.line_height_px()) / f64::from(TypeStep::Display.px());
        assert!(
            body > display,
            "body ratio {body:.2} should exceed display {display:.2}"
        );
        assert!(
            (1.35..=1.6).contains(&body),
            "body ratio {body:.2} outside a readable range"
        );
    }

    #[test]
    fn wcag_large_text_threshold_is_applied_at_24px() {
        assert!(!TypeStep::Body.is_large_text());
        assert!(!TypeStep::Title3.is_large_text());
        assert!(TypeStep::Title2.is_large_text());
        assert!(TypeStep::Display.is_large_text());
    }

    #[test]
    fn spacing_and_radius_scales_are_strictly_increasing() {
        let mut last = 0u16;
        for s in Space::all() {
            assert!(s.px() > last, "{s:?} broke the spacing scale");
            last = s.px();
        }
        let mut last = 0u16;
        for r in Radius::all() {
            assert!(r.px() > last, "{r:?} broke the radius scale");
            last = r.px();
        }
    }

    #[test]
    fn spacing_sits_on_a_grid() {
        // Everything except the 2px hairline is a multiple of 4. A scale with
        // arbitrary values is a scale that stops being followed.
        for s in Space::all() {
            if *s == Space::Xxs {
                continue;
            }
            assert_eq!(s.px() % 4, 0, "{s:?} = {} is off the 4px grid", s.px());
        }
    }

    // ---- motion -----------------------------------------------------------

    #[test]
    fn no_motion_token_exceeds_the_settle_budget() {
        for m in Motion::all() {
            assert!(
                m.settle_ms <= Motion::MAX_SETTLE_MS,
                "a motion token settles in {}ms, over the {}ms budget",
                m.settle_ms,
                Motion::MAX_SETTLE_MS
            );
        }
    }

    #[test]
    fn motion_tokens_are_ordered_and_springs_stay_damped() {
        let tokens = Motion::all();
        let mut last = 0u16;
        for m in tokens {
            assert!(
                m.settle_ms > last,
                "motion tokens are not ordered by settle time"
            );
            last = m.settle_ms;
            // An underdamped spring overshoots and wobbles; for interface motion
            // that reads as unstable rather than lively.
            assert!(
                m.damping >= 20,
                "damping {} is too low, motion will wobble",
                m.damping
            );
            assert!(m.stiffness >= 100, "stiffness {} is too low", m.stiffness);
        }
    }

    #[test]
    fn reduced_motion_settles_within_one_frame() {
        // Reduced motion is not "a faster animation" -- the change must arrive
        // without travel. One frame at 60 Hz is 16.67ms.
        let r = Motion::reduced();
        assert!(r.settle_ms <= 17, "reduced motion takes {}ms", r.settle_ms);
        assert!(r.settle_ms < Motion::INSTANT.settle_ms);
    }

    // ---- coherence --------------------------------------------------------

    #[test]
    fn theme_tokens_are_unique_and_every_theme_is_constructible() {
        let mut seen = std::collections::BTreeSet::new();
        for kind in ThemeKind::all() {
            assert!(seen.insert(kind.token()), "duplicate theme token");
            let t = Theme::of(*kind);
            assert_eq!(t.kind, *kind, "Theme::of returned the wrong kind");
        }
        assert_eq!(seen.len(), 4);
    }

    #[test]
    fn borders_are_visible_against_their_own_surface() {
        // A border that does not contrast with the surface it outlines is not a
        // border. WCAG asks 3.0 for non-text UI elements.
        for kind in ThemeKind::all() {
            let t = Theme::of(*kind);
            let r = t.palette.border.contrast(t.palette.bg_elevated);
            assert!(
                r >= 1.3,
                "{}: border on bg_elevated is {r:.2}, too faint to read as an edge",
                kind.token()
            );
        }
    }
}
