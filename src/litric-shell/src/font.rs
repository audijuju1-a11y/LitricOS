//! LitricOS text drawing: offline OpenType faces with a safe vector fallback.
//!
//! # Why a typeface is in here at all
//!
//! The shell has to put words on a screen at boot, before any font library,
//! font file or font server exists, and this project ships no third-party
//! crates. The two usual answers are both bad: a bitmap font is blocky at every
//! size except the one it was drawn for, and embedding somebody else's TTF
//! means shipping their file and writing a TrueType interpreter to read it.
//!
//! So the letters are described the way a signwriter describes them — as the
//! path a pen takes — and rasterised with the same anti-aliased line drawer the
//! rest of the interface uses. That makes the type resolution-independent,
//! smooth at any size, cheap to render, and genuinely ours: every curve below
//! was placed here, not copied from a font.
//!
//! It is a monoline geometric sans, which is a real design decision rather than
//! a limitation: circles and straight lines, uniform stroke weight, single-
//! storey `a` and `g`. It sits beside the interface's geometry instead of
//! arguing with it.
//!
//! # The grid
//!
//! ```text
//!   y = 10   cap height, ascender
//!   y =  7   x-height
//!   y =  0   baseline
//!   y = -3   descender
//! ```
//!
//! `y` increases upward, as in every other font format and in none of the
//! raster APIs, so the renderer flips it once, at the point where font space
//! becomes pixels.

use std::sync::OnceLock;

use crate::canvas::{arc_points, Canvas};

/// Typeface family selected for a run.
pub use litric_text::Face as FontFace;

static OPEN_TYPE: OnceLock<Option<litric_text::TextSystem>> = OnceLock::new();

/// Initialize the offline font stack before the first UI frame.
///
/// The image installs Noto Sans Thai and JetBrains Mono. If an image is missing
/// one of them, drawing keeps the local vector fallback alive and reports the
/// missing stack through the return value instead of failing desktop startup.
#[must_use]
pub fn initialize() -> bool {
    OPEN_TYPE
        .get_or_init(|| litric_text::TextSystem::load_system().ok())
        .is_some()
}

/// Initialize the text stack from a directory of installed faces.
///
/// For the review tools on a host that is not the image. `frame_dump` used to
/// refuse to run at all without the installed faces — correctly, because a frame
/// set in a different typeface has different word widths and is not evidence
/// about the machine — and the cost was that on a development host with no
/// package manager in the loop, *no* frame could be produced, so UI work could
/// only be reviewed by reading it. This is the third option: point the tool at a
/// local copy of the same six files and get frames that mean something.
///
/// The shipping shell never calls this, and an application still cannot supply a
/// font: the directory is named on the command line by the person running a
/// development tool.
#[must_use]
pub fn initialize_from_dir<P: AsRef<std::path::Path>>(dir: P) -> bool {
    OPEN_TYPE
        .get_or_init(|| litric_text::TextSystem::load_dir(dir).ok())
        .is_some()
}

fn open_type() -> Option<&'static litric_text::TextSystem> {
    OPEN_TYPE
        .get_or_init(|| litric_text::TextSystem::load_system().ok())
        .as_ref()
}

/// Cap height in pixels for a font size, from whichever face is live.
///
/// The two text paths must agree on what [`Style::size`] means or one `Style`
/// draws two different sizes depending on which path runs, and every layout
/// audit performed on one machine describes a UI the other machine never shows.
/// The tokens are ems (`TypeStep::px()` is *"size in logical pixels"*, and the
/// WCAG large-text test they carry is defined on ems), the installed OpenType
/// faces take ems, and the vector fallback reads this to draw a cap that matches
/// them. The centring arithmetic elsewhere in the shell — `cy + size * 0.36`,
/// which is half of a 0.72-em cap — has assumed ems all along.
#[must_use]
pub fn cap_px(em: f32) -> f32 {
    em * cap_em_ratio()
}

/// Cap height as a fraction of the em, for whichever face is live.
///
/// Measured from the loaded face when one is installed, so a change of typeface
/// moves the fallback with it.
#[must_use]
pub fn cap_em_ratio() -> f32 {
    open_type().map_or(litric_text::DOCUMENTED_CAP_EM_RATIO, |system| {
        system.cap_em_ratio()
    })
}

/// Whether the live text stack has a face that can draw Thai script.
///
/// When it does not, every Thai character is a missing glyph, and the interface
/// is safer in a language it can actually print — see [`crate::i18n`].
#[must_use]
pub fn draws_thai() -> bool {
    open_type().is_some_and(litric_text::TextSystem::draws_thai)
}

/// Which faces loaded and which were substituted, for the diagnostics page.
#[must_use]
pub fn face_report() -> litric_text::Faces {
    open_type().map_or_else(litric_text::Faces::default, |system| system.faces())
}

/// One pen movement.
#[derive(Debug, Clone, Copy)]
pub enum Stroke {
    /// A connected run of straight segments.
    Poly(&'static [(f32, f32)]),
    /// An elliptical arc, in degrees, counter-clockwise from the +x axis.
    ///
    /// Sweeping from a larger angle to a smaller one takes the other way round
    /// the ellipse. That is not a detail: it is the difference between the hook
    /// of an `f` and a loop.
    Arc {
        /// Centre x.
        cx: f32,
        /// Centre y.
        cy: f32,
        /// Radius in x.
        rx: f32,
        /// Radius in y.
        ry: f32,
        /// Start angle, degrees.
        a0: f32,
        /// End angle, degrees.
        a1: f32,
    },
    /// A filled disc: the tittle on an `i`, a full stop.
    Dot {
        /// Centre x.
        x: f32,
        /// Centre y.
        y: f32,
        /// Radius, in font units.
        r: f32,
    },
}

/// One character.
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    /// The character it draws.
    pub ch: char,
    /// How far the pen moves after drawing it, in font units.
    pub adv: f32,
    /// The strokes that make it.
    pub strokes: &'static [Stroke],
}

/// Cap height in font units. A size in pixels is a cap height in pixels.
pub const CAP: f32 = 10.0;
/// x-height.
pub const X_HEIGHT: f32 = 7.0;
/// Distance from one baseline to the next, in font units.
pub const LINE: f32 = 15.0;
/// Stroke weight as a fraction of cap height, for regular text.
///
/// 0.105 read as pencil sketching: at UI sizes the hairline thinned faster
/// than the eye forgives. 0.135 is the band a product UI font occupies
/// (SF Pro and Inter land near it at their regular weight) while monoline
/// geometry stays legible.
pub const WEIGHT_REGULAR: f32 = 0.135;
/// Stroke weight for headings and emphasis.
pub const WEIGHT_BOLD: f32 = 0.195;

/// How a run of text should be set.
///
/// Bundled rather than passed as six arguments: a call site with `20.0, 0.105,
/// 0.0, colour` in it is one transposition away from silently drawing hairline
/// text, and the compiler cannot tell the difference between two `f32`s.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Font size in pixels: the em, the same unit [`litric_design::TypeStep::px`]
    /// uses. Not the cap height — see [`cap_px`].
    pub size: f32,
    /// Stroke weight as a fraction of the size.
    pub weight: f32,
    /// Extra pixels between characters.
    pub tracking: f32,
    /// Ink colour.
    pub color: u32,
    /// Typeface family.
    pub face: FontFace,
}

impl Style {
    /// Regular weight, no tracking.
    #[must_use]
    pub const fn new(size: f32, color: u32) -> Self {
        Self {
            size,
            weight: WEIGHT_REGULAR,
            tracking: 0.0,
            color,
            face: FontFace::Sans,
        }
    }

    /// The same, at the bold weight.
    #[must_use]
    pub const fn bold(size: f32, color: u32) -> Self {
        Self {
            size,
            weight: WEIGHT_BOLD,
            tracking: 0.0,
            color,
            face: FontFace::Sans,
        }
    }

    /// With letter-spacing.
    #[must_use]
    pub const fn tracked(mut self, tracking: f32) -> Self {
        self.tracking = tracking;
        self
    }

