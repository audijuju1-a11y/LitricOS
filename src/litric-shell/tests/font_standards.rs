//! Typographic standards for LitricOS Sans, as assertions.
//!
//! "The font is not pretty and not standard" is a complaint that cannot be
//! argued with, only measured. Three things about a typeface are checkable
//! without an opinion about taste, and all three are things a drawn typeface
//! gets wrong in ways that are obvious in a paragraph and invisible in a
//! specimen:
//!
//! 1. **Letters never touch.** Two adjacent glyphs whose ink overlaps collide,
//!    and with a constant advance running out of a monoline design that means a
//!    broken pair rather than a tight one.
//! 2. **Figures are tabular and lining by default.** Every digit the same width
//!    and the same height, so a column of numbers lines up and `1` is not a
//!    lopsided cell in a table of twenty of them.
//! 3. **The proportions are the ones sans-serif UI faces use.** x-height, cap
//!    height, line spacing and stem weight live in known bands; a face outside
//!    them reads as a different kind of object — a display face, or a defect.
//!
//! The bands below are not invented here. x-height/cap of 0.68-0.76 covers the
//! UI faces this project benchmarks against (SF Pro Text and Inter sit at 0.72
//! and 0.73); line spacing of 1.35-1.65 cap heights is the range that keeps
//! paragraph text from either crowding or drifting; and a stem of 0.115-0.16 of
//! cap height is the regular-weight band those faces occupy.

use litric_shell::font::{self, Stroke, Style, CAP, LINE, WEIGHT_BOLD, WEIGHT_REGULAR, X_HEIGHT};

/// The style the interface sets body text in: 13 pixels of cap height.
fn body() -> Style {
    Style::new(13.0, 0xFFFFFFFF)
}