    /// In a different colour.
    #[must_use]
    pub const fn colored(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    /// Use the installed monospace face for code and terminal output.
    #[must_use]
    pub const fn monospace(mut self) -> Self {
        self.face = FontFace::Mono;
        self
    }
}

use Stroke::{Arc, Dot, Poly};

// ---------------------------------------------------------------------------
// Uppercase. Cap height 10, most letters 7 wide.
// ---------------------------------------------------------------------------

const G_A: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (3.5, 10.0), (7.0, 0.0)]),
    Poly(&[(1.1, 3.1), (5.9, 3.1)]),
];
const G_B: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 10.0)]),
    Poly(&[(0.0, 10.0), (3.5, 10.0)]),
    Arc {
        cx: 3.5,
        cy: 7.5,
        rx: 2.7,
        ry: 2.5,
        a0: 90.0,
        a1: -90.0,
    },
    Poly(&[(3.5, 5.0), (0.0, 5.0)]),
    Poly(&[(0.0, 5.0), (3.7, 5.0)]),
    Arc {
        cx: 3.7,
        cy: 2.5,
        rx: 3.0,
        ry: 2.5,
        a0: 90.0,
        a1: -90.0,
    },
    Poly(&[(3.7, 0.0), (0.0, 0.0)]),
];
const G_C: &[Stroke] = &[Arc {
    cx: 3.6,
    cy: 5.0,
    rx: 3.6,
    ry: 5.0,
    a0: 55.0,
    a1: 305.0,
}];
const G_D: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 10.0)]),
    Poly(&[(0.0, 10.0), (2.6, 10.0)]),
    Arc {
        cx: 2.6,
        cy: 5.0,
        rx: 4.2,
        ry: 5.0,
        a0: 90.0,
        a1: -90.0,
    },
    Poly(&[(2.6, 0.0), (0.0, 0.0)]),
];
const G_E: &[Stroke] = &[
    Poly(&[(6.5, 10.0), (0.0, 10.0), (0.0, 0.0), (6.5, 0.0)]),
    Poly(&[(0.0, 5.0), (5.3, 5.0)]),
];
const G_F: &[Stroke] = &[
    Poly(&[(6.5, 10.0), (0.0, 10.0), (0.0, 0.0)]),
    Poly(&[(0.0, 5.0), (5.1, 5.0)]),
];
const G_G: &[Stroke] = &[
    Arc {
        cx: 3.6,
        cy: 5.0,
        rx: 3.6,
        ry: 5.0,
        a0: 55.0,
        a1: 350.0,
    },
    Poly(&[(4.1, 4.13), (7.14, 4.13)]),
];
const G_H: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 10.0)]),
    Poly(&[(7.0, 0.0), (7.0, 10.0)]),
    Poly(&[(0.0, 5.0), (7.0, 5.0)]),
];
const G_I: &[Stroke] = &[Poly(&[(2.1, 0.0), (2.1, 10.0)])];
const G_J: &[Stroke] = &[
    Poly(&[(5.6, 10.0), (5.6, 2.7)]),
    Arc {
        cx: 3.0,
        cy: 2.7,
        rx: 2.6,
        ry: 2.7,
        a0: 0.0,
        a1: -180.0,
    },
];
const G_K: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 10.0)]),
    Poly(&[(6.4, 10.0), (0.5, 4.3)]),
    Poly(&[(2.3, 6.0), (6.8, 0.0)]),
];
const G_L: &[Stroke] = &[Poly(&[(0.0, 10.0), (0.0, 0.0), (6.3, 0.0)])];
const G_M: &[Stroke] = &[Poly(&[
    (0.0, 0.0),
    (0.0, 10.0),
    (3.7, 3.2),
    (7.4, 10.0),
    (7.4, 0.0),
])];
const G_N: &[Stroke] = &[Poly(&[(0.0, 0.0), (0.0, 10.0), (7.0, 0.0), (7.0, 10.0)])];
const G_O: &[Stroke] = &[Arc {
    cx: 3.6,
    cy: 5.0,
    rx: 3.6,
    ry: 5.0,
    a0: 0.0,
    a1: 360.0,
}];
const G_P: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 10.0)]),
    Poly(&[(0.0, 10.0), (3.6, 10.0)]),
    Arc {
        cx: 3.6,
        cy: 7.4,
        rx: 3.0,
        ry: 2.6,
        a0: 90.0,
        a1: -90.0,
    },
    Poly(&[(3.6, 4.8), (0.0, 4.8)]),
];
const G_Q: &[Stroke] = &[
    Arc {
        cx: 3.6,
        cy: 5.0,
        rx: 3.6,
        ry: 5.0,
        a0: 0.0,
        a1: 360.0,
    },
    Poly(&[(4.6, 2.3), (7.4, -0.6)]),
];
const G_R: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 10.0)]),
    Poly(&[(0.0, 10.0), (3.6, 10.0)]),
    Arc {
        cx: 3.6,
        cy: 7.4,
        rx: 3.0,
        ry: 2.6,
        a0: 90.0,
        a1: -90.0,
    },
    Poly(&[(3.6, 4.8), (0.0, 4.8)]),
    Poly(&[(3.3, 4.8), (7.0, 0.0)]),
];
const G_S: &[Stroke] = &[Poly(&[
    (6.5, 8.4),
    (5.4, 9.7),
    (2.4, 10.0),
    (0.7, 8.8),
    (0.6, 6.9),
    (2.0, 5.7),
    (4.8, 4.8),
    (6.4, 3.6),
    (6.5, 1.6),
    (4.9, 0.2),
    (1.9, 0.0),
    (0.4, 1.3),
])];
const G_T: &[Stroke] = &[
    Poly(&[(0.0, 10.0), (7.0, 10.0)]),
    Poly(&[(3.5, 10.0), (3.5, 0.0)]),
];
const G_U: &[Stroke] = &[
    Poly(&[(0.0, 10.0), (0.0, 3.2)]),
    Arc {
        cx: 3.5,
        cy: 3.2,
        rx: 3.5,
        ry: 3.2,
        a0: 180.0,
        a1: 360.0,
    },
    Poly(&[(7.0, 3.2), (7.0, 10.0)]),
];
const G_V: &[Stroke] = &[Poly(&[(0.0, 10.0), (3.5, 0.0), (7.0, 10.0)])];
const G_W: &[Stroke] = &[Poly(&[
    (0.0, 10.0),
    (2.0, 0.0),
    (4.6, 7.0),
    (7.2, 0.0),
    (9.2, 10.0),
])];
const G_X: &[Stroke] = &[
    Poly(&[(0.0, 10.0), (7.0, 0.0)]),
    Poly(&[(0.0, 0.0), (7.0, 10.0)]),
];
const G_Y: &[Stroke] = &[
    Poly(&[(0.0, 10.0), (3.5, 5.2), (7.0, 10.0)]),
    Poly(&[(3.5, 5.2), (3.5, 0.0)]),
];
const G_Z: &[Stroke] = &[Poly(&[(0.0, 10.0), (7.0, 10.0), (0.0, 0.0), (7.0, 0.0)])];

// ---------------------------------------------------------------------------
// Lowercase. x-height 7, ascenders to 10, descenders to -3.
// ---------------------------------------------------------------------------

const G_LOW_A: &[Stroke] = &[
    Arc {
        cx: 3.3,
        cy: 3.5,
        rx: 3.3,
        ry: 3.5,
        a0: 0.0,
        a1: 360.0,
    },
    Poly(&[(6.6, 7.0), (6.6, 0.0)]),
];
const G_LOW_B: &[Stroke] = &[
    Poly(&[(0.0, 10.0), (0.0, 0.0)]),
    Arc {
        cx: 3.4,
        cy: 3.5,
        rx: 3.4,
        ry: 3.5,
        a0: 0.0,
        a1: 360.0,
    },
];
const G_LOW_C: &[Stroke] = &[Arc {
    cx: 3.4,
    cy: 3.5,
    rx: 3.4,
    ry: 3.5,
    a0: 55.0,
    a1: 305.0,
}];
const G_LOW_D: &[Stroke] = &[
    Arc {
        cx: 3.4,
        cy: 3.5,
        rx: 3.4,
        ry: 3.5,
        a0: 0.0,
        a1: 360.0,
    },
    Poly(&[(6.8, 10.0), (6.8, 0.0)]),
];
const G_LOW_E: &[Stroke] = &[
    Arc {
        cx: 3.4,
        cy: 3.5,
        rx: 3.4,
        ry: 3.5,
        a0: 0.0,
        a1: 325.0,
    },
    Poly(&[(0.0, 3.5), (6.8, 3.5)]),
];
const G_LOW_F: &[Stroke] = &[
    Poly(&[(2.6, 0.0), (2.6, 8.3)]),
    Arc {
        cx: 4.4,
        cy: 8.3,
        rx: 1.8,
        ry: 1.8,
        a0: 180.0,
        a1: 90.0,
    },
    Poly(&[(0.5, 7.0), (5.0, 7.0)]),
];
const G_LOW_G: &[Stroke] = &[
    Arc {
        cx: 3.4,
        cy: 3.5,
        rx: 3.4,
        ry: 3.5,
        a0: 0.0,
        a1: 360.0,
    },
    Poly(&[(6.8, 7.0), (6.8, -1.0)]),
    Arc {
        cx: 4.0,
        cy: -1.0,
        rx: 2.8,
        ry: 1.9,
        a0: 0.0,
        a1: -165.0,
    },
];
const G_LOW_H: &[Stroke] = &[
    Poly(&[(0.0, 10.0), (0.0, 0.0)]),
    Arc {
        cx: 3.4,
        cy: 3.6,
        rx: 3.4,
        ry: 3.4,
        a0: 180.0,
        a1: 0.0,
    },
    Poly(&[(6.8, 3.6), (6.8, 0.0)]),
];
const G_LOW_I: &[Stroke] = &[
    Poly(&[(1.6, 0.0), (1.6, 7.0)]),
    Dot {
        x: 1.6,
        y: 9.1,
        r: 0.85,
    },
];
const G_LOW_J: &[Stroke] = &[
    Poly(&[(2.4, 7.0), (2.4, -1.1)]),
    Arc {
        cx: 1.0,
        cy: -1.1,
        rx: 1.4,
        ry: 1.7,
        a0: 0.0,
        a1: -180.0,
    },
    Dot {
        x: 2.4,
        y: 9.1,
        r: 0.85,
    },
];
const G_LOW_K: &[Stroke] = &[
    Poly(&[(0.0, 10.0), (0.0, 0.0)]),
    Poly(&[(5.9, 7.0), (0.5, 2.6)]),
    Poly(&[(2.2, 4.0), (6.2, 0.0)]),
];
const G_LOW_L: &[Stroke] = &[Poly(&[(1.6, 10.0), (1.6, 0.0)])];
const G_LOW_M: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 7.0)]),
    Arc {
        cx: 2.7,
        cy: 3.6,
        rx: 2.7,
        ry: 3.4,
        a0: 180.0,
        a1: 0.0,
    },
    Poly(&[(5.4, 3.6), (5.4, 0.0)]),
    Arc {
        cx: 8.1,
        cy: 3.6,
        rx: 2.7,
        ry: 3.4,
        a0: 180.0,
        a1: 0.0,
    },
    Poly(&[(10.8, 3.6), (10.8, 0.0)]),
];
const G_LOW_N: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 7.0)]),
    Arc {
        cx: 3.4,
        cy: 3.6,
        rx: 3.4,
        ry: 3.4,
        a0: 180.0,
        a1: 0.0,
    },
    Poly(&[(6.8, 3.6), (6.8, 0.0)]),
];
const G_LOW_O: &[Stroke] = &[Arc {
    cx: 3.4,
    cy: 3.5,
    rx: 3.4,
    ry: 3.5,
    a0: 0.0,
    a1: 360.0,
}];
const G_LOW_P: &[Stroke] = &[
    Poly(&[(0.0, 7.0), (0.0, -3.0)]),
    Arc {
        cx: 3.4,
        cy: 3.5,
        rx: 3.4,
        ry: 3.5,
        a0: 0.0,
        a1: 360.0,
    },
];
const G_LOW_Q: &[Stroke] = &[
    Arc {
        cx: 3.4,
        cy: 3.5,
        rx: 3.4,
        ry: 3.5,
        a0: 0.0,
        a1: 360.0,
    },
    Poly(&[(6.8, 7.0), (6.8, -3.0)]),
];
const G_LOW_R: &[Stroke] = &[
    Poly(&[(0.0, 0.0), (0.0, 7.0)]),
    Arc {
        cx: 3.2,
        cy: 3.9,
        rx: 3.2,
        ry: 3.1,
        a0: 180.0,
        a1: 55.0,
    },
];
const G_LOW_S: &[Stroke] = &[Poly(&[
    (5.7, 5.6),
    (4.7, 6.9),
    (1.9, 7.0),
    (0.5, 5.9),
    (0.6, 4.5),
    (1.9, 3.8),
    (4.3, 3.2),
    (5.6, 2.3),
    (5.6, 1.0),
    (4.2, 0.0),
    (1.6, 0.0),
    (0.3, 0.9),
])];
const G_LOW_T: &[Stroke] = &[
    Poly(&[(2.5, 10.0), (2.5, 1.7)]),
    Arc {
        cx: 4.1,
        cy: 1.7,
        rx: 1.6,
        ry: 1.7,
        a0: 180.0,
        a1: 270.0,
    },
    Poly(&[(0.5, 7.0), (4.8, 7.0)]),
];
const G_LOW_U: &[Stroke] = &[
    Poly(&[(0.0, 7.0), (0.0, 3.4)]),
    Arc {
        cx: 3.4,
        cy: 3.4,
        rx: 3.4,
        ry: 3.4,
        a0: 180.0,
        a1: 360.0,
    },
    Poly(&[(6.8, 0.0), (6.8, 7.0)]),
];
const G_LOW_V: &[Stroke] = &[Poly(&[(0.0, 7.0), (3.4, 0.0), (6.8, 7.0)])];
const G_LOW_W: &[Stroke] = &[Poly(&[
    (0.0, 7.0),
    (1.9, 0.0),
    (4.4, 5.0),
    (6.9, 0.0),
    (8.8, 7.0),
])];
const G_LOW_X: &[Stroke] = &[
    Poly(&[(0.0, 7.0), (6.4, 0.0)]),
    Poly(&[(0.0, 0.0), (6.4, 7.0)]),
];
const G_LOW_Y: &[Stroke] = &[
    Poly(&[(0.0, 7.0), (3.4, 0.0)]),
    Poly(&[(6.8, 7.0), (1.5, -3.0)]),
];
const G_LOW_Z: &[Stroke] = &[Poly(&[(0.0, 7.0), (6.4, 7.0), (0.0, 0.0), (6.4, 0.0)])];

// ---------------------------------------------------------------------------
// Digits. Lining figures, cap height, all one width so columns line up.
// ---------------------------------------------------------------------------

const G_0: &[Stroke] = &[Arc {
    cx: 3.3,
    cy: 5.0,
    rx: 3.3,
    ry: 5.0,
    a0: 0.0,
    a1: 360.0,
}];
const G_1: &[Stroke] = &[Poly(&[(1.0, 8.2), (3.3, 10.0), (3.3, 0.0)])];
const G_2: &[Stroke] = &[
    Arc {
        cx: 3.3,
        cy: 7.0,
        rx: 3.0,
        ry: 3.0,
        a0: 200.0,
        a1: -30.0,
    },
    Poly(&[(5.9, 5.5), (0.2, 0.0)]),
    Poly(&[(0.2, 0.0), (6.5, 0.0)]),
];
const G_3: &[Stroke] = &[
    Arc {
        cx: 3.3,
        cy: 7.5,
        rx: 2.8,
        ry: 2.5,
        a0: 155.0,
        a1: -75.0,
    },
    Arc {
        cx: 3.3,
        cy: 2.6,
        rx: 3.2,
        ry: 2.6,
        a0: 75.0,
        a1: -160.0,
    },
];
const G_4: &[Stroke] = &[
    Poly(&[(5.0, 0.0), (5.0, 10.0)]),
    Poly(&[(5.0, 10.0), (0.2, 3.2)]),
    Poly(&[(0.2, 3.2), (6.5, 3.2)]),
];
const G_5: &[Stroke] = &[
    Poly(&[(6.0, 10.0), (1.2, 10.0), (0.95, 5.72), (2.13, 5.72)]),
    Arc {
        cx: 3.4,
        cy: 3.0,
        rx: 3.0,
        ry: 3.0,
        a0: 115.0,
        a1: -115.0,
    },
];
const G_6: &[Stroke] = &[
    Arc {
        cx: 3.3,
        cy: 3.4,
        rx: 3.3,
        ry: 3.4,
        a0: 0.0,
        a1: 360.0,
    },
    Arc {
        cx: 3.3,
        cy: 6.6,
        rx: 3.3,
        ry: 3.4,
        a0: 180.0,
        a1: 95.0,
    },
];
const G_7: &[Stroke] = &[Poly(&[(0.2, 10.0), (6.5, 10.0), (2.3, 0.0)])];
const G_8: &[Stroke] = &[
    Arc {
        cx: 3.3,
        cy: 7.4,
        rx: 2.6,
        ry: 2.6,
        a0: 0.0,
        a1: 360.0,
    },
    Arc {
        cx: 3.3,
        cy: 2.6,
        rx: 3.1,
        ry: 2.6,
        a0: 0.0,
        a1: 360.0,
    },
];
const G_9: &[Stroke] = &[
    Arc {
        cx: 3.3,
        cy: 6.6,
        rx: 3.3,
        ry: 3.4,
        a0: 0.0,
        a1: 360.0,
    },
    Arc {
        cx: 3.3,
        cy: 3.4,
        rx: 3.3,
        ry: 3.4,
        a0: 0.0,
        a1: -85.0,
    },
];

// ---------------------------------------------------------------------------
// Punctuation.
// ---------------------------------------------------------------------------