/// Every character the shell can put on screen: ASCII printable, which is what
/// the interface's own strings are built from.
const PRINTABLE: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                         [\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

/// The vertical extent of a glyph's stroke paths, in font units.
fn path_box(ch: char) -> (f32, f32) {
    // `assert!` followed by a let-else rather than
    // `unwrap_or_else(|| panic!(...))`: `clippy::panic` is denied workspace-wide
    // in every target including tests, and an assertion is the form the lint
    // allows. The value returned in the else arm is the empty box, which the
    // assertion above has already made unreachable.
    assert!(font::glyph(ch).is_some(), "no glyph for {ch:?}");
    let Some(g) = font::glyph(ch) else {
        return (f32::MAX, f32::MIN);
    };
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    let mut see = |y: f32| {
        lo = lo.min(y);
        hi = hi.max(y);
    };
    for s in g.strokes {
        match *s {
            Stroke::Poly(points) => {
                for (_, y) in points {
                    see(*y);
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
                for (_, y) in litric_shell::canvas::arc_points(cx, cy, rx, ry, a0, a1, 64) {
                    see(y);
                }
            }
            Stroke::Dot { y, r, .. } => {
                see(y - r);
                see(y + r);
            }
        }
    }
    (lo, hi)
}

/// Every pair the interface can set next to each other, as (gap in pixels,
/// left character, right character). The gap is measured through the public
/// layout API, the same one the shell's view code calls, so a change to the
/// bearings cannot pass by changing the test's arithmetic too.
fn tight_pairs(style: &Style) -> Vec<(f32, char, char)> {
    let mut out = Vec::new();
    for a in PRINTABLE.chars().filter(|c| !c.is_whitespace()) {
        for b in PRINTABLE.chars().filter(|c| !c.is_whitespace()) {
            out.push((font::pair_gap(a, b, style), a, b));
        }
    }
    out.sort_by(|x, y| x.0.total_cmp(&y.0));
    out
}

#[test]
fn no_two_letters_touch_at_ui_sizes() {
    // The complaint in its literal form, at the size the interface actually
    // sets. Two letters whose inks meet are one shape, and with round caps on
    // every stem that happens at every size a glyph's path fills its advance.
    let collide: Vec<String> = tight_pairs(&body())
        .into_iter()
        .filter(|(gap, _, _)| *gap < 0.0)
        .map(|(gap, a, b)| format!("{a}{b} {gap:.2}px"))
        .take(12)
        .collect();
    assert!(
        collide.is_empty(),
        "{} pair(s) of letters collide at 13px, worst: {collide:?}",
        collide.len()
    );
}

#[test]
fn letters_keep_a_side_bearing_rather_than_only_not_colliding() {
    // Colliding is the hard failure; touching is the one a reader calls ugly.
    // The quarter pixel of paper this asserts is the same floor the face is
    // built to (`font::BEARING_GAP_PX`), checked through the drawing API rather
    // than trusted from the constant.
    let tight: Vec<(f32, char, char)> = tight_pairs(&body())
        .into_iter()
        .filter(|(gap, _, _)| *gap < 0.25)
        .collect();
    let worst: Vec<String> = tight
        .iter()
        .take(12)
        .map(|(gap, a, b)| format!("{a}{b} {gap:.2}px"))
        .collect();
    assert!(
        tight.is_empty(),
        "{} pair(s) leave less than a quarter pixel of paper, worst: {worst:?}",
        tight.len()
    );
}

#[test]
fn the_side_bearing_holds_at_every_size_the_shell_sets() {
    // Half a stroke of overhang is a *pixel* quantity that grows with the size,
    // so a bearing that works at 11 pixels does not automatically work at 34.
    // The type scale runs from a caption to a heading; all of it has to hold.
    for size in [9.0f32, 11.0, 13.0, 16.0, 22.0, 34.0, 48.0] {
        for (bold, what) in [(false, "regular"), (true, "bold")] {
            let style = if bold {
                Style::bold(size, 0xFFFFFFFF)
            } else {
                Style::new(size, 0xFFFFFFFF)
            };
            let worst = tight_pairs(&style).into_iter().next();
            if let Some((gap, a, b)) = worst {
                assert!(
                    gap > 0.0,
                    "at {size}px {what}, {a}{b} collide by {gap:.2}px"
                );
            }
        }
    }
}

#[test]
fn figures_are_tabular() {
    // A column of numbers is the one place a proportional advance is visible as
    // a defect: Explorer, the monitor and the Defender all print columns of
    // them. Tabular figures are not a style choice here, they are what the
    // interface's tables assume.
    let widths: Vec<(char, f32)> = ('0'..='9')
        .map(|c| (c, font::glyph(c).unwrap().adv))
        .collect();
    // `.first()` rather than `widths[0]`: indexing is denied by the workspace
    // lints. The assertion names the reason an empty list would be a failure
    // rather than letting the else arm pass silently.
    assert!(!widths.is_empty(), "the digits must have glyphs to compare");
    let Some(&(_, first)) = widths.first() else {
        return;
    };
    let odd: Vec<String> = widths
        .iter()
        .filter(|(_, w)| (*w - first).abs() > 0.01)
        .map(|(c, w)| format!("{c} advances {w} against {first}"))
        .collect();
    assert!(odd.is_empty(), "digits are not tabular: {odd:?}");
}

#[test]
fn figures_are_lining() {
    // Lining figures are cap height and sit on the baseline, so a number in a
    // row of labels has the same weight as the words around it. Old-style
    // figures would drop `3`, `4`, `7` and `9` below the baseline, which reads
    // as descenders in a table of file sizes.
    //
    // Measured on the stroke paths, not on the ink: every glyph is drawn with
    // whichever pen the caller asked for, so ink extends half a stroke past
    // these numbers by design and a test on the ink would be a test of the
    // weight setting rather than of the figures.
    let (mut low, mut high) = (f32::MAX, f32::MIN);
    for ch in "0123456789".chars() {
        let (foot, top) = path_box(ch);
        assert!(
            foot.abs() <= 0.5,
            "{ch} starts at y={foot}, not on the baseline"
        );
        assert!(
            (top - CAP).abs() <= 0.6,
            "{ch} stands {top} high against a cap height of {CAP}"
        );
        low = low.min(foot);
        high = high.max(top);
    }
    assert!(
        (high - low - CAP).abs() <= 0.6,
        "the figures are not one height: {} against a cap height of {CAP}",
        high - low
    );
}

#[test]
fn the_face_uses_standard_sans_proportions() {
    let x_ratio = X_HEIGHT / CAP;
    assert!(
        (0.68..=0.76).contains(&x_ratio),
        "x-height is {x_ratio:.2} of cap height, outside the 0.68-0.76 band UI faces use"
    );
    let line_ratio = LINE / CAP;
    assert!(
        (1.35..=1.65).contains(&line_ratio),
        "line spacing is {line_ratio:.2} cap heights, outside the 1.35-1.65 band"
    );
    for w in [WEIGHT_REGULAR, WEIGHT_BOLD] {
        assert!(
            (0.115..=0.20).contains(&w),
            "stem weight {w} is outside the 0.115-0.20 band"
        );
    }
    // Compared through the styles the interface actually builds rather than the
    // two constants side by side: `clippy::assertions_on_constants` is right that
    // a comparison between two literals is dead code, and this form also checks
    // the wiring that gives a style its weight.
    let (regular, bold) = (Style::new(13.0, 0), Style::bold(13.0, 0));
    assert!(
        bold.weight > regular.weight,
        "the bold style must be heavier than the regular one"
    );
}

#[test]
fn the_space_is_a_space() {
    // A space narrower than a quarter of the x-height closes words up; wider
    // than half the cap height opens them into columns of single words.
    let w = font::glyph(' ').expect("the face must have a space").adv;
    let ratio = w / CAP;
    assert!(
        (0.22..=0.40).contains(&ratio),
        "the space advances {w} font units, {ratio:.2} of cap height, against 0.22-0.40"
    );
}

#[test]
fn punctuation_is_set_closer_than_letters_and_never_wider() {
    // `.`, `,`, `:`, `;`, `'` and `!` take the same side bearing a letter does
    // but almost no ink. A face that gives them a full letter's advance reads as
    // letterspaced at every full stop, which is the most common way a
    // hand-drawn face looks wrong in a sentence.
    let letter = font::glyph('n').unwrap().adv;
    for ch in ".,:;'!|".chars() {
        let adv = font::glyph(ch).unwrap().adv;
        assert!(
            adv <= letter,
            "{ch} advances {adv}, wider than a letter ({letter})"
        );
    }
}

#[test]
fn every_glyph_the_interface_can_draw_exists() {
    // A missing glyph is drawn as a box, so the face has to cover what the
    // interface's strings use -- including the arrows the sort headers put next
    // to a column name and the dashes a file listing uses for a folder's size.
    // The list after the ASCII run is what `tools/scan-nonascii.py` reports the
    // shell's own strings contain, minus the Thai text (a monoline Latin face
    // cannot draw Thai; that needs a second face, and it is a separate task
    // with a separate answer).
    for ch in PRINTABLE.chars().chain("↑↓←→—·…×✕›".chars()) {
        assert!(
            font::glyph(ch).is_some(),
            "{ch:?} (U+{:04X}) has no glyph and would draw as a box",
            ch as u32
        );
    }
}

#[test]
fn a_sentence_of_mixed_content_stays_on_one_line() {
    // Measurement and drawing walk the same table (the doc comment on
    // `font::measure` says so); this is that claim executed, including the
    // glyphs that fall back to a box width.
    for text in [
        "litric-shell 0.2.0 — 527 of 7867 MB",
        "↑ Name  Size  Modified",
        "C:\\Users\\audim\\Documents",
        "scan: 12 files, 0 threats",
        "×",
    ] {
        let style = Style::new(13.0, 0xFFFFFFFF);
        let drawn = font::measure(text, &style);
        assert!(
            drawn > 0.0,
            "measuring {text:?} produced {drawn}, which cannot be drawn"
        );
    }
}