const G_DOT: &[Stroke] = &[Dot {
    x: 1.3,
    y: 0.75,
    r: 0.85,
}];
const G_COMMA: &[Stroke] = &[
    Dot {
        x: 1.3,
        y: 0.75,
        r: 0.85,
    },
    Poly(&[(1.4, 0.3), (0.5, -1.9)]),
];
const G_COLON: &[Stroke] = &[
    Dot {
        x: 1.3,
        y: 0.75,
        r: 0.85,
    },
    Dot {
        x: 1.3,
        y: 4.6,
        r: 0.85,
    },
];
const G_SEMI: &[Stroke] = &[
    Dot {
        x: 1.3,
        y: 4.6,
        r: 0.85,
    },
    Dot {
        x: 1.3,
        y: 0.75,
        r: 0.85,
    },
    Poly(&[(1.4, 0.3), (0.5, -1.9)]),
];
const G_BANG: &[Stroke] = &[
    Poly(&[(1.3, 10.0), (1.3, 2.6)]),
    Dot {
        x: 1.3,
        y: 0.85,
        r: 0.9,
    },
];
const G_QUERY: &[Stroke] = &[
    Arc {
        cx: 2.9,
        cy: 7.7,
        rx: 2.4,
        ry: 2.3,
        a0: 200.0,
        a1: -25.0,
    },
    Poly(&[(5.08, 6.73), (2.9, 4.4)]),
    Poly(&[(2.9, 4.4), (2.9, 2.8)]),
    Dot {
        x: 2.9,
        y: 0.9,
        r: 0.9,
    },
];
const G_MINUS: &[Stroke] = &[Poly(&[(0.5, 4.3), (4.7, 4.3)])];
const G_PLUS: &[Stroke] = &[
    Poly(&[(0.6, 4.3), (6.0, 4.3)]),
    Poly(&[(3.3, 1.6), (3.3, 7.0)]),
];
const G_EQ: &[Stroke] = &[
    Poly(&[(0.6, 5.5), (6.0, 5.5)]),
    Poly(&[(0.6, 3.1), (6.0, 3.1)]),
];
const G_STAR: &[Stroke] = &[
    Poly(&[(2.6, 9.8), (2.6, 5.6)]),
    Poly(&[(0.8, 8.7), (4.4, 6.7)]),
    Poly(&[(0.8, 6.7), (4.4, 8.7)]),
];
const G_SLASH: &[Stroke] = &[Poly(&[(0.2, -1.2), (4.8, 10.8)])];
const G_BACKSLASH: &[Stroke] = &[Poly(&[(0.2, 10.8), (4.8, -1.2)])];
const G_PIPE: &[Stroke] = &[Poly(&[(1.5, -1.2), (1.5, 10.8)])];
const G_LPAREN: &[Stroke] = &[Arc {
    cx: 4.4,
    cy: 4.4,
    rx: 3.6,
    ry: 6.4,
    a0: 145.0,
    a1: 215.0,
}];
const G_RPAREN: &[Stroke] = &[Arc {
    cx: 0.4,
    cy: 4.4,
    rx: 3.6,
    ry: 6.4,
    a0: 35.0,
    a1: -35.0,
}];
const G_LBRACK: &[Stroke] = &[Poly(&[(3.4, -1.0), (1.0, -1.0), (1.0, 10.4), (3.4, 10.4)])];
const G_RBRACK: &[Stroke] = &[Poly(&[(1.0, -1.0), (3.4, -1.0), (3.4, 10.4), (1.0, 10.4)])];
const G_LBRACE: &[Stroke] = &[Poly(&[
    (3.6, 10.4),
    (2.4, 10.4),
    (2.4, 5.6),
    (1.0, 4.7),
    (2.4, 3.8),
    (2.4, -1.0),
    (3.6, -1.0),
])];
const G_RBRACE: &[Stroke] = &[Poly(&[
    (1.0, 10.4),
    (2.2, 10.4),
    (2.2, 5.6),
    (3.6, 4.7),
    (2.2, 3.8),
    (2.2, -1.0),
    (1.0, -1.0),
])];
const G_APOS: &[Stroke] = &[Poly(&[(1.2, 10.0), (1.2, 7.2)])];
const G_QUOTE: &[Stroke] = &[
    Poly(&[(0.9, 10.0), (0.9, 7.2)]),
    Poly(&[(2.7, 10.0), (2.7, 7.2)]),
];
const G_PERCENT: &[Stroke] = &[
    Arc {
        cx: 1.9,
        cy: 8.1,
        rx: 1.7,
        ry: 1.9,
        a0: 0.0,
        a1: 360.0,
    },
    Arc {
        cx: 6.9,
        cy: 1.9,
        rx: 1.7,
        ry: 1.9,
        a0: 0.0,
        a1: 360.0,
    },
    Poly(&[(0.3, 0.0), (8.5, 10.0)]),
];
const G_HASH: &[Stroke] = &[
    Poly(&[(1.6, 0.0), (2.5, 10.0)]),
    Poly(&[(4.6, 0.0), (5.5, 10.0)]),
    Poly(&[(0.4, 3.3), (6.5, 3.3)]),
    Poly(&[(0.8, 6.8), (6.9, 6.8)]),
];
const G_AMP: &[Stroke] = &[
    Poly(&[(6.8, 0.0), (2.6, 4.2)]),
    Arc {
        cx: 3.4,
        cy: 7.6,
        rx: 2.0,
        ry: 2.4,
        a0: -40.0,
        a1: 220.0,
    },
    Arc {
        cx: 3.0,
        cy: 2.4,
        rx: 3.0,
        ry: 2.4,
        a0: 130.0,
        a1: -60.0,
    },
];
const G_AT: &[Stroke] = &[
    Arc {
        cx: 4.4,
        cy: 4.6,
        rx: 4.4,
        ry: 4.6,
        a0: -35.0,
        a1: 290.0,
    },
    Arc {
        cx: 4.4,
        cy: 3.9,
        rx: 2.0,
        ry: 2.2,
        a0: 0.0,
        a1: 360.0,
    },
    Poly(&[(6.4, 3.9), (6.4, 2.6)]),
];
const G_UNDER: &[Stroke] = &[Poly(&[(0.0, -1.8), (6.6, -1.8)])];
const G_LT: &[Stroke] = &[Poly(&[(4.9, 7.3), (0.8, 4.3), (4.9, 1.3)])];
const G_GT: &[Stroke] = &[Poly(&[(1.1, 7.3), (5.2, 4.3), (1.1, 1.3)])];
const G_TILDE: &[Stroke] = &[Poly(&[(0.4, 4.2), (1.9, 5.5), (3.9, 3.5), (5.4, 4.8)])];
const G_CARET: &[Stroke] = &[Poly(&[(0.6, 7.0), (3.2, 10.0), (5.8, 7.0)])];
const G_DOLLAR: &[Stroke] = &[
    Poly(&[
        (6.2, 8.2),
        (5.2, 9.4),
        (2.3, 9.6),
        (0.7, 8.5),
        (0.6, 6.8),
        (1.9, 5.7),
        (4.6, 4.8),
        (6.2, 3.7),
        (6.2, 1.8),
        (4.7, 0.5),
        (1.8, 0.4),
        (0.4, 1.5),
    ]),
    Poly(&[(3.4, -0.8), (3.4, 10.8)]),
];
const G_GRAVE: &[Stroke] = &[Poly(&[(0.6, 10.4), (2.4, 8.6)])];
/// A middle dot, used as a separator in the menu bar and status line.
const G_MIDDOT: &[Stroke] = &[Dot {
    x: 1.6,
    y: 4.3,
    r: 0.85,
}];
/// A right-pointing chevron, for breadcrumbs and disclosure.
const G_CHEVRON: &[Stroke] = &[Poly(&[(1.4, 7.6), (4.6, 4.4), (1.4, 1.2)])];

// ---------------------------------------------------------------------------
// Symbols the interface prints.
//
// These were missing, and a missing glyph draws as a box: the sort header of a
// Files table printed a box where its arrow goes, a folder's size column printed
// a box instead of a dash, "Scanning..." printed one, and the Details pane's
// close button printed one. `tools/scan-nonascii.py` lists what the shell's own
// strings contain and `tests/font_standards.rs` holds the face to it, so the
// gap cannot reopen quietly.
// ---------------------------------------------------------------------------

/// An up arrow: the ascending sort marker, and the "up" affordance generally.
const G_ARROW_UP: &[Stroke] = &[
    Poly(&[(2.9, 0.6), (2.9, 7.4)]),
    Poly(&[(0.9, 5.4), (2.9, 7.4), (4.9, 5.4)]),
];
/// A down arrow.
const G_ARROW_DOWN: &[Stroke] = &[
    Poly(&[(2.9, 7.4), (2.9, 0.6)]),
    Poly(&[(0.9, 2.6), (2.9, 0.6), (4.9, 2.6)]),
];
/// A left arrow.
const G_ARROW_LEFT: &[Stroke] = &[
    Poly(&[(6.4, 4.0), (0.8, 4.0)]),
    Poly(&[(2.6, 1.9), (0.6, 4.0), (2.6, 6.1)]),
];
/// A right arrow.
const G_ARROW_RIGHT: &[Stroke] = &[
    Poly(&[(0.8, 4.0), (6.4, 4.0)]),
    Poly(&[(4.6, 1.9), (6.6, 4.0), (4.6, 6.1)]),
];
/// An em dash, the mark a table uses for "not applicable".
const G_EM_DASH: &[Stroke] = &[Poly(&[(0.4, 4.4), (7.6, 4.4)])];
/// A horizontal ellipsis, for work in progress.
const G_ELLIPSIS: &[Stroke] = &[
    Dot {
        x: 1.0,
        y: 0.75,
        r: 0.8,
    },
    Dot {
        x: 3.6,
        y: 0.75,
        r: 0.8,
    },
    Dot {
        x: 6.2,
        y: 0.75,
        r: 0.8,
    },
];
/// A multiplication sign, on the mathematical axis.
const G_TIMES: &[Stroke] = &[
    Poly(&[(1.8, 1.6), (5.6, 5.4)]),
    Poly(&[(5.6, 1.6), (1.8, 5.4)]),
];
/// A cross, the close affordance. Drawn on the x-height rather than the
/// mathematical axis: it labels a control, not an operation.
const G_CROSS: &[Stroke] = &[
    Poly(&[(1.4, 1.0), (6.0, 5.6)]),
    Poly(&[(6.0, 1.0), (1.4, 5.6)]),
];

/// Every glyph in LitricOS Sans.
pub const GLYPHS: &[Glyph] = &[
    Glyph {
        ch: ' ',
        adv: 4.0,
        strokes: &[],
    },
    Glyph {
        ch: 'A',
        adv: 8.6,
        strokes: G_A,
    },
    Glyph {
        ch: 'B',
        adv: 8.6,
        strokes: G_B,
    },
    Glyph {
        ch: 'C',
        adv: 8.8,
        strokes: G_C,
    },
    Glyph {
        ch: 'D',
        adv: 8.8,
        strokes: G_D,
    },
    Glyph {
        ch: 'E',
        adv: 8.2,
        strokes: G_E,
    },
    Glyph {
        ch: 'F',
        adv: 8.0,
        strokes: G_F,
    },
    Glyph {
        ch: 'G',
        adv: 9.0,
        strokes: G_G,
    },
    Glyph {
        ch: 'H',
        adv: 8.6,
        strokes: G_H,
    },
    Glyph {
        ch: 'I',
        adv: 4.2,
        strokes: G_I,
    },
    Glyph {
        ch: 'J',
        adv: 7.2,
        strokes: G_J,
    },
    Glyph {
        ch: 'K',
        adv: 8.4,
        strokes: G_K,
    },
    Glyph {
        ch: 'L',
        adv: 7.8,
        strokes: G_L,
    },
    Glyph {
        ch: 'M',
        adv: 9.0,
        strokes: G_M,
    },
    Glyph {
        ch: 'N',
        adv: 8.6,
        strokes: G_N,
    },
    Glyph {
        ch: 'O',
        adv: 8.8,
        strokes: G_O,
    },
    Glyph {
        ch: 'P',
        adv: 8.4,
        strokes: G_P,
    },
    Glyph {
        ch: 'Q',
        adv: 8.8,
        strokes: G_Q,
    },
    Glyph {
        ch: 'R',
        adv: 8.6,
        strokes: G_R,
    },
    Glyph {
        ch: 'S',
        adv: 8.4,
        strokes: G_S,
    },
    Glyph {
        ch: 'T',
        adv: 8.2,
        strokes: G_T,
    },
    Glyph {
        ch: 'U',
        adv: 8.6,
        strokes: G_U,
    },
    Glyph {
        ch: 'V',
        adv: 8.4,
        strokes: G_V,
    },
    Glyph {
        ch: 'W',
        adv: 10.8,
        strokes: G_W,
    },
    Glyph {
        ch: 'X',
        adv: 8.4,
        strokes: G_X,
    },
    Glyph {
        ch: 'Y',
        adv: 8.4,
        strokes: G_Y,
    },
    Glyph {
        ch: 'Z',
        adv: 8.4,
        strokes: G_Z,
    },
    Glyph {
        ch: 'a',
        adv: 8.0,
        strokes: G_LOW_A,
    },
    Glyph {
        ch: 'b',
        adv: 8.2,
        strokes: G_LOW_B,
    },
    Glyph {
        ch: 'c',
        adv: 7.8,
        strokes: G_LOW_C,
    },
    Glyph {
        ch: 'd',
        adv: 8.2,
        strokes: G_LOW_D,
    },
    Glyph {
        ch: 'e',
        adv: 8.0,
        strokes: G_LOW_E,
    },
    Glyph {
        ch: 'f',
        adv: 5.4,
        strokes: G_LOW_F,
    },
    Glyph {
        ch: 'g',
        adv: 8.2,
        strokes: G_LOW_G,
    },
    Glyph {
        ch: 'h',
        adv: 8.2,
        strokes: G_LOW_H,
    },
    Glyph {
        ch: 'i',
        adv: 3.4,
        strokes: G_LOW_I,
    },
    Glyph {
        ch: 'j',
        adv: 4.0,
        strokes: G_LOW_J,
    },
    Glyph {
        ch: 'k',
        adv: 7.4,
        strokes: G_LOW_K,
    },
    Glyph {
        ch: 'l',
        adv: 3.4,
        strokes: G_LOW_L,
    },
    Glyph {
        ch: 'm',
        adv: 12.2,
        strokes: G_LOW_M,
    },
    Glyph {
        ch: 'n',
        adv: 8.2,
        strokes: G_LOW_N,
    },
    Glyph {
        ch: 'o',
        adv: 8.2,
        strokes: G_LOW_O,
    },
    Glyph {
        ch: 'p',
        adv: 8.2,
        strokes: G_LOW_P,
    },
    Glyph {
        ch: 'q',
        adv: 8.2,
        strokes: G_LOW_Q,
    },
    Glyph {
        ch: 'r',
        adv: 5.6,
        strokes: G_LOW_R,
    },
    Glyph {
        ch: 's',
        adv: 7.2,
        strokes: G_LOW_S,
    },
    Glyph {
        ch: 't',
        adv: 5.6,
        strokes: G_LOW_T,
    },
    Glyph {
        ch: 'u',
        adv: 8.2,
        strokes: G_LOW_U,
    },
    Glyph {
        ch: 'v',
        adv: 7.8,
        strokes: G_LOW_V,
    },
    Glyph {
        ch: 'w',
        adv: 10.2,
        strokes: G_LOW_W,
    },
    Glyph {
        ch: 'x',
        adv: 7.6,
        strokes: G_LOW_X,
    },
    Glyph {
        ch: 'y',
        adv: 7.8,
        strokes: G_LOW_Y,
    },
    Glyph {
        ch: 'z',
        adv: 7.6,
        strokes: G_LOW_Z,
    },
    Glyph {
        ch: '0',
        adv: 8.0,
        strokes: G_0,
    },
    Glyph {
        ch: '1',
        adv: 8.0,
        strokes: G_1,
    },
    Glyph {
        ch: '2',
        adv: 8.0,
        strokes: G_2,
    },
    Glyph {
        ch: '3',
        adv: 8.0,
        strokes: G_3,
    },
    Glyph {
        ch: '4',
        adv: 8.0,
        strokes: G_4,
    },
    Glyph {
        ch: '5',
        adv: 8.0,
        strokes: G_5,
    },
    Glyph {
        ch: '6',
        adv: 8.0,
        strokes: G_6,
    },
    Glyph {
        ch: '7',
        adv: 8.0,
        strokes: G_7,
    },
    Glyph {
        ch: '8',
        adv: 8.0,
        strokes: G_8,
    },
    Glyph {
        ch: '9',
        adv: 8.0,
        strokes: G_9,
    },
    Glyph {
        ch: '.',
        adv: 3.4,
        strokes: G_DOT,
    },
    Glyph {
        ch: ',',
        adv: 3.4,
        strokes: G_COMMA,
    },
    Glyph {
        ch: ':',
        adv: 3.4,
        strokes: G_COLON,
    },
    Glyph {
        ch: ';',
        adv: 3.4,
        strokes: G_SEMI,
    },
    Glyph {
        ch: '!',
        adv: 3.4,
        strokes: G_BANG,
    },
    Glyph {
        ch: '?',
        adv: 6.4,
        strokes: G_QUERY,
    },
    Glyph {
        ch: '-',
        adv: 5.6,
        strokes: G_MINUS,
    },
    Glyph {
        ch: '+',
        adv: 6.8,
        strokes: G_PLUS,
    },
    Glyph {
        ch: '=',
        adv: 6.8,
        strokes: G_EQ,
    },
    Glyph {
        ch: '*',
        adv: 5.4,
        strokes: G_STAR,
    },
    Glyph {
        ch: '/',
        adv: 5.2,
        strokes: G_SLASH,
    },
    Glyph {
        ch: '\\',
        adv: 5.2,
        strokes: G_BACKSLASH,
    },
    Glyph {
        ch: '|',
        adv: 3.2,
        strokes: G_PIPE,
    },
    Glyph {
        ch: '(',
        adv: 4.4,
        strokes: G_LPAREN,
    },
    Glyph {
        ch: ')',
        adv: 4.4,
        strokes: G_RPAREN,
    },
    Glyph {
        ch: '[',
        adv: 4.4,
        strokes: G_LBRACK,
    },
    Glyph {
        ch: ']',
        adv: 4.4,
        strokes: G_RBRACK,
    },
    Glyph {
        ch: '{',
        adv: 4.8,
        strokes: G_LBRACE,
    },
    Glyph {
        ch: '}',
        adv: 4.8,
        strokes: G_RBRACE,
    },
    Glyph {
        ch: '\'',
        adv: 2.8,
        strokes: G_APOS,
    },
    Glyph {
        ch: '"',
        adv: 4.4,
        strokes: G_QUOTE,
    },
    Glyph {
        ch: '%',
        adv: 9.8,
        strokes: G_PERCENT,
    },
    Glyph {
        ch: '#',
        adv: 8.0,
        strokes: G_HASH,
    },
    Glyph {
        ch: '&',
        adv: 8.6,
        strokes: G_AMP,
    },
    Glyph {
        ch: '@',
        adv: 10.2,
        strokes: G_AT,
    },
    Glyph {
        ch: '_',
        adv: 7.2,
        strokes: G_UNDER,
    },
    Glyph {
        ch: '<',
        adv: 6.0,
        strokes: G_LT,
    },
    Glyph {
        ch: '>',
        adv: 6.0,
        strokes: G_GT,
    },
    Glyph {
        ch: '~',
        adv: 6.4,
        strokes: G_TILDE,
    },
    Glyph {
        ch: '^',
        adv: 6.4,
        strokes: G_CARET,
    },
    Glyph {
        ch: '$',
        adv: 7.6,
        strokes: G_DOLLAR,
    },
    Glyph {
        ch: '`',
        adv: 3.2,
        strokes: G_GRAVE,
    },
    Glyph {
        ch: '\u{00B7}',
        adv: 3.6,
        strokes: G_MIDDOT,
    },
    Glyph {
        ch: '\u{203A}',
        adv: 6.0,
        strokes: G_CHEVRON,
    },
    Glyph {
        ch: '\u{2191}',
        adv: 6.2,
        strokes: G_ARROW_UP,
    },
    Glyph {
        ch: '\u{2193}',
        adv: 6.2,
        strokes: G_ARROW_DOWN,
    },
    Glyph {
        ch: '\u{2190}',
        adv: 7.4,
        strokes: G_ARROW_LEFT,
    },
    Glyph {
        ch: '\u{2192}',
        adv: 7.4,
        strokes: G_ARROW_RIGHT,
    },
    Glyph {
        ch: '\u{2014}',
        adv: 8.4,
        strokes: G_EM_DASH,
    },
    Glyph {
        ch: '\u{2026}',
        adv: 7.6,
        strokes: G_ELLIPSIS,
    },
    Glyph {
        ch: '\u{00D7}',
        adv: 7.8,
        strokes: G_TIMES,
    },
    Glyph {
        ch: '\u{2715}',
        adv: 7.8,
        strokes: G_CROSS,
    },
];

/// The glyph for a character, if the typeface has one.
#[must_use]
pub fn glyph(ch: char) -> Option<&'static Glyph> {
    GLYPHS.iter().find(|g| g.ch == ch)
}

/// The paper a glyph must leave on each side, in pixels, before its neighbour's
/// stroke can reach it.
///
/// A stem is drawn as a path with a round cap, so its ink reaches half a stroke
/// beyond the path's end, on both axes. A glyph whose path reaches its own
/// advance -- `T`'s crossbar, `_`, `—`, most capitals -- therefore touches the
/// letter after it at every size, and two letters whose inks meet read as one
/// shape. Real faces do not solve this by drawing narrower letters: they carry a
/// side bearing, extra advance on each side, and the drawing stays a drawing.
///
/// The floor is half a stroke plus this much paper. It scales with the weight
/// because the overhang does: a bold glyph needs more bearing than a light one
/// for the same reason it needs a thicker stem.
pub const BEARING_GAP_PX: f32 = 0.25;

/// The ink box of glyph `i`, in font units, from its stroke paths.
///
/// Computed once per glyph and cached: the alternative is walking every stroke
/// of every glyph on every draw call, and a frame draws thousands of them.
fn ink_box(i: usize) -> (f32, f32) {
    use std::sync::OnceLock;
    static BOXES: OnceLock<Vec<(f32, f32)>> = OnceLock::new();
    let boxes = BOXES.get_or_init(|| {
        GLYPHS
            .iter()
            .map(|g| {
                let (mut lo, mut hi) = (f32::MAX, f32::MIN);
                let mut see = |x: f32| {
                    lo = lo.min(x);
                    hi = hi.max(x);
                };
                for s in g.strokes {
                    match *s {
                        Stroke::Poly(points) => {
                            for (x, _) in points {
                                see(*x);
                            }
                        }
                        Stroke::Arc {
                            cx,
                            cy,
                            rx,
                            ry,
                            a0,
                            a1,
                        } => {
                            for (x, _) in arc_points(cx, cy, rx, ry, a0, a1, 48) {
                                see(x);
                            }
                        }
                        Stroke::Dot { x, r, .. } => {
                            see(x - r);
                            see(x + r);
                        }
                    }
                }
                if lo > hi {
                    (0.0, 0.0)
                } else {
                    (lo, hi)
                }
            })
            .collect()
    });
    boxes.get(i).copied().unwrap_or((0.0, 0.0))
}

/// The index of a character's glyph, for pairing it with [`ink_box`].
fn glyph_index(ch: char) -> Option<usize> {
    GLYPHS.iter().position(|g| g.ch == ch)
}

/// How much advance a glyph needs on each side at this size, in font units.
///
/// Returns `(left, right)`: the extra advance the caller must add to the
/// authored one, and how far to shift the glyph so the bearings come out even.
/// A glyph that already carries enough bearing on both sides gets `(0, 0)` and
/// is drawn exactly where it was authored.
fn bearings(i: usize, scale: f32, stroke_w: f32) -> (f32, f32) {
    let (min_x, max_x) = ink_box(i);
    // Half a stroke of overhang, plus the paper, in font units.
    let floor = (stroke_w / 2.0 + BEARING_GAP_PX) / scale;
    let adv = GLYPHS.get(i).map_or(0.0, |g| g.adv);
    let left = (floor - min_x).max(0.0);
    let right = (floor - (adv - max_x)).max(0.0);
    (left, right)
}

/// The pen movement after a glyph, in pixels: the authored advance plus the
/// bearings it needs, plus the caller's tracking.
fn advance_px(i: usize, scale: f32, stroke_w: f32, tracking: f32) -> f32 {
    let adv = GLYPHS.get(i).map_or(6.0, |g| g.adv);
    let (left, right) = bearings(i, scale, stroke_w);
    (adv + left + right) * scale + tracking
}

/// How wide `text` will be at `size` pixels of cap height, plus `tracking`
/// pixels between characters.
///
/// Measuring and drawing walk the same advance table, so a right-aligned label
/// cannot drift away from where it was measured to.
#[must_use]
pub fn measure(text: &str, style: &Style) -> f32 {
    let localized = crate::i18n::text(text);
    let text = localized.as_ref();
    if let Some(system) = open_type() {
        let face = selected_face(style);
        if let Some(run) = system.layout(text, face, style.size, style.tracking) {
            return run.width;
        }
    }
    measure_vector(text, style)
}

fn measure_vector(text: &str, style: &Style) -> f32 {
    let (cap, tracking) = (cap_px(style.size), style.tracking);
    let scale = cap / CAP;
    let stroke_w = (cap * style.weight).max(1.0);
    let mut w = 0.0f32;
    let mut n = 0u32;
    for ch in text.chars() {
        match glyph_index(ch) {
            Some(i) => w += advance_px(i, scale, stroke_w, 0.0),
            // A character with no glyph is drawn as a box of this width; the
            // measurement has to agree with the drawing or a right-aligned
            // label drifts away from where it was measured to.
            None => w += 6.0 * scale + 2.0 * (stroke_w / 2.0 + BEARING_GAP_PX),
        }
        n = n.saturating_add(1);
    }
    w + tracking * f32::from(n.saturating_sub(1).min(u32::from(u16::MAX)) as u16)
}

/// Where a character's ink lands, in pixels from the pen position, as
/// [`font::draw`] will place it: `(left, right)` of the strokes' outer edge.
///
/// Layout asks this question when it needs to know whether two things will
/// touch -- the strength of a label against the field beside it, the air
/// between a column heading and the arrow on it -- and `tests/font_standards.rs`
/// uses it to hold the face to its own side bearings.
#[must_use]
pub fn ink_span(ch: char, style: &Style) -> Option<(f32, f32)> {
    let i = glyph_index(ch)?;
    let scale = cap_px(style.size) / CAP;
    let stroke_w = (cap_px(style.size) * style.weight).max(1.0);
    let (min_x, max_x) = ink_box(i);
    let (left, _) = bearings(i, scale, stroke_w);
    let shift = left * scale;
    Some((
        shift + min_x * scale - stroke_w / 2.0,
        shift + max_x * scale + stroke_w / 2.0,
    ))
}

/// The paper between two characters set next to each other, in pixels, at this
/// style and with no tracking. Negative means their inks overlap.
#[must_use]
pub fn pair_gap(a: char, b: char, style: &Style) -> f32 {
    let (Some((_, a_right)), Some((b_left, _))) = (ink_span(a, style), ink_span(b, style)) else {
        return 0.0;
    };
    // `ink_span` describes the vector fallback glyphs, so its pen advance has
    // to come from that same face. Mixing OpenType advances with vector bounds
    // reports false overlaps and makes the typography audit meaningless.
    let advance = measure_vector(&a.to_string(), style);
    advance - a_right + b_left
}

/// Draw `text` with its baseline at `(x, baseline)`, in pixels.
///
/// Returns the x the pen finished at, so callers can chain runs of different
/// colour or weight without measuring twice.
pub fn draw(canvas: &mut Canvas, text: &str, x: f32, baseline: f32, style: &Style) -> f32 {
    let localized = crate::i18n::text(text);
    draw_literal(canvas, localized.as_ref(), x, baseline, style)
}

/// Draw text exactly as it is passed: no catalogue lookup.
///
/// A section label is a structural keyword -- `POWER`, `SYSTEM`,
/// `ENVIRONMENT` -- and it is drawn in letter-spaced capitals, which is a form
/// only the Latin alphabet has. Translating one produced a Thai word at a size
/// where its vowel marks collide into an unreadable smear, so those labels are
/// drawn literally and the dictionary is left for prose.
pub fn draw_literal(canvas: &mut Canvas, text: &str, x: f32, baseline: f32, style: &Style) -> f32 {
    if let Some(system) = open_type() {
        let face = selected_face(style);
        if let Some(run) = system.layout(text, face, style.size, style.tracking) {
            return draw_open_type(canvas, text, x, baseline, style, &run);
        }
    }
    let Style {
        size,
        weight,
        tracking,
        color,
        ..
    } = *style;
    let cap = cap_px(size);
    let scale = cap / CAP;
    let stroke_w = (cap * weight).max(1.0);
    let mut pen = x;
    // Glyphs cost real pixels and there are thousands of them in a frame; the
    // frame's pixel bill should say what share went to type rather than folding
    // it into the fills. Restored before returning, so a caller's own tally is
    // not silently relabelled.
    let prev_site = canvas.set_site(crate::canvas::Site::Text);
    for ch in text.chars() {
        let Some(i) = glyph_index(ch) else {
            // A missing glyph draws a box. Silence would make the gap look
            // like a spacing bug rather than a typeface that is short a
            // character, and the two are fixed in different places.
            let pad = stroke_w / 2.0 + BEARING_GAP_PX;
            let w = 6.0 * scale;
            canvas.stroke_round_rect(
                crate::canvas::Rect::new(pen + pad, baseline - cap - pad, w, cap + pad * 2.0),
                1.0,
                stroke_w,
                color,
            );
            pen += w + pad * 2.0 + tracking;
            continue;
        };
        // A run whose glyph is out of the table falls back to the first one,
        // and a table with no glyphs at all draws nothing: indexing would be a
        // panic in the middle of a frame, which is the one place a panic must
        // never happen.
        let Some(g) = GLYPHS.get(i).or_else(|| GLYPHS.first()) else {
            continue;
        };
        // The bearings are part of the advance, so the glyph is shifted by the
        // left one before it is drawn: the ink lands where the bearings say.
        let (left, _) = bearings(i, scale, stroke_w);
        draw_glyph(
            canvas,
            g,
            pen + left * scale,
            baseline,
            scale,
            stroke_w,
            color,
        );
        pen += advance_px(i, scale, stroke_w, tracking);
    }
    // Recorded only when someone asked, so the layout audit can find text that
    // was drawn over other text. Off in every frame nobody is auditing: one
    // comparison, no allocation.
    canvas.set_site(prev_site);
    if canvas.is_recording_text() {
        canvas.record_run(crate::canvas::TextRun {
            text: text.to_owned(),
            x,
            end_x: pen,
            baseline,
            size,
            surface: canvas.surface(),
        });
    }
    pen
}

fn selected_face(style: &Style) -> FontFace {
    match (style.face, style.weight >= WEIGHT_BOLD) {
        (FontFace::Sans, true) => FontFace::SansBold,
        (FontFace::Mono, true) => FontFace::MonoBold,
        (face, false) => face,
        (face, true) => face,
    }
}

fn draw_open_type(
    canvas: &mut Canvas,
    text: &str,
    x: f32,
    baseline: f32,
    style: &Style,
    run: &litric_text::Run,
) -> f32 {
    let prev_site = canvas.set_site(crate::canvas::Site::Text);
    for glyph in &run.glyphs {
        let gx = (x + glyph.x).floor() as i64;
        let gy = (baseline + glyph.y).floor() as i64;
        for row in 0..glyph.height {
            let Some(y) = gy.checked_add(i64::try_from(row).unwrap_or(i64::MAX)) else {
                continue;
            };
            for col in 0..glyph.width {
                let Some(at) = row
                    .checked_mul(glyph.width)
                    .and_then(|base| base.checked_add(col))
                else {
                    continue;
                };
                let Some(coverage) = glyph.coverage.get(at).copied() else {
                    continue;
                };
                if coverage == 0 {
                    continue;
                }
                let Some(px) = gx.checked_add(i64::try_from(col).unwrap_or(i64::MAX)) else {
                    continue;
                };
                canvas.blend_px(px, y, style.color, f32::from(coverage) / 255.0);
            }
        }
    }
    canvas.set_site(prev_site);
    let end_x = x + run.width;
    if canvas.is_recording_text() {
        canvas.record_run(crate::canvas::TextRun {
            text: text.to_owned(),
            x,
            end_x,
            baseline,
            size: style.size,
            surface: canvas.surface(),
        });
    }
    end_x
}

/// Draw one glyph. `scale` converts font units to pixels; font `y` points up,
/// so it is subtracted from the baseline.
fn draw_glyph(
    canvas: &mut Canvas,
    g: &Glyph,
    x: f32,
    baseline: f32,
    scale: f32,
    stroke_w: f32,
    color: u32,
) {
    let map = |p: (f32, f32)| -> (f32, f32) { (x + p.0 * scale, baseline - p.1 * scale) };
    for s in g.strokes {
        match *s {
            Poly(points) => {
                let mut prev: Option<(f32, f32)> = None;
                for p in points {
                    let q = map(*p);
                    if let Some(a) = prev {
                        canvas.stroke_line(a.0, a.1, q.0, q.1, stroke_w, color);
                    }
                    prev = Some(q);
                }
            }
            Arc {
                cx,
                cy,
                rx,
                ry,
                a0,
                a1,
            } => {
                // Steps proportional to the sweep and the radius, so a small
                // comma is not paying for the segment count of a capital O and
                // a big O is not visibly faceted.
                let sweep = (a1 - a0).abs();
                let radius_px = rx.max(ry) * scale;
                let steps = ((sweep / 360.0) * radius_px * 1.6).clamp(6.0, 96.0) as u32;
                let pts = arc_points(cx, cy, rx, ry, a0, a1, steps);
                let mut prev: Option<(f32, f32)> = None;
                for p in &pts {
                    let q = map(*p);
                    if let Some(a) = prev {
                        canvas.stroke_line(a.0, a.1, q.0, q.1, stroke_w, color);
                    }
                    prev = Some(q);
                }
            }
            Dot { x: dx, y: dy, r } => {
                let q = map((dx, dy));
                // The dot's radius is in font units but must never render
                // thinner than the stroke it belongs to, or an 'i' loses its
                // tittle at small sizes.
                canvas.fill_circle(q.0, q.1, (r * scale).max(stroke_w / 2.0), color);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::canvas::{luminance, rgb, Rect};

    #[test]
    fn every_printable_ascii_character_has_a_glyph() {
        let missing: Vec<char> = (0x20u8..0x7Fu8)
            .map(char::from)
            .filter(|c| glyph(*c).is_none())
            .collect();
        assert!(missing.is_empty(), "no glyph for: {missing:?}");
    }

    #[test]
    fn no_character_is_defined_twice() {
        let mut seen: Vec<char> = GLYPHS.iter().map(|g| g.ch).collect();
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(before, seen.len(), "a character appears twice in GLYPHS");
    }

    #[test]
    fn every_glyph_has_a_positive_advance() {
        for g in GLYPHS {
            assert!(g.adv > 0.0, "{:?} has advance {}", g.ch, g.adv);
        }
    }

    #[test]
    fn every_glyph_except_space_actually_draws_something() {
        for g in GLYPHS {
            if g.ch == ' ' {
                assert!(g.strokes.is_empty(), "space must be blank");
                continue;
            }
            assert!(!g.strokes.is_empty(), "{:?} has no strokes", g.ch);
        }
    }

    #[test]
    fn glyphs_stay_inside_their_advance_and_their_vertical_bounds() {
        // A glyph that overruns its advance collides with the next letter, and
        // one that overruns the line box collides with the line above. Both are
        // invisible in a single specimen and obvious in a paragraph.
        for g in GLYPHS {
            for s in g.strokes {
                let pts: Vec<(f32, f32)> = match *s {
                    Poly(p) => p.to_vec(),
                    Arc {
                        cx,
                        cy,
                        rx,
                        ry,
                        a0,
                        a1,
                    } => arc_points(cx, cy, rx, ry, a0, a1, 24),
                    Dot { x, y, r } => vec![(x - r, y - r), (x + r, y + r)],
                };
                for (px, py) in pts {
                    assert!(px >= -0.6, "{:?} starts left of its origin at {px}", g.ch);
                    assert!(
                        px <= g.adv + 0.6,
                        "{:?} is {px} wide but advances only {}",
                        g.ch,
                        g.adv
                    );
                    assert!(py <= 11.2, "{:?} rises to {py}, above the cap line", g.ch);
                    assert!(py >= -3.2, "{:?} drops to {py}, below the descender", g.ch);
                }
            }
        }
    }

    #[test]
    fn measuring_matches_where_drawing_leaves_the_pen() {
        // The invariant that matters is not "the sum of the authored advances":
        // it is that the width a caller measures is the width the pen travels,
        // bearings included. A right-aligned label is placed from `measure` and
        // drawn by `draw`; if those two ever disagree the label drifts from the
        // edge it was aligned to, by an amount that grows with the string.
        let style = Style::new(20.0, 0);
        for text in [
            "AV",
            "AVATAR",
            "Tj _— ✕↑↓",
            "Text With Spaces",
            "1234567890",
            "Mm/WW",
        ] {
            let mut canvas = Canvas::new(400, 60, 0);
            let drawn = draw(&mut canvas, text, 10.0, 40.0, &style);
            let measured = 10.0 + measure(text, &style);
            assert!(
                (drawn - measured).abs() < 1e-3,
                "{text:?}: drawn to {drawn}, measured to {measured}"
            );
        }
    }

    #[test]
    fn a_capital_is_as_tall_as_the_tokens_say_it_is() {
        // `Style::size` is an em. `TypeStep::px()` is documented as "size in
        // logical pixels", its WCAG large-text test is defined on ems, and every
        // centring call in the shell offsets a baseline by `size * 0.36` — which
        // is half of a 0.72-em cap. The vector fallback used to read that same
        // number as a cap height and draw roughly 40% too large, so a machine
        // without the installed faces showed a different interface from the one
        // the layout audits were checking, and short labels were elided that
        // would have fitted (`Documents` inside an 80px box became `Docum…`).
        // This pins the invariant on both paths: the ink height of a capital is
        // the cap height of the requested size.
        for size in [12.0_f32, 17.0, 38.0] {
            let style = Style::new(size, rgb(0xFFFFFF));
            let mut c = Canvas::new(160, 220, rgb(0x000000));
            draw(&mut c, "H", 20.0, 180.0, &style);
            let mut top: Option<i64> = None;
            let mut bottom: Option<i64> = None;
            for y in 0..220i64 {
                let inked =
                    (0..160i64).any(|x| crate::canvas::luminance(c.get(x, y).unwrap_or(0)) > 0.2);
                if inked {
                    top.get_or_insert(y);
                    bottom = Some(y);
                }
            }
            let (Some(top), Some(bottom)) = (top, bottom) else {
                panic!("a capital at {size}px drew nothing");
            };
            let height = (bottom - top + 1) as f32;
            let cap = cap_px(size);
            // Above the cap by more than a stroke's overhang means the size was
            // read as a cap height; below it means the face collapsed.
            assert!(
                height <= cap * 1.25,
                "a capital at {size}px is {height}px tall, but a {cap}px cap was asked for"
            );
            assert!(
                height >= cap * 0.90,
                "a capital at {size}px is only {height}px tall against a {cap}px cap"
            );
        }
    }

    #[test]
    fn tracking_is_applied_between_characters_and_not_after_the_last() {
        let a = measure("AAA", &Style::new(20.0, 0));
        let b = measure("AAA", &Style::new(20.0, 0).tracked(2.0));
        assert!(
            (b - a - 4.0).abs() < 1e-3,
            "expected two gaps, got {}",
            b - a
        );
        assert!(
            (measure("A", &Style::new(20.0, 0).tracked(5.0)) - measure("A", &Style::new(20.0, 0)))
                .abs()
                < 1e-3
        );
    }

    #[test]
    fn drawing_puts_ink_on_the_canvas_within_the_measured_box() {
        let mut c = Canvas::new(200, 60, rgb(0x000000));
        let st = Style::new(24.0, rgb(0xFFFFFF));
        let w = measure("Litric", &st);
        draw(&mut c, "Litric", 10.0, 44.0, &st);

        let mut ink = 0u32;
        for y in 0..60i64 {
            for x in 0..200i64 {
                if luminance(c.get(x, y).unwrap_or(0)) > 0.2 {
                    ink = ink.saturating_add(1);
                    assert!(
                        (x as f32) >= 8.0 && (x as f32) <= 10.0 + w + 2.0,
                        "ink at x={x} is outside the measured width {w}"
                    );
                }
            }
        }
        assert!(ink > 150, "only {ink} lit pixels; the text did not draw");
    }

    #[test]
    fn text_sits_on_its_baseline_rather_than_over_it() {
        // Everything below the baseline must be a descender. If the renderer
        // forgets to flip y, the whole line lands above where it was asked for
        // and every layout in the shell is silently wrong by a line height.
        let mut c = Canvas::new(120, 80, rgb(0x000000));
        draw(
            &mut c,
            "HELLO",
            10.0,
            50.0,
            &Style::new(20.0, rgb(0xFFFFFF)),
        );
        let mut lowest = 0i64;
        let mut highest = 80i64;
        for y in 0..80i64 {
            for x in 0..120i64 {
                if luminance(c.get(x, y).unwrap_or(0)) > 0.2 {
                    lowest = lowest.max(y);
                    highest = highest.min(y);
                }
            }
        }
        assert!(
            lowest <= 52,
            "capitals extend to y={lowest}, below the baseline"
        );
        assert!(
            highest >= 27,
            "capitals reach y={highest}, above the cap line"
        );
    }

    #[test]
    fn an_unknown_character_draws_a_box_rather_than_nothing() {
        let mut c = Canvas::new(60, 60, rgb(0x000000));
        draw(
            &mut c,
            "\u{4E2D}",
            10.0,
            44.0,
            &Style::new(24.0, rgb(0xFFFFFF)),
        );
        let lit = (0..60i64)
            .flat_map(|y| (0..60i64).map(move |x| (x, y)))
            .filter(|(x, y)| luminance(c.get(*x, *y).unwrap_or(0)) > 0.2)
            .count();
        assert!(lit > 20, "a missing glyph drew nothing at all");
    }

    #[test]
    fn a_heavier_weight_puts_down_more_ink() {
        let initialized = initialize();
        eprintln!("OpenType stack initialized: {initialized}");
        let regular_style = Style::new(22.0, rgb(0xFFFFFF));
        let mut bold_style = regular_style;
        bold_style.weight = WEIGHT_BOLD;
        let regular_run = open_type()
            .and_then(|system| system.layout("Weight", selected_face(&regular_style), 22.0, 0.0));
        let bold_run = open_type()
            .and_then(|system| system.layout("Weight", selected_face(&bold_style), 22.0, 0.0));
        eprintln!(
            "faces={:?}/{:?}, widths={:?}/{:?}, equal={}",
            selected_face(&regular_style),
            selected_face(&bold_style),
            regular_run.as_ref().map(|r| r.width),
            bold_run.as_ref().map(|r| r.width),
            regular_run
                .as_ref()
                .zip(bold_run.as_ref())
                .is_some_and(|(a, b)| a
                    .glyphs
                    .first()
                    .zip(b.glyphs.first())
                    .is_some_and(|(x, y)| x.coverage == y.coverage))
        );
        let ink = |weight: f32| -> f64 {
            let mut c = Canvas::new(120, 60, rgb(0x000000));
            let mut st = Style::new(22.0, rgb(0xFFFFFF));
            st.weight = weight;
            draw(&mut c, "Weight", 6.0, 44.0, &st);
            // Sum coverage instead of counting pixels above a hard threshold:
            // OpenType antialiasing can make bold ink darker without changing
            // how many samples happen to cross 50% coverage.
            c.pixels()
                .iter()
                .fold(0.0, |sum, px| sum + f64::from(luminance(*px)))
        };
        let regular = ink(WEIGHT_REGULAR);
        let bold = ink(WEIGHT_BOLD);
        assert!(
            bold > regular,
            "bold is not heavier than regular: bold={bold}, regular={regular}"
        );
    }

    #[test]
    fn drawing_nothing_costs_nothing_and_returns_the_pen_unmoved() {
        let mut c = Canvas::new(20, 20, rgb(0x000000));
        let end = draw(&mut c, "", 7.0, 15.0, &Style::new(12.0, rgb(0xFFFFFF)));
        assert!((end - 7.0).abs() < 1e-6);
        assert!(c.pixels().iter().all(|p| *p == rgb(0x000000)));
    }

    #[test]
    fn text_is_clipped_by_the_canvas_rather_than_writing_out_of_bounds() {
        let mut c = Canvas::new(24, 24, rgb(0x000000));
        draw(
            &mut c,
            "Wide text",
            -40.0,
            12.0,
            &Style::bold(30.0, rgb(0xFFFFFF)),
        );
        draw(
            &mut c,
            "Wide text",
            400.0,
            12.0,
            &Style::bold(30.0, rgb(0xFFFFFF)),
        );
        // Reaching here without a panic is the assertion; the canvas bounds
        // check is what makes an off-screen label safe rather than fatal.
        assert_eq!(c.width(), 24);
        let _ = Rect::new(0.0, 0.0, 1.0, 1.0);
    }
}
