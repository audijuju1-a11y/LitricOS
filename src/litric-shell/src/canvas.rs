//! A software raster canvas.
//!
//! Every pixel LitricOS puts on a screen goes through here. There is no GPU in
//! this path and, for a desktop that is mostly flat colour and one blur, there
//! does not need to be: a compositor that needs a GPU to draw a menu bar is a
//! compositor that cannot draw a menu bar when the GPU driver is the thing that
//! broke.
//!
//! Pixels are `0xAARRGGBB` in a `Vec<u32>`, host byte order. The conversion to
//! whatever the framebuffer actually wants happens once, at the edge, in
//! [`crate::fb`].
//!
//! # Anti-aliasing
//!
//! Shapes are drawn from signed distance functions rather than by filling
//! scanlines. A rounded rectangle knows how far every pixel is from its edge,
//! so the edge pixel gets partial coverage and the corner is smooth. This is
//! the difference between a desktop that looks drawn and one that looks
//! computed, and it costs one `sqrt` per pixel of the shape's bounding box.
//!
//! # Clipping and repainting part of a frame
//!
//! Every drawing loop takes its pixel range from [`Canvas::span`], which is
//! already cut down to the clip, so a clipped draw does not visit the pixels it
//! is not allowed to change. Operations that read *neighbouring* pixels (the
//! live glass blur, [`Canvas::brightest`]) read the unclipped range instead, so
//! their result does not depend on which part of the screen is being repainted.
//!
//! Glass normally reads no neighbours at all: [`Canvas::set_backdrop`] attaches
//! a copy of the wallpaper that is blurred and saturated once, and every glass
//! surface samples that. A surface drawn from it depends only on its own
//! position, which is what lets a damaged region be repainted on its own and
//! come out exactly as a full frame would.

use core::cell::Cell;
use core::f32::consts::PI;
use std::rc::Rc;

/// Straight-alpha composite of `src` over `dst`, both `0xAARRGGBB`.
///
/// `extra` scales the source alpha, which is how coverage from an anti-aliased
/// edge is applied without building a separate mask buffer.
#[must_use]
pub fn blend(dst: u32, src: u32, extra: f32) -> u32 {
    let sa = f32::from(((src >> 24) & 0xFF) as u8) / 255.0 * extra.clamp(0.0, 1.0);
    if sa <= 0.0 {
        return dst;
    }
    if sa >= 1.0 {
        return 0xFF00_0000 | (src & 0x00FF_FFFF);
    }
    let mix = |shift: u32| -> u32 {
        let s = f32::from(((src >> shift) & 0xFF) as u8);
        let d = f32::from(((dst >> shift) & 0xFF) as u8);
        let v = s * sa + d * (1.0 - sa);
        (v.round().clamp(0.0, 255.0) as u32) << shift
    };
    0xFF00_0000 | mix(16) | mix(8) | mix(0)
}

/// Pack straight 8-bit channels into `0xAARRGGBB`.
#[must_use]
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Opaque colour from `0xRRGGBB`.
#[must_use]
pub const fn rgb(hex: u32) -> u32 {
    0xFF00_0000 | (hex & 0x00FF_FFFF)
}

/// The same colour at a different alpha.
#[must_use]
pub fn with_alpha(color: u32, alpha: f32) -> u32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round().clamp(0.0, 255.0) as u32;
    (a << 24) | (color & 0x00FF_FFFF)
}

/// A rectangle in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect {
    /// Construct.
    #[must_use]
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    /// Right edge.
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    /// Bottom edge.
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    /// Centre.
    #[must_use]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
    /// Whether a point is inside. Half-open on the right and bottom, so two
    /// rectangles that share an edge do not both claim the same pixel column.
    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }

    /// The overlap with another rectangle, or `None` when they do not touch.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = self.right().min(other.right());
        let b = self.bottom().min(other.bottom());
        if r <= x || b <= y {
            None
        } else {
            Some(Self::new(x, y, r - x, b - y))
        }
    }

    /// This rectangle grown by `d` on every side.
    #[must_use]
    pub fn inflate(&self, d: f32) -> Self {
        Self::new(self.x - d, self.y - d, self.w + d * 2.0, self.h + d * 2.0)
    }
}

/// How much of the screen a frame repaints.
#[derive(Debug, Clone, PartialEq)]
pub enum Repaint {
    /// Every pixel.
    Full,
    /// Only these rectangles: whole pixels, inside the screen, not overlapping
    /// one another. Empty when all the damage fell outside the screen.
    Regions(Vec<Rect>),
}

/// Damage is grown by this much before it is merged: anti-aliased edges
/// reach half a pixel past a shape, and a stroke's bounds reach further.
const REGION_GROW: f32 = 2.0;

/// Damage rectangles closer together than this are repainted as one region.
/// Every region is a separate pass over the layout, so two cursor rectangles a
/// few pixels apart are cheaper as one.
const REGION_NEAR: i64 = 16;

/// The most regions one frame repaints. Past this, the pair whose union adds
/// the fewest pixels is merged.
const REGION_MAX: usize = 6;

/// A pixel-aligned box, `[x0, x1) x [y0, y1)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelBox {
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
}

impl PixelBox {
    fn area(&self) -> i64 {
        self.x1
            .saturating_sub(self.x0)
            .max(0)
            .saturating_mul(self.y1.saturating_sub(self.y0).max(0))
    }

    fn union(&self, o: &Self) -> Self {
        Self {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }

    /// Whether the gap between the two, on both axes, is at most `gap`.
    /// Overlapping boxes have no gap and are always near.
    fn near(&self, o: &Self, gap: i64) -> bool {
        self.x0 <= o.x1.saturating_add(gap)
            && o.x0 <= self.x1.saturating_add(gap)
            && self.y0 <= o.y1.saturating_add(gap)
            && o.y0 <= self.y1.saturating_add(gap)
    }

    fn rect(&self) -> Rect {
        #[allow(clippy::cast_precision_loss)]
        Rect::new(
            self.x0 as f32,
            self.y0 as f32,
            self.x1.saturating_sub(self.x0) as f32,
            self.y1.saturating_sub(self.y0) as f32,
        )
    }
}

impl Repaint {
    /// Turn a frame's damage into the regions to repaint.
    ///
    /// Each rectangle is grown by two pixels, rounded out to whole pixels and
    /// cut to the screen; rectangles that overlap or lie within sixteen pixels
    /// of each other are merged; more than six regions are merged down to six;
    /// and when the regions cover more than half the screen the answer is
    /// [`Repaint::Full`], because at that point the per-region passes cost more
    /// than they save. A rectangle with a non-finite edge contributes whatever
    /// part of it still lies on the screen after rounding, which for NaN is
    /// nothing.
    #[must_use]
    pub fn plan(damage: &[Rect], width: u32, height: u32) -> Self {
        let (sw, sh) = (i64::from(width), i64::from(height));
        let mut boxes: Vec<PixelBox> = damage
            .iter()
            .map(|r| {
                let g = r.inflate(REGION_GROW);
                PixelBox {
                    x0: (g.x.floor() as i64).max(0),
                    y0: (g.y.floor() as i64).max(0),
                    x1: (g.right().ceil() as i64).min(sw),
                    y1: (g.bottom().ceil() as i64).min(sh),
                }
            })
            .filter(|b| b.x1 > b.x0 && b.y1 > b.y0)
            .collect();

        loop {
            while let Some((i, j)) = near_pair(&boxes) {
                merge(&mut boxes, i, j);
            }
            if boxes.len() <= REGION_MAX {
                break;
            }
            match cheapest_pair(&boxes) {
                Some((i, j)) => merge(&mut boxes, i, j),
                None => break,
            }
        }

        let covered = boxes
            .iter()
            .fold(0i64, |acc, b| acc.saturating_add(b.area()));
        if covered.saturating_mul(2) > sw.saturating_mul(sh) {
            return Self::Full;
        }
        Self::Regions(boxes.iter().map(PixelBox::rect).collect())
    }

    /// The rectangles this plan repaints: the whole screen for
    /// [`Repaint::Full`].
    #[must_use]
    pub fn rects(&self, width: u32, height: u32) -> Vec<Rect> {
        match self {
            #[allow(clippy::cast_precision_loss)]
            Self::Full => vec![Rect::new(0.0, 0.0, width as f32, height as f32)],
            Self::Regions(r) => r.clone(),
        }
    }
}

/// The first pair of boxes that overlap or nearly touch.
fn near_pair(boxes: &[PixelBox]) -> Option<(usize, usize)> {
    for (i, a) in boxes.iter().enumerate() {
        for (j, b) in boxes.iter().enumerate().skip(i.saturating_add(1)) {
            if a.near(b, REGION_NEAR) {
                return Some((i, j));
            }
        }
    }
    None
}

/// The pair whose union adds the fewest pixels to the repaint.
fn cheapest_pair(boxes: &[PixelBox]) -> Option<(usize, usize)> {
    let mut best: Option<(i64, usize, usize)> = None;
    for (i, a) in boxes.iter().enumerate() {
        for (j, b) in boxes.iter().enumerate().skip(i.saturating_add(1)) {
            let cost = a
                .union(b)
                .area()
                .saturating_sub(a.area())
                .saturating_sub(b.area());
            if best.is_none_or(|(c, _, _)| cost < c) {
                best = Some((cost, i, j));
            }
        }
    }
    best.map(|(_, i, j)| (i, j))
}

/// Replace box `i` with the union of `i` and `j`, and remove `j`. `i < j`.
fn merge(boxes: &mut Vec<PixelBox>, i: usize, j: usize) {
    if j >= boxes.len() || i >= j {
        return;
    }
    let b = boxes.swap_remove(j);
    if let Some(a) = boxes.get_mut(i) {
        *a = a.union(&b);
    }
}

/// Signed distance from a point to a rounded rectangle. Negative inside.
#[must_use]
fn sd_round_rect(px: f32, py: f32, r: &Rect, radius: f32) -> f32 {
    let (cx, cy) = r.center();
    let hw = r.w / 2.0;
    let hh = r.h / 2.0;
    let rad = radius.min(hw).min(hh).max(0.0);
    let qx = (px - cx).abs() - (hw - rad);
    let qy = (py - cy).abs() - (hh - rad);
    let ax = qx.max(0.0);
    let ay = qy.max(0.0);
    (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad
}

/// Coverage from a signed distance: 1 well inside, 0 well outside, a smooth
/// ramp across one pixel at the boundary.
#[must_use]
fn coverage(d: f32) -> f32 {
    (0.5 - d).clamp(0.0, 1.0)
}

/// The pixels whose centres lie inside a clip rectangle, as a half-open range
/// `(x0, y0, x1, y1)`.
///
/// The same rule as the per-pixel test in [`Canvas::put`]: pixel `x` is inside
/// when `x + 0.5` lies in `[c.x, c.right())`, which for an integer `x` is
/// `x >= ceil(c.x - 0.5)` and `x < ceil(c.right() - 0.5)`.
fn clip_pixels(c: &Rect) -> (i64, i64, i64, i64) {
    (
        (c.x - 0.5).ceil() as i64,
        (c.y - 0.5).ceil() as i64,
        (c.right() - 0.5).ceil() as i64,
        (c.bottom() - 0.5).ceil() as i64,
    )
}

/// A canvas of `0xAARRGGBB` pixels.
#[derive(Debug, Clone)]
pub struct Canvas {
    width: u32,
    height: u32,
    px: Vec<u32>,
    /// Nothing is drawn outside this. `None` means the whole canvas.
    clip: Option<Rect>,
    /// What glass samples instead of the live pixels, when attached.
    backdrop: Option<Rc<Canvas>>,
    /// Pixel-loop iterations handed out by [`Canvas::span`] and
    /// [`Canvas::span_all`], for measuring how much a frame actually visits.
    visited: Cell<u64>,
    /// The same total, split by which kind of loop was given the pixels. A
    /// frame's total is not actionable on its own: 755 391 pixels mean one
    /// thing when a fill asked for them and another when a blur did, and the
    /// difference is the whole of BUG-0015.
    visited_by: Cell<[u64; Site::COUNT]>,
    /// Every text run drawn since recording was switched on, for the layout
    /// audit. `None` normally: a frame nobody is auditing pays one flag check
    /// per run and nothing else.
    runs: Option<Vec<TextRun>>,
    /// The surface currently being drawn, for [`TextRun::surface`]. Zero is the
    /// desktop chrome; [`Canvas::set_surface`] gives each window its own.
    surface: u32,
    /// Which kind of loop the next drawing call is. [`Site::Draw`] except while
    /// a shadow's rings or a run of glyphs is being drawn, so the frame's pixel
    /// bill says which of them cost what.
    site: Cell<Site>,
}

/// One run of text a frame drew, in the coordinates it was drawn at.
///
/// Recorded at the draw call rather than measured afterwards, because two
/// strings drawn over each other leave pixels that cannot be told apart once
/// they are blended. The only place that knows a collision happened is the call
/// that caused it, so that is where the record is made.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    /// The string as drawn.
    pub text: String,
    /// Left edge of the run.
    pub x: f32,
    /// Where the pen finished: the right edge of the last glyph.
    pub end_x: f32,
    /// The baseline the run was drawn on.
    pub baseline: f32,
    /// Cap height in pixels, which is what `Style::size` means here.
    pub size: f32,
    /// Which surface the run belongs to: 0 for the desktop chrome (menubar,
    /// dock, overlays), and one number per window otherwise.
    ///
    /// Two windows may overlap on screen, which puts their text on top of each
    /// other legitimately -- the upper window's background covers the lower
    /// one's letters. Without this, every frame of a stacked desktop reports a
    /// screenful of collisions that no user can see, and the real ones are lost
    /// in the noise.
    pub surface: u32,
}

impl TextRun {
    /// The band the glyphs occupy: cap height above the baseline, descenders
    /// below it. Ascenders reach the cap in this typeface; the 0.35 em below is
    /// the depth of `g`, `p`, `q` and `y`.
    #[must_use]
    pub fn band(&self) -> (f32, f32) {
        (
            self.baseline - self.size * 1.05,
            self.baseline + self.size * 0.35,
        )
    }

    /// The horizontal extent, left to right.
    #[must_use]
    pub fn span(&self) -> (f32, f32) {
        (self.x.min(self.end_x), self.x.max(self.end_x))
    }

    /// Whether this run put any ink on the canvas at all.
    #[must_use]
    pub fn has_ink(&self) -> bool {
        self.end_x > self.x + 0.01 && self.text.chars().any(|c| !c.is_whitespace())
    }
}

/// How far apart [`Canvas::brightest`] samples. A wallpaper is smooth; every
/// fourth pixel finds the same bright corner as every pixel would and costs a
/// sixteenth as much.
pub const SAMPLE_STEP: i64 = 4;

/// The kind of pixel loop a range was handed to, for
/// [`Canvas::pixels_visited_by_site`].
///
/// This is the measurement that makes a partial-frame figure actionable. A
/// damage-clipped frame reports one number, and 755 391 of them look alarming
/// until the split says how many a *fill* was given: a fill limited by the clip
/// cannot exceed the damaged area, so anything above that came from a loop that
/// ignored the clip, and only neighbour-reading loops may do that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Site {
    /// A drawing loop cut to the clip: fills, strokes, gradients, blits.
    Draw,
    /// The glass blur behind a panel.
    Glass,
    /// An explicit region blur.
    Blur,
    /// A mean over a rectangle.
    Average,
    /// A sampled search for the worst pixel over a rectangle.
    Brightest,
    /// A soft drop shadow's rings.
    Shadow,
    /// Glyph strokes.
    Text,
}

impl Site {
    /// How many kinds there are. The array in [`Canvas`] is sized by this.
    pub const COUNT: usize = 7;

    /// Every kind, in a fixed order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::Draw,
        Self::Glass,
        Self::Blur,
        Self::Average,
        Self::Brightest,
        Self::Shadow,
        Self::Text,
    ];

    /// Position in [`Site::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Draw => 0,
            Self::Glass => 1,
            Self::Blur => 2,
            Self::Average => 3,
            Self::Brightest => 4,
            Self::Shadow => 5,
            Self::Text => 6,
        }
    }

    /// The name this kind reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draw => "draw",
            Self::Glass => "glass",
            Self::Blur => "blur",
            Self::Average => "average",
            Self::Brightest => "brightest",
            Self::Shadow => "shadow",
            Self::Text => "text",
        }
    }
}

impl Canvas {
    /// A canvas filled with one colour.
    ///
    /// # Panics
    /// Never: an over-large size saturates instead of overflowing, and the
    /// allocation is the only thing that can fail, which aborts rather than
    /// returning here.
    #[must_use]
    pub fn new(width: u32, height: u32, fill: u32) -> Self {
        let n = (width as usize).saturating_mul(height as usize);
        Self {
            width,
            height,
            px: vec![fill; n],
            clip: None,
            backdrop: None,
            visited: Cell::new(0),
            visited_by: Cell::new([0; Site::COUNT]),
            runs: None,
            surface: 0,
            site: Cell::new(Site::Draw),
        }
    }

    /// How many pixel-loop iterations the drawing calls have made since the
    /// last [`Canvas::reset_pixels_visited`]: the area of every pixel range a
    /// drawing, blending or sampling loop was given, whether or not it then
    /// changed the pixel.
    #[must_use]
    pub fn pixels_visited(&self) -> u64 {
        self.visited.get()
    }

    /// Start counting [`Canvas::pixels_visited`] from zero.
    pub fn reset_pixels_visited(&self) {
        self.visited.set(0);
        self.visited_by.set([0; Site::COUNT]);
    }

    /// [`Canvas::pixels_visited`] split by the kind of loop that asked for the
    /// pixels, largest first.
    #[must_use]
    pub fn pixels_visited_by_site(&self) -> Vec<(Site, u64)> {
        let counts = self.visited_by.get();
        let mut out: Vec<(Site, u64)> = Site::ALL
            .into_iter()
            .map(|s| (s, counts.get(s.index()).copied().unwrap_or(0)))
            .filter(|(_, n)| *n > 0)
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.1));
        out
    }

    /// Mark everything drawn from now on as belonging to `surface`, and return
    /// the previous one so the caller can restore it.
    ///
    /// Zero is the desktop chrome. A window passes its own id, so the layout
    /// audit can tell two windows' overlapping text (expected, hidden by the
    /// upper window) from two labels colliding inside one window (a defect).
    pub fn set_surface(&mut self, surface: u32) -> u32 {
        core::mem::replace(&mut self.surface, surface)
    }

    /// The surface being drawn.
    #[must_use]
    pub const fn surface(&self) -> u32 {
        self.surface
    }

    /// Switch the text-run recorder on (`true`) or off (`false`).
    ///
    /// Off by default and off in production: switching it off drops the buffer
    /// rather than clearing it, so the cost of the audit is one comparison per
    /// text run and no allocation at all.
    pub fn record_text(&mut self, on: bool) {
        self.runs = if on { Some(Vec::new()) } else { None };
    }

    /// The runs recorded since recording was switched on, oldest first.
    #[must_use]
    pub fn text_runs(&self) -> &[TextRun] {
        self.runs.as_deref().unwrap_or(&[])
    }

    /// Whether a recorder is attached.
    #[must_use]
    pub const fn is_recording_text(&self) -> bool {
        self.runs.is_some()
    }

    /// Record one run. Called by [`crate::font::draw`].
    pub fn record_run(&mut self, run: TextRun) {
        if let Some(runs) = self.runs.as_mut() {
            runs.push(run);
        }
    }

    /// Restrict drawing to a rectangle. Returns the previous clip so the caller
    /// can restore it.
    ///
    /// Without this a window's contents run past its own frame and off the
    /// bottom of the panel, which is exactly what the first interactive build
    /// of this desktop did.
    pub fn set_clip(&mut self, r: Option<Rect>) -> Option<Rect> {
        core::mem::replace(&mut self.clip, r)
    }

    /// The clip in force.
    #[must_use]
    pub const fn clip(&self) -> Option<Rect> {
        self.clip
    }

    /// Intersect the current clip with `r` and return the previous one.
    pub fn push_clip(&mut self, r: Rect) -> Option<Rect> {
        let next = match self.clip {
            Some(c) => c.intersection(&r).unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0)),
            None => r,
        };
        self.set_clip(Some(next))
    }

    /// Attach the backdrop glass samples: normally
    /// [`Canvas::glass_source`] of the wallpaper under this canvas.
    ///
    /// While one is attached, [`Canvas::glass_backdrop`] copies from it
    /// instead of blurring the live pixels, and [`Canvas::backdrop_brightest`]
    /// searches it. Returns `false`, and leaves the current backdrop in place,
    /// when the backdrop is a different size from this canvas: sampling it at
    /// the wrong coordinates would put the wrong colours behind every panel.
    /// `None` detaches and always succeeds.
    pub fn set_backdrop(&mut self, backdrop: Option<Rc<Self>>) -> bool {
        match backdrop {
            Some(b) if b.width != self.width || b.height != self.height => false,
            other => {
                self.backdrop = other;
                true
            }
        }
    }

    /// The attached backdrop, if any.
    #[must_use]
    pub fn backdrop(&self) -> Option<&Self> {
        self.backdrop.as_deref()
    }

    /// Width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }
    /// Height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }
    /// The raw pixels, row by row, `width` to a row.
    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        &self.px
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        if x < 0 || y < 0 || x >= i64::from(self.width) || y >= i64::from(self.height) {
            return None;
        }
        let i = (y as usize).checked_mul(self.width as usize)?;
        i.checked_add(x as usize)
    }

    /// The pixel at `(x, y)`, or `None` when outside.
    #[must_use]
    pub fn get(&self, x: i64, y: i64) -> Option<u32> {
        self.index(x, y).and_then(|i| self.px.get(i).copied())
    }

    /// Whether a pixel is inside the clip.
    fn clipped_in(&self, x: i64, y: i64) -> bool {
        self.clip.is_none_or(|c| {
            #[allow(clippy::cast_precision_loss)]
            c.contains(x as f32 + 0.5, y as f32 + 0.5)
        })
    }

    /// Write a pixel, ignoring writes outside the canvas or the clip.
    pub fn put(&mut self, x: i64, y: i64, color: u32) {
        if self.clipped_in(x, y) {
            self.put_in(x, y, color);
        }
    }

    /// Composite `color` over the pixel at `(x, y)` with extra coverage.
    pub fn blend_px(&mut self, x: i64, y: i64, color: u32, cov: f32) {
        if self.clipped_in(x, y) {
            self.blend_in(x, y, color, cov);
        }
    }

    /// Write a pixel that [`Canvas::span`] already placed inside the clip.
    fn put_in(&mut self, x: i64, y: i64, color: u32) {
        if let Some(p) = self.index(x, y).and_then(|i| self.px.get_mut(i)) {
            *p = color;
        }
    }

    /// Composite at a pixel that [`Canvas::span`] already placed inside the
    /// clip.
    fn blend_in(&mut self, x: i64, y: i64, color: u32, cov: f32) {
        if cov <= 0.0 {
            return;
        }
        if let Some(p) = self.index(x, y).and_then(|i| self.px.get_mut(i)) {
            *p = blend(*p, color, cov);
        }
    }

    /// Fill the whole canvas, ignoring the clip.
    ///
    /// The one drawing call that does. Clearing is a frame-level operation and
    /// a clip is a within-frame one; honouring it here would make "start a new
    /// frame" depend on whatever the last panel happened to leave set.
    pub fn clear(&mut self, color: u32) {
        self.px.fill(color);
    }

    /// Copy a region from another canvas, inside the clip.
    ///
    /// This is how damage tracking erases: the wallpaper is rendered once into
    /// its own canvas, and repainting a region starts by restoring it here.
    /// Without it, a window that moves leaves a copy of itself behind. Between
    /// canvases of the same size each row is one slice copy.
    pub fn blit_from(&mut self, src: &Self, r: Rect) {
        let (x0, y0, x1, y1) = self.span(&r);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        if src.width == self.width && src.height == self.height {
            let w = self.width as usize;
            // `span` never returns a negative start.
            let (a, b) = (x0 as usize, x1 as usize);
            for y in y0..y1 {
                let row = (y as usize).saturating_mul(w);
                let range = row.saturating_add(a)..row.saturating_add(b);
                if let (Some(dst), Some(from)) = (self.px.get_mut(range.clone()), src.px.get(range))
                {
                    dst.copy_from_slice(from);
                }
            }
            return;
        }
        for y in y0..y1 {
            for x in x0..x1 {
                if let Some(p) = src.get(x, y) {
                    self.put_in(x, y, p);
                }
            }
        }
    }

    /// The integer pixel range a float rectangle touches, limited to the
    /// canvas.
    fn bounds_of(&self, r: &Rect) -> (i64, i64, i64, i64) {
        let x0 = (r.x.floor() as i64).max(0);
        let y0 = (r.y.floor() as i64).max(0);
        let x1 = (r.right().ceil() as i64).min(i64::from(self.width));
        let y1 = (r.bottom().ceil() as i64).min(i64::from(self.height));
        (x0, y0, x1, y1)
    }

    /// Record `n` pixel-loop iterations of kind `site`.
    fn count_px(&self, site: Site, n: u64) {
        self.visited.set(self.visited.get().saturating_add(n));
        let i = site.index();
        let mut by = self.visited_by.get();
        if let Some(c) = by.get_mut(i) {
            *c = c.saturating_add(n);
        }
        self.visited_by.set(by);
    }

    fn count(&self, site: Site, (x0, y0, x1, y1): (i64, i64, i64, i64)) {
        let w = u64::try_from(x1.saturating_sub(x0)).unwrap_or(0);
        let h = u64::try_from(y1.saturating_sub(y0)).unwrap_or(0);
        self.count_px(site, w.saturating_mul(h));
    }

    /// Mark everything drawn from now on as belonging to `site`, and return the
    /// previous one so the caller can restore it.
    pub fn set_site(&self, site: Site) -> Site {
        self.site.replace(site)
    }

    /// The pixel range a drawing loop may visit: the rectangle, cut to the
    /// canvas **and to the clip**. For loops where each pixel's new value
    /// depends only on that pixel.
    fn span(&self, r: &Rect) -> (i64, i64, i64, i64) {
        let (mut x0, mut y0, mut x1, mut y1) = self.bounds_of(r);
        if let Some(c) = self.clip {
            let (cx0, cy0, cx1, cy1) = clip_pixels(&c);
            x0 = x0.max(cx0);
            y0 = y0.max(cy0);
            x1 = x1.min(cx1);
            y1 = y1.min(cy1);
        }
        let s = (x0, y0, x1.max(x0), y1.max(y0));
        self.count(self.site.get(), s);
        s
    }

    /// The pixel range a reading loop visits: the rectangle cut to the canvas
    /// only. For anything that reads neighbours, whose answer must not change
    /// with the clip.
    fn span_all(&self, r: &Rect, site: Site) -> (i64, i64, i64, i64) {
        let (x0, y0, x1, y1) = self.bounds_of(r);
        let s = (x0, y0, x1.max(x0), y1.max(y0));
        self.count(site, s);
        s
    }

    /// The pixel range a reading loop visits when it walks the rectangle on a
    /// grid of `step` pixels. Counted as the samples it really takes: a search
    /// that reads every fourth pixel on both axes visits a sixteenth of the
    /// rectangle, and reporting the rectangle would overstate the frame by
    /// exactly the factor that makes the number unusable for finding work.
    fn span_sampled(&self, r: &Rect, step: i64, site: Site) -> (i64, i64, i64, i64) {
        let (x0, y0, x1, y1) = self.bounds_of(r);
        let s = (x0, y0, x1.max(x0), y1.max(y0));
        let step = step.max(1);
        let nx = u64::try_from(
            x1.saturating_sub(x0)
                .saturating_add(step.saturating_sub(1))
                .div_euclid(step),
        )
        .unwrap_or(0);
        let ny = u64::try_from(
            y1.saturating_sub(y0)
                .saturating_add(step.saturating_sub(1))
                .div_euclid(step),
        )
        .unwrap_or(0);
        self.count_px(site, nx.saturating_mul(ny));
        s
    }

    /// A filled, anti-aliased rounded rectangle. `radius` of 0 is a plain rect.
    pub fn fill_round_rect(&mut self, r: Rect, radius: f32, color: u32) {
        let (x0, y0, x1, y1) = self.span(&r.inflate(1.5));
        for y in y0..y1 {
            for x in x0..x1 {
                let d = sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius);
                self.blend_in(x, y, color, coverage(d));
            }
        }
    }

    /// A one-pixel-wide (or thicker) rounded outline.
    pub fn stroke_round_rect(&mut self, r: Rect, radius: f32, width: f32, color: u32) {
        let half = width / 2.0;
        let (x0, y0, x1, y1) = self.span(&r.inflate(width + 1.5));
        for y in y0..y1 {
            for x in x0..x1 {
                let d = sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius).abs() - half;
                self.blend_in(x, y, color, coverage(d));
            }
        }
    }

    /// A filled circle.
    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: u32) {
        let r = Rect::new(cx - radius, cy - radius, radius * 2.0, radius * 2.0);
        self.fill_round_rect(r, radius, color);
    }

    /// An anti-aliased line of the given width, with round caps.
    pub fn stroke_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, color: u32) {
        let half = width / 2.0;
        let minx = x0.min(x1) - half - 1.5;
        let miny = y0.min(y1) - half - 1.5;
        let maxx = x0.max(x1) + half + 1.5;
        let maxy = y0.max(y1) + half + 1.5;
        let bounds = Rect::new(minx, miny, maxx - minx, maxy - miny);
        let (bx0, by0, bx1, by1) = self.span(&bounds);
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len2 = dx * dx + dy * dy;
        for y in by0..by1 {
            for x in bx0..bx1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let t = if len2 <= f32::EPSILON {
                    0.0
                } else {
                    (((px - x0) * dx + (py - y0) * dy) / len2).clamp(0.0, 1.0)
                };
                let qx = px - (x0 + dx * t);
                let qy = py - (y0 + dy * t);
                let d = (qx * qx + qy * qy).sqrt() - half;
                self.blend_in(x, y, color, coverage(d));
            }
        }
    }

    /// A vertical linear gradient across `r`.
    pub fn fill_vertical_gradient(&mut self, r: Rect, top: u32, bottom: u32) {
        let (x0, y0, x1, y1) = self.span(&r);
        for y in y0..y1 {
            let t = if r.h.abs() <= f32::EPSILON {
                0.0
            } else {
                ((y as f32 + 0.5 - r.y) / r.h).clamp(0.0, 1.0)
            };
            let c = lerp_color(top, bottom, t);
            for x in x0..x1 {
                self.put_in(x, y, c);
            }
        }
    }

    /// A radial wash of `color` centred at `(cx, cy)`, fading to nothing at
    /// `radius`. Used for the wallpaper.
    pub fn radial_wash(&mut self, cx: f32, cy: f32, radius: f32, color: u32, peak: f32) {
        if radius <= 0.0 {
            return;
        }
        let bounds = Rect::new(cx - radius, cy - radius, radius * 2.0, radius * 2.0);
        let (x0, y0, x1, y1) = self.span(&bounds);
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let t = ((dx * dx + dy * dy).sqrt() / radius).clamp(0.0, 1.0);
                // Smoothstep falloff: a linear one leaves a visible ring where
                // the gradient stops, which reads as a rendering bug.
                let f = 1.0 - t * t * (3.0 - 2.0 * t);
                self.blend_in(x, y, color, f * peak);
            }
        }
    }

    /// Average colour of a rectangle, for deciding what a panel sits on. Reads
    /// the whole rectangle whatever the clip.
    #[must_use]
    pub fn average(&self, r: Rect) -> u32 {
        let (x0, y0, x1, y1) = self.span_all(&r, Site::Average);
        let mut n = 0u64;
        let (mut sr, mut sg, mut sb) = (0u64, 0u64, 0u64);
        for y in y0..y1 {
            for x in x0..x1 {
                if let Some(p) = self.get(x, y) {
                    sr = sr.saturating_add(u64::from((p >> 16) & 0xFF));
                    sg = sg.saturating_add(u64::from((p >> 8) & 0xFF));
                    sb = sb.saturating_add(u64::from(p & 0xFF));
                    n = n.saturating_add(1);
                }
            }
        }
        if n == 0 {
            return 0xFF00_0000;
        }
        let d = |s: u64| -> u8 { s.checked_div(n).unwrap_or(0).min(255) as u8 };
        rgba(d(sr), d(sg), d(sb), 255)
    }

    /// The brightest pixel in a rectangle, by WCAG relative luminance. Reads
    /// the whole rectangle whatever the clip.
    ///
    /// The glass solver needs the *worst* backdrop under a panel, not the mean:
    /// text has to stay readable over the bright corner of a wallpaper, and an
    /// average would quietly excuse exactly that pixel.
    #[must_use]
    pub fn brightest(&self, r: Rect) -> u32 {
        let s = self.span_sampled(&r, SAMPLE_STEP, Site::Brightest);
        brightest_sampled(self, s)
    }

    /// [`Canvas::brightest`] of what glass at `r` samples: the attached
    /// backdrop when there is one, otherwise the live pixels.
    #[must_use]
    pub fn backdrop_brightest(&self, r: Rect) -> u32 {
        let s = self.span_sampled(&r, SAMPLE_STEP, Site::Brightest);
        brightest_sampled(self.backdrop.as_deref().unwrap_or(self), s)
    }

    /// Box-blur a region in place, `passes` times.
    ///
    /// Three passes of a box blur approximate a Gaussian closely enough that no
    /// one can tell, at a fraction of the cost. Reads the whole region whatever
    /// the clip and writes back only inside it.
    ///
    /// A region the clip does not touch is skipped entirely, and that changes
    /// no pixel: the blur exists to feed writes, every write goes through
    /// [`Canvas::put`], and `put` refuses anything outside the clip. What the
    /// skip removes is the read of the region and every blur pass over it --
    /// which was, measured on a three-window desktop, **98x the pixels the
    /// damaged region actually needed**: a 7 680-pixel damage area handed the
    /// blur loops 755 391 pixels of untouched window behind it. Three windows
    /// at 1280x800 re-blurred their whole backdrops on every frame that damaged
    /// a corner of the dock.
    pub fn blur_region(&mut self, r: Rect, radius: u32, passes: u32) {
        if radius == 0 {
            return;
        }
        if let Some(c) = self.clip {
            if c.intersection(&r).is_none() {
                return;
            }
        }
        let (x0, y0, x1, y1) = self.span_all(&r, Site::Blur);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let w = x1.saturating_sub(x0) as usize;
        let h = y1.saturating_sub(y0) as usize;
        let mut buf: Vec<u32> = Vec::with_capacity(w.saturating_mul(h));
        for y in y0..y1 {
            for x in x0..x1 {
                buf.push(self.get(x, y).unwrap_or(0xFF00_0000));
            }
        }
        let mut tmp = buf.clone();
        for _ in 0..passes {
            box_blur_pass(&buf, &mut tmp, w, h, radius as usize, true);
            box_blur_pass(&tmp, &mut buf, w, h, radius as usize, false);
        }
        let mut i = 0usize;
        for y in y0..y1 {
            for x in x0..x1 {
                if let Some(v) = buf.get(i) {
                    self.put(x, y, *v);
                }
                i = i.saturating_add(1);
            }
        }
    }

    /// What glass samples: this canvas blurred with two box-blur passes of
    /// `blur` and pushed away from grey by `saturation`, over the whole canvas.
    ///
    /// The same operations [`Canvas::glass_backdrop`] applies to the live
    /// pixels around one panel, done once. Near a panel's edge the results
    /// differ slightly, because here the blur has the real neighbours beyond
    /// the panel's working region to average.
    #[must_use]
    pub fn glass_source(&self, blur: u32, saturation: f32) -> Self {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut buf = self.px.clone();
        if blur > 0 {
            let mut tmp = buf.clone();
            for _ in 0..2 {
                box_blur_pass(&buf, &mut tmp, w, h, blur as usize, true);
                box_blur_pass(&tmp, &mut buf, w, h, blur as usize, false);
            }
        }
        if (saturation - 1.0).abs() > f32::EPSILON {
            for p in &mut buf {
                *p = saturate_px(*p, saturation);
            }
        }
        Self {
            width: self.width,
            height: self.height,
            px: buf,
            clip: None,
            backdrop: None,
            visited: Cell::new(0),
            visited_by: Cell::new([0; Site::COUNT]),
            runs: None,
            surface: 0,
            site: Cell::new(Site::Draw),
        }
    }

    /// Prepare the backdrop for a glass panel, written **only inside the
    /// rounded shape** with anti-aliased coverage at its edge.
    ///
    /// With a backdrop attached ([`Canvas::set_backdrop`]) the pixels come
    /// from it and `blur` and `saturation` are not used: they were applied when
    /// the backdrop was made. The panel then looks the same whatever was drawn
    /// under it and whatever part of the screen is being repainted.
    ///
    /// Without one, the live pixels around the panel are blurred and boosted
    /// here. Doing that over a plain rectangle is visibly wrong: the
    /// rectangle's edges show up as a box of slightly different colour around
    /// every panel, because a saturation boost has a hard edge even when the
    /// blur does not. Masking the write-back by the same signed distance
    /// function that draws the panel is what makes the material end exactly
    /// where the panel ends.
    pub fn glass_backdrop(&mut self, r: Rect, radius: f32, blur: u32, saturation: f32) {
        if let Some(source) = self.backdrop.clone() {
            let (x0, y0, x1, y1) = self.span(&r.inflate(1.5));
            for y in y0..y1 {
                for x in x0..x1 {
                    let cov = coverage(sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius));
                    if cov > 0.0 {
                        if let Some(v) = source.get(x, y) {
                            self.blend_in(x, y, v, cov);
                        }
                    }
                }
            }
            return;
        }

        // The working region is larger than what is painted, because the blur
        // has to average real neighbours instead of running off the edge of its
        // own buffer and darkening the border -- but only larger by the reach of
        // the kernel, not by the whole panel. Two box passes of `blur` reach
        // `2 * blur` pixels in each direction, so a painted pixel's value
        // depends on source pixels no further away than that, and a working
        // region of the painted area grown by the reach contains every source
        // pixel any painted pixel can see. Whatever lies outside it would be
        // read, blurred twice in each direction and then discarded by the
        // coverage mask -- which is exactly the waste BUG-0015 measured at 98x:
        // on a three-window desktop a 7 680-pixel damage region handed the blur
        // loops 755 391 pixels, every one of them behind a panel the frame was
        // not repainting. Taking the region from the clip instead of the panel
        // changes no pixel the mask lets through, so a partial frame is
        // byte-identical to the full frame it was cut from.
        let reach = (blur as f32).mul_add(2.0, 2.0);
        let painted = match self.clip {
            Some(c) => match c.intersection(&r) {
                // The clip does not touch the panel: there is no pixel this call
                // could write, so there is none it should read. Inflating an
                // empty intersection would only invent a region at the origin.
                Some(p) if p.w > 0.0 && p.h > 0.0 => p,
                _ => return,
            },
            None => r,
        };
        let region = painted.inflate(reach + radius);
        let (x0, y0, x1, y1) = self.span_all(&region, Site::Glass);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let w = x1.saturating_sub(x0) as usize;
        let h = y1.saturating_sub(y0) as usize;

        let mut buf: Vec<u32> = Vec::with_capacity(w.saturating_mul(h));
        for y in y0..y1 {
            for x in x0..x1 {
                buf.push(self.get(x, y).unwrap_or(0xFF00_0000));
            }
        }

        if blur > 0 {
            let mut tmp = buf.clone();
            for _ in 0..2 {
                box_blur_pass(&buf, &mut tmp, w, h, blur as usize, true);
                box_blur_pass(&tmp, &mut buf, w, h, blur as usize, false);
            }
        }
        if (saturation - 1.0).abs() > f32::EPSILON {
            for p in &mut buf {
                *p = saturate_px(*p, saturation);
            }
        }

        let mut i = 0usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let cov = coverage(sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius));
                if cov > 0.0 {
                    if let Some(v) = buf.get(i) {
                        self.blend_px(x, y, *v, cov);
                    }
                }
                i = i.saturating_add(1);
            }
        }
    }

    /// A soft drop shadow under a rounded rectangle.
    ///
    /// Built from concentric rounded rectangles rather than by blurring a mask.
    /// Twelve steps is enough that the banding is below one 8-bit level, and it
    /// avoids allocating a second full-size buffer for something that is only
    /// ever a dark halo. The outermost ring is grown by 11/12 of `spread` and
    /// moved down by `dy`.
    pub fn soft_shadow(&mut self, r: Rect, radius: f32, spread: f32, dy: f32, alpha: f32) {
        const STEPS: u32 = 12;
        let prev = self.set_site(Site::Shadow);
        for i in (0..STEPS).rev() {
            let t = f32::from(i as u16) / f32::from(STEPS as u16);
            let grow = spread * t;
            // Squared falloff: a linear one leaves a visible outer edge where
            // the last ring stops, which reads as a rectangle rather than a
            // shadow.
            let a = alpha * (1.0 - t) * (1.0 - t) / f32::from(STEPS as u16) * 3.0;
            self.fill_round_rect(
                Rect::new(
                    r.x - grow,
                    r.y + dy - grow,
                    r.w + grow * 2.0,
                    r.h + grow * 2.0,
                ),
                radius + grow,
                with_alpha(rgba(0, 0, 0, 255), a),
            );
        }
        self.set_site(prev);
    }

    /// Push saturation of a region away from grey by `amount` (1.0 = unchanged).
    ///
    /// Part of the glass material: colour from behind the panel should bleed
    /// through more strongly than a plain blur would leave it, which is what
    /// makes the surface look like it is refracting something.
    pub fn saturate_region(&mut self, r: Rect, amount: f32) {
        let (x0, y0, x1, y1) = self.span(&r);
        for y in y0..y1 {
            for x in x0..x1 {
                let Some(p) = self.get(x, y) else { continue };
                self.put_in(x, y, saturate_px(p, amount));
            }
        }
    }
}

/// The brightest pixel of `source` over a pixel range, sampling every fourth
/// pixel on both axes starting at the range's corner.
fn brightest_sampled(source: &Canvas, (x0, y0, x1, y1): (i64, i64, i64, i64)) -> u32 {
    let mut best = 0xFF00_0000u32;
    let mut best_l = -1.0f32;
    // Sampling every fourth pixel: a wallpaper is smooth, and this turns a
    // full-screen search from milliseconds into microseconds.
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            if let Some(p) = source.get(x, y) {
                let l = luminance(p);
                if l > best_l {
                    best_l = l;
                    best = p;
                }
            }
            x = x.saturating_add(4);
        }
        y = y.saturating_add(4);
    }
    best
}

/// One pixel pushed away from grey. `amount` of 1.0 leaves it unchanged.
#[must_use]
fn saturate_px(p: u32, amount: f32) -> u32 {
    let rf = f32::from(((p >> 16) & 0xFF) as u8);
    let gf = f32::from(((p >> 8) & 0xFF) as u8);
    let bf = f32::from((p & 0xFF) as u8);
    let grey = 0.2126 * rf + 0.7152 * gf + 0.0722 * bf;
    let f = |c: f32| -> u8 { (grey + (c - grey) * amount).clamp(0.0, 255.0) as u8 };
    rgba(f(rf), f(gf), f(bf), 255)
}

/// One direction of a box blur: each output pixel is the truncated mean of the
/// pixels within `radius` of it along the row (or column), counting only the
/// ones that exist.
///
/// The window slides: moving one pixel along adds the sample entering the
/// window and subtracts the one leaving it, so the cost per pixel does not
/// grow with the radius. Sums are `u64`, which holds any window a `u32`-sized
/// canvas can have without saturating.
fn box_blur_pass(
    src: &[u32],
    dst: &mut [u32],
    w: usize,
    h: usize,
    radius: usize,
    horizontal: bool,
) {
    let (outer, inner) = if horizontal { (h, w) } else { (w, h) };
    if inner == 0 {
        return;
    }
    let at = |o: usize, i: usize| -> usize {
        if horizontal {
            o.saturating_mul(w).saturating_add(i)
        } else {
            i.saturating_mul(w).saturating_add(o)
        }
    };
    for o in 0..outer {
        // Running sums over the current window: red, green, blue, samples.
        let mut sum = [0u64; 4];
        // The window for the first pixel is [0, min(radius, inner - 1)].
        for k in 0..=radius.min(inner.saturating_sub(1)) {
            window_add(&mut sum, src.get(at(o, k)).copied());
        }
        for i in 0..inner {
            if i > 0 {
                // [max(0, i - r), min(inner - 1, i + r)] against the previous
                // window: the top grows while i + r is still inside, the bottom
                // moves once i - r is past zero.
                let entering = i.saturating_add(radius);
                if entering < inner {
                    window_add(&mut sum, src.get(at(o, entering)).copied());
                }
                if let Some(leaving) = i.checked_sub(radius).and_then(|v| v.checked_sub(1)) {
                    window_remove(&mut sum, src.get(at(o, leaving)).copied());
                }
            }
            let [sr, sg, sb, n] = sum;
            if n == 0 {
                continue;
            }
            let d = |s: u64| -> u8 { s.checked_div(n).unwrap_or(0).min(255) as u8 };
            if let Some(t) = dst.get_mut(at(o, i)) {
                *t = rgba(d(sr), d(sg), d(sb), 255);
            }
        }
    }
}

/// Add a sample to a blur window's running sums. A missing sample (past the
/// end of a short buffer) is not counted.
fn window_add(sum: &mut [u64; 4], p: Option<u32>) {
    if let Some(p) = p {
        let [r, g, b, n] = sum;
        *r = r.saturating_add(u64::from((p >> 16) & 0xFF));
        *g = g.saturating_add(u64::from((p >> 8) & 0xFF));
        *b = b.saturating_add(u64::from(p & 0xFF));
        *n = n.saturating_add(1);
    }
}

/// Take back a sample [`window_add`] added.
fn window_remove(sum: &mut [u64; 4], p: Option<u32>) {
    if let Some(p) = p {
        let [r, g, b, n] = sum;
        *r = r.saturating_sub(u64::from((p >> 16) & 0xFF));
        *g = g.saturating_sub(u64::from((p >> 8) & 0xFF));
        *b = b.saturating_sub(u64::from(p & 0xFF));
        *n = n.saturating_sub(1);
    }
}

/// Linear interpolation between two opaque colours.
#[must_use]
pub fn lerp_color(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |shift: u32| -> u8 {
        let av = f32::from(((a >> shift) & 0xFF) as u8);
        let bv = f32::from(((b >> shift) & 0xFF) as u8);
        (av + (bv - av) * t).round().clamp(0.0, 255.0) as u8
    };
    rgba(ch(16), ch(8), ch(0), 255)
}

/// WCAG 2.1 relative luminance of an opaque `0xAARRGGBB` pixel.
///
/// The same gamma decode as `litric-design`'s `Rgb::luminance`, in `f32`
/// because this runs per pixel. The design crate stays the authority on the
/// numbers; this is the same function at raster precision.
#[must_use]
pub fn luminance(px: u32) -> f32 {
    fn ch(c: u32) -> f32 {
        let v = f32::from((c & 0xFF) as u8) / 255.0;
        if v <= 0.039_28 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * ch(px >> 16) + 0.7152 * ch(px >> 8) + 0.0722 * ch(px)
}

/// Points along an elliptical arc, for the stroke font and for dials.
///
/// `a0` and `a1` are degrees, measured counter-clockwise from the positive x
/// axis with y increasing upward. Sweeping from a larger angle to a smaller one
/// draws the other way round, which is how a font describes the two directions
/// a curve can leave a stem.
#[must_use]
pub fn arc_points(
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    a0: f32,
    a1: f32,
    steps: u32,
) -> Vec<(f32, f32)> {
    let n = steps.max(2);
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let t = f32::from(i as u16) / f32::from(n.saturating_sub(1).max(1) as u16);
        let ang = (a0 + (a1 - a0) * t) * PI / 180.0;
        out.push((cx + rx * ang.cos(), cy + ry * ang.sin()));
    }
    out
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn blending_at_the_extremes_is_exact() {
        let dst = rgb(0x000000);
        let src = rgb(0xFFFFFF);
        assert_eq!(blend(dst, src, 1.0), rgb(0xFFFFFF));
        assert_eq!(blend(dst, src, 0.0), dst);
        assert_eq!(blend(dst, with_alpha(src, 0.0), 1.0), dst);
    }

    #[test]
    fn blending_halfway_lands_halfway() {
        let out = blend(rgb(0x000000), with_alpha(rgb(0xFFFFFF), 0.5), 1.0);
        let r = (out >> 16) & 0xFF;
        assert!((127..=128).contains(&r), "got {r}");
    }

    #[test]
    fn out_of_range_coverage_cannot_wrap_a_channel() {
        // Coverage is computed from a distance function and can arrive slightly
        // out of range; a wrapped channel would be a bright pixel in the middle
        // of a smooth edge, which is exactly what a clamp prevents.
        let out = blend(rgb(0x000000), rgb(0xFFFFFF), 5.0);
        assert_eq!(out, rgb(0xFFFFFF));
        let out = blend(rgb(0x102030), rgb(0xFFFFFF), -3.0);
        assert_eq!(out, rgb(0x102030));
    }

    #[test]
    fn writes_outside_the_canvas_are_dropped_not_wrapped() {
        let mut c = Canvas::new(4, 4, rgb(0x000000));
        c.put(-1, 0, rgb(0xFF0000));
        c.put(0, -1, rgb(0xFF0000));
        c.put(4, 0, rgb(0xFF0000));
        c.put(0, 4, rgb(0xFF0000));
        c.put(999_999, 999_999, rgb(0xFF0000));
        assert!(
            c.pixels().iter().all(|p| *p == rgb(0x000000)),
            "an off-canvas write reached a pixel"
        );
    }

    #[test]
    fn a_clip_stops_drawing_outside_it() {
        let mut c = Canvas::new(40, 40, rgb(0x000000));
        c.set_clip(Some(Rect::new(10.0, 10.0, 10.0, 10.0)));
        c.fill_round_rect(Rect::new(0.0, 0.0, 40.0, 40.0), 0.0, rgb(0xFFFFFF));
        assert_eq!(c.get(15, 15), Some(rgb(0xFFFFFF)), "inside the clip");
        assert_eq!(c.get(5, 5), Some(rgb(0x000000)), "outside the clip");
        assert_eq!(c.get(25, 25), Some(rgb(0x000000)), "outside the clip");
    }

    #[test]
    fn a_fractional_clip_draws_exactly_the_pixels_whose_centres_it_contains() {
        // The drawing loops take their range from the clip rather than testing
        // every pixel, so the range has to be the per-pixel rule exactly: one
        // pixel more leaks outside a repainted region, one fewer leaves a seam.
        for clip in [
            Rect::new(3.3, 2.5, 10.2, 7.7),
            Rect::new(0.5, 0.5, 1.0, 1.0),
            Rect::new(-3.7, 4.49, 8.0, 3.02),
            Rect::new(12.51, 0.0, 0.98, 20.0),
            Rect::new(5.0, 5.0, 0.0, 4.0),
        ] {
            let mut c = Canvas::new(20, 20, rgb(0x000000));
            c.set_clip(Some(clip));
            c.reset_pixels_visited();
            c.fill_vertical_gradient(
                Rect::new(0.0, 0.0, 20.0, 20.0),
                rgb(0xFFFFFF),
                rgb(0xFFFFFF),
            );
            let mut inside = 0u64;
            for y in 0..20i64 {
                for x in 0..20i64 {
                    let want = clip.contains(x as f32 + 0.5, y as f32 + 0.5);
                    inside += u64::from(want);
                    let got = c.get(x, y) == Some(rgb(0xFFFFFF));
                    assert_eq!(got, want, "pixel {x},{y} under clip {clip:?}");
                }
            }
            assert_eq!(
                c.pixels_visited(),
                inside,
                "the loop visited pixels outside {clip:?}"
            );
        }
    }

    #[test]
    fn clips_nest_and_restore() {
        let mut c = Canvas::new(40, 40, rgb(0x000000));
        let outer = c.push_clip(Rect::new(0.0, 0.0, 20.0, 20.0));
        let inner = c.push_clip(Rect::new(10.0, 10.0, 20.0, 20.0));
        // The intersection, not the newer rectangle.
        assert_eq!(c.clip(), Some(Rect::new(10.0, 10.0, 10.0, 10.0)));
        c.set_clip(inner);
        assert_eq!(c.clip(), Some(Rect::new(0.0, 0.0, 20.0, 20.0)));
        c.set_clip(outer);
        assert_eq!(c.clip(), None);
    }

    #[test]
    fn two_clips_that_do_not_overlap_draw_nothing() {
        let mut c = Canvas::new(40, 40, rgb(0x000000));
        c.push_clip(Rect::new(0.0, 0.0, 10.0, 10.0));
        c.push_clip(Rect::new(20.0, 20.0, 10.0, 10.0));
        c.fill_round_rect(Rect::new(0.0, 0.0, 40.0, 40.0), 0.0, rgb(0xFFFFFF));
        assert!(
            c.pixels().iter().all(|p| *p == rgb(0x000000)),
            "disjoint clips still drew something"
        );
    }

    #[test]
    fn blitting_restores_only_inside_the_clip() {
        let src = Canvas::new(16, 16, rgb(0x00FF00));
        let mut c = Canvas::new(16, 16, rgb(0x000000));
        c.set_clip(Some(Rect::new(4.0, 4.0, 4.0, 4.0)));
        c.blit_from(&src, Rect::new(0.0, 0.0, 16.0, 16.0));
        assert_eq!(c.get(5, 5), Some(rgb(0x00FF00)));
        assert_eq!(c.get(3, 5), Some(rgb(0x000000)));
        assert_eq!(c.get(8, 5), Some(rgb(0x000000)));
        assert_eq!(
            c.pixels().iter().filter(|p| **p == rgb(0x00FF00)).count(),
            16
        );
    }

    #[test]
    fn a_filled_rect_covers_its_interior_and_leaves_the_outside_alone() {
        let mut c = Canvas::new(20, 20, rgb(0x000000));
        c.fill_round_rect(Rect::new(5.0, 5.0, 10.0, 10.0), 0.0, rgb(0xFFFFFF));
        assert_eq!(c.get(10, 10), Some(rgb(0xFFFFFF)), "centre not filled");
        assert_eq!(c.get(1, 1), Some(rgb(0x000000)), "outside was touched");
    }

    #[test]
    fn a_rounded_rect_actually_rounds_its_corners() {
        let mut c = Canvas::new(40, 40, rgb(0x000000));
        c.fill_round_rect(Rect::new(5.0, 5.0, 30.0, 30.0), 10.0, rgb(0xFFFFFF));
        let corner = c.get(6, 6).unwrap_or(0);
        let centre = c.get(20, 20).unwrap_or(0);
        assert_eq!(centre, rgb(0xFFFFFF));
        assert!(
            luminance(corner) < 0.5,
            "the corner is filled, so the radius did nothing"
        );
    }

    #[test]
    fn edges_are_anti_aliased_rather_than_stepped() {
        // A fractional edge must produce a partly covered pixel. Without this
        // the whole desktop looks like it was drawn in 1995.
        let mut c = Canvas::new(20, 20, rgb(0x000000));
        c.fill_round_rect(Rect::new(5.5, 5.0, 9.0, 9.0), 0.0, rgb(0xFFFFFF));
        let edge = c.get(5, 8).unwrap_or(0);
        let v = (edge >> 16) & 0xFF;
        assert!((1..=254).contains(&v), "edge pixel was {v}, not partial");
    }

    #[test]
    fn a_line_is_drawn_between_its_endpoints_and_not_beyond_them() {
        let mut c = Canvas::new(30, 30, rgb(0x000000));
        c.stroke_line(5.0, 15.0, 25.0, 15.0, 2.0, rgb(0xFFFFFF));
        assert!(
            luminance(c.get(15, 15).unwrap_or(0)) > 0.8,
            "midpoint missing"
        );
        assert!(
            luminance(c.get(2, 15).unwrap_or(0)) < 0.1,
            "drew past the start"
        );
        assert!(
            luminance(c.get(28, 15).unwrap_or(0)) < 0.1,
            "drew past the end"
        );
    }

    #[test]
    fn a_zero_length_line_draws_a_dot_rather_than_dividing_by_zero() {
        let mut c = Canvas::new(10, 10, rgb(0x000000));
        c.stroke_line(5.0, 5.0, 5.0, 5.0, 3.0, rgb(0xFFFFFF));
        assert!(luminance(c.get(5, 5).unwrap_or(0)) > 0.5);
    }

    #[test]
    fn a_gradient_runs_from_top_to_bottom() {
        let mut c = Canvas::new(10, 20, rgb(0x000000));
        c.fill_vertical_gradient(
            Rect::new(0.0, 0.0, 10.0, 20.0),
            rgb(0x000000),
            rgb(0xFFFFFF),
        );
        let top = luminance(c.get(5, 0).unwrap_or(0));
        let mid = luminance(c.get(5, 10).unwrap_or(0));
        let bot = luminance(c.get(5, 19).unwrap_or(0));
        assert!(top < mid && mid < bot, "{top} {mid} {bot}");
    }

    #[test]
    fn blur_moves_a_hard_edge_into_a_ramp() {
        let mut c = Canvas::new(40, 10, rgb(0x000000));
        c.fill_round_rect(Rect::new(20.0, 0.0, 20.0, 10.0), 0.0, rgb(0xFFFFFF));
        let before = c.get(18, 5).unwrap_or(0);
        c.blur_region(Rect::new(0.0, 0.0, 40.0, 10.0), 6, 2);
        let after = c.get(18, 5).unwrap_or(0);
        assert_eq!(luminance(before), 0.0, "the sample point started black");
        assert!(
            luminance(after) > 0.02,
            "blur did not carry any light across the edge"
        );
    }

    #[test]
    fn blur_of_a_flat_region_changes_nothing() {
        let mut c = Canvas::new(20, 20, rgb(0x203040));
        c.blur_region(Rect::new(0.0, 0.0, 20.0, 20.0), 4, 3);
        assert!(
            c.pixels().iter().all(|p| *p == rgb(0x203040)),
            "a blur of one colour must be that colour"
        );
    }

    /// The box blur as it was before the sliding window: every output pixel
    /// sums its whole window again. Kept here only as the reference the
    /// faster pass must match bit for bit.
    fn reference_box_blur_pass(
        src: &[u32],
        dst: &mut [u32],
        w: usize,
        h: usize,
        radius: usize,
        horizontal: bool,
    ) {
        let (outer, inner) = if horizontal { (h, w) } else { (w, h) };
        for o in 0..outer {
            for i in 0..inner {
                let (mut sr, mut sg, mut sb, mut n) = (0u32, 0u32, 0u32, 0u32);
                let lo = i.saturating_sub(radius);
                let hi = i.saturating_add(radius).min(inner.saturating_sub(1));
                let mut k = lo;
                while k <= hi {
                    let idx = if horizontal {
                        o.saturating_mul(w).saturating_add(k)
                    } else {
                        k.saturating_mul(w).saturating_add(o)
                    };
                    if let Some(p) = src.get(idx) {
                        sr = sr.saturating_add((*p >> 16) & 0xFF);
                        sg = sg.saturating_add((*p >> 8) & 0xFF);
                        sb = sb.saturating_add(*p & 0xFF);
                        n = n.saturating_add(1);
                    }
                    k = k.saturating_add(1);
                }
                if n == 0 {
                    continue;
                }
                let d = |s: u32| -> u8 { s.checked_div(n).unwrap_or(0).min(255) as u8 };
                let idx = if horizontal {
                    o.saturating_mul(w).saturating_add(i)
                } else {
                    i.saturating_mul(w).saturating_add(o)
                };
                if let Some(t) = dst.get_mut(idx) {
                    *t = rgba(d(sr), d(sg), d(sb), 255);
                }
            }
        }
    }

    /// xorshift64: a fixed-seed source of test images, so a failure reproduces.
    fn noise(seed: &mut u64) -> u32 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        (*seed >> 16) as u32
    }

    #[test]
    fn the_sliding_window_blur_matches_the_summing_blur_exactly() {
        let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
        let sizes = [
            (1usize, 1usize),
            (1, 9),
            (9, 1),
            (2, 2),
            (7, 13),
            (33, 5),
            (64, 48),
        ];
        for (w, h) in sizes {
            for radius in [0usize, 1, 2, 3, 5, 8, 31, 1000] {
                for horizontal in [true, false] {
                    let src: Vec<u32> = (0..w * h).map(|_| noise(&mut seed)).collect();
                    let start: Vec<u32> = (0..w * h).map(|_| noise(&mut seed)).collect();
                    let mut want = start.clone();
                    let mut got = start.clone();
                    reference_box_blur_pass(&src, &mut want, w, h, radius, horizontal);
                    box_blur_pass(&src, &mut got, w, h, radius, horizontal);
                    assert_eq!(got, want, "{w}x{h} radius {radius} horizontal={horizontal}");
                }
            }
        }
    }

    #[test]
    fn the_sliding_window_blur_matches_even_over_a_short_buffer() {
        // A source shorter than w*h leaves the last samples out of their
        // windows. The old pass skipped them; so must the new one.
        let mut seed = 42u64;
        let (w, h) = (10usize, 10usize);
        let src: Vec<u32> = (0..95).map(|_| noise(&mut seed)).collect();
        for radius in [1usize, 4, 12] {
            for horizontal in [true, false] {
                let mut want = vec![7u32; 97];
                let mut got = vec![7u32; 97];
                reference_box_blur_pass(&src, &mut want, w, h, radius, horizontal);
                box_blur_pass(&src, &mut got, w, h, radius, horizontal);
                assert_eq!(got, want, "radius {radius} horizontal={horizontal}");
            }
        }
    }

    #[test]
    fn glass_with_a_backdrop_ignores_what_is_under_it() {
        // The property partial repaints rest on: a panel drawn from the
        // backdrop comes out the same over a black scene and a white one.
        let mut paper = Canvas::new(60, 40, rgb(0x000000));
        paper.fill_vertical_gradient(
            Rect::new(0.0, 0.0, 60.0, 40.0),
            rgb(0x2040A0),
            rgb(0xA04020),
        );
        let source = Rc::new(paper.glass_source(3, 1.4));
        let panel = Rect::new(8.0, 6.0, 40.0, 24.0);
        let draw = |fill: u32| -> Canvas {
            let mut c = Canvas::new(60, 40, fill);
            assert!(c.set_backdrop(Some(Rc::clone(&source))));
            c.glass_backdrop(panel, 6.0, 3, 1.4);
            c
        };
        let dark = draw(rgb(0x000000));
        let light = draw(rgb(0xFFFFFF));
        for y in 10..26i64 {
            for x in 14..42i64 {
                assert_eq!(dark.get(x, y), light.get(x, y), "{x},{y}");
                assert_eq!(dark.get(x, y), source.get(x, y), "{x},{y}");
            }
        }
        assert_eq!(dark.get(2, 2), Some(rgb(0x000000)), "outside the panel");
        assert_eq!(
            dark.backdrop_brightest(panel),
            source.brightest(panel),
            "the solver must search the same pixels the panel shows"
        );
    }

    #[test]
    fn a_backdrop_of_the_wrong_size_is_refused() {
        let mut c = Canvas::new(10, 10, rgb(0x000000));
        let wrong = Rc::new(Canvas::new(9, 10, rgb(0xFFFFFF)));
        assert!(!c.set_backdrop(Some(wrong)));
        assert!(c.backdrop().is_none());
        assert!(c.set_backdrop(Some(Rc::new(Canvas::new(10, 10, 0)))));
        assert!(c.backdrop().is_some());
        assert!(c.set_backdrop(None));
        assert!(c.backdrop().is_none());
    }

    #[test]
    fn the_glass_source_is_the_live_glass_path_away_from_the_edges() {
        // The backdrop is the same blur and saturation the live path applies
        // per panel. Deep inside a panel both see the same neighbours.
        let mut paper = Canvas::new(120, 80, rgb(0x000000));
        paper.radial_wash(30.0, 20.0, 70.0, rgb(0x2E5BFF), 0.8);
        paper.radial_wash(100.0, 70.0, 50.0, rgb(0xC2408E), 0.6);
        let panel = Rect::new(30.0, 20.0, 60.0, 40.0);
        let mut live = paper.clone();
        live.glass_backdrop(panel, 4.0, 3, 1.5);
        let source = paper.glass_source(3, 1.5);
        for y in 35..45i64 {
            for x in 50..70i64 {
                assert_eq!(live.get(x, y), source.get(x, y), "{x},{y}");
            }
        }
    }

    #[test]
    fn brightest_finds_the_bright_corner_an_average_would_hide() {
        let mut c = Canvas::new(64, 64, rgb(0x000000));
        c.fill_round_rect(Rect::new(0.0, 0.0, 8.0, 8.0), 0.0, rgb(0xFFFFFF));
        let all = Rect::new(0.0, 0.0, 64.0, 64.0);
        assert!(luminance(c.brightest(all)) > 0.9, "missed the bright patch");
        assert!(
            luminance(c.average(all)) < 0.1,
            "the average is dark, which is why the solver must not use it"
        );
    }

    #[test]
    fn brightest_does_not_change_with_the_clip() {
        let mut c = Canvas::new(64, 64, rgb(0x000000));
        c.fill_round_rect(Rect::new(0.0, 0.0, 8.0, 8.0), 0.0, rgb(0xFFFFFF));
        let all = Rect::new(0.0, 0.0, 64.0, 64.0);
        let unclipped = c.brightest(all);
        c.set_clip(Some(Rect::new(40.0, 40.0, 10.0, 10.0)));
        assert_eq!(c.brightest(all), unclipped);
    }

    #[test]
    fn luminance_agrees_with_the_design_crate_at_the_anchors() {
        assert!(luminance(rgb(0x000000)).abs() < 1e-6);
        assert!((luminance(rgb(0xFFFFFF)) - 1.0).abs() < 1e-4);
        // Mid grey is 0.216, not 0.5. If this ever reads 0.5 the gamma decode
        // has been dropped and every contrast decision on screen is wrong.
        assert!((luminance(rgb(0x808080)) - 0.2159).abs() < 0.002);
    }

    #[test]
    fn an_arc_starts_and_ends_where_it_was_asked_to() {
        let pts = arc_points(0.0, 0.0, 10.0, 10.0, 0.0, 180.0, 16);
        let first = pts.first().copied().unwrap_or((0.0, 0.0));
        let last = pts.last().copied().unwrap_or((0.0, 0.0));
        assert!(
            (first.0 - 10.0).abs() < 0.01 && first.1.abs() < 0.01,
            "{first:?}"
        );
        assert!(
            (last.0 + 10.0).abs() < 0.01 && last.1.abs() < 0.01,
            "{last:?}"
        );
    }

    #[test]
    fn an_arc_swept_backwards_takes_the_other_route() {
        // 180 -> 90 must pass through 135, not through 270. Fonts depend on it:
        // it is the difference between the hook of an f and a loop.
        let pts = arc_points(0.0, 0.0, 10.0, 10.0, 180.0, 90.0, 9);
        let mid = pts.get(4).copied().unwrap_or((0.0, 0.0));
        assert!(mid.1 > 0.0, "the sweep went the wrong way: {mid:?}");
    }

    #[test]
    fn saturation_leaves_grey_grey() {
        let mut c = Canvas::new(4, 4, rgb(0x808080));
        c.saturate_region(Rect::new(0.0, 0.0, 4.0, 4.0), 2.0);
        let p = c.get(1, 1).unwrap_or(0);
        let r = (p >> 16) & 0xFF;
        let g = (p >> 8) & 0xFF;
        let b = p & 0xFF;
        assert!(r.abs_diff(g) <= 1 && g.abs_diff(b) <= 1, "{r} {g} {b}");
    }

    // ---- repaint planning ------------------------------------------------

    fn covers(regions: &[Rect], r: &Rect) -> bool {
        // Every whole pixel of `r` (grown as the planner grows it, cut to the
        // screen) lies inside some region.
        let g = r.inflate(REGION_GROW);
        let (x0, y0) = (g.x.floor().max(0.0) as i64, g.y.floor().max(0.0) as i64);
        let (x1, y1) = (
            g.right().ceil().min(1280.0) as i64,
            g.bottom().ceil().min(800.0) as i64,
        );
        (y0..y1).all(|y| {
            (x0..x1).all(|x| {
                regions
                    .iter()
                    .any(|q| q.contains(x as f32 + 0.5, y as f32 + 0.5))
            })
        })
    }

    #[test]
    fn no_damage_plans_no_regions() {
        assert_eq!(Repaint::plan(&[], 1280, 800), Repaint::Regions(Vec::new()));
    }

    #[test]
    fn the_two_cursor_rectangles_of_a_pointer_move_become_one_whole_pixel_region() {
        let d = [
            Rect::new(637.0, 557.0, 20.0, 26.0),
            Rect::new(645.0, 563.0, 20.0, 26.0),
        ];
        let Repaint::Regions(r) = Repaint::plan(&d, 1280, 800) else {
            panic!("a cursor move planned a full repaint");
        };
        assert_eq!(r, vec![Rect::new(635.0, 555.0, 32.0, 36.0)]);
    }

    #[test]
    fn distant_damage_stays_separate_and_nearby_damage_merges() {
        let d = [
            Rect::new(10.0, 10.0, 10.0, 10.0),
            Rect::new(1000.0, 700.0, 10.0, 10.0),
            Rect::new(30.0, 10.0, 10.0, 10.0),
        ];
        let Repaint::Regions(r) = Repaint::plan(&d, 1280, 800) else {
            panic!("full");
        };
        assert_eq!(r.len(), 2, "{r:?}");
        for x in &d {
            assert!(covers(&r, x), "{x:?} not covered by {r:?}");
        }
    }

    #[test]
    fn regions_are_capped_and_still_cover_every_damaged_pixel() {
        let d: Vec<Rect> = (0..24)
            .map(|i| Rect::new(40.0 + 50.0 * i as f32, 40.0 + 30.0 * i as f32, 6.0, 6.0))
            .collect();
        let Repaint::Regions(r) = Repaint::plan(&d, 1280, 800) else {
            panic!("full");
        };
        assert!(r.len() <= REGION_MAX, "{} regions", r.len());
        for x in &d {
            assert!(covers(&r, x), "{x:?} not covered by {r:?}");
        }
        for (i, a) in r.iter().enumerate() {
            for b in r.iter().skip(i + 1) {
                assert!(a.intersection(b).is_none(), "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn damage_off_the_screen_is_dropped_and_damage_across_its_edge_is_cut() {
        let d = [
            Rect::new(-500.0, -500.0, 100.0, 100.0),
            Rect::new(1270.0, 790.0, 100.0, 100.0),
            Rect::new(f32::NAN, 3.0, 10.0, 10.0),
        ];
        let Repaint::Regions(r) = Repaint::plan(&d, 1280, 800) else {
            panic!("full");
        };
        assert_eq!(r, vec![Rect::new(1268.0, 788.0, 12.0, 12.0)]);
    }

    /// A canvas with something under the glass, so the blur has real pixels to
    /// average rather than a flat colour where every answer looks the same.
    fn textured(w: u32, h: u32) -> Canvas {
        let mut c = Canvas::new(w, h, rgb(0x101820));
        for y in 0..h as i64 {
            for x in 0..w as i64 {
                let v = ((x * 7 + y * 13) % 256) as u8;
                c.put(x, y, rgba(v, v.div_euclid(3), v.div_euclid(7), 255));
            }
        }
        c
    }

    #[test]
    fn a_partial_glass_blur_paints_what_the_full_one_painted() {
        // The whole point of cutting the blur's working region to the clip is
        // that it must not be a change of rendering, only of work. Painting the
        // same panel once unclipped and once clipped to one damage region must
        // leave that region's pixels bit-identical, or the clip-aware region has
        // bought speed with a seam.
        let panel = Rect::new(80.0, 60.0, 300.0, 200.0);
        let damage = Rect::new(300.0, 200.0, 40.0, 24.0);
        let full = {
            let mut c = textured(480, 360);
            c.glass_backdrop(panel, 12.0, 6, 1.2);
            c
        };
        let mut part = textured(480, 360);
        part.set_clip(Some(damage));
        part.glass_backdrop(panel, 12.0, 6, 1.2);
        let (x0, y0, x1, y1) = part.span_all(&damage, Site::Glass);
        for y in y0..y1 {
            for x in x0..x1 {
                assert_eq!(
                    part.get(x, y),
                    full.get(x, y),
                    "clipped blur changed ({x},{y}) against the unclipped one"
                );
            }
        }
    }

    #[test]
    fn a_glass_panel_the_damage_region_misses_blurs_nothing() {
        // BUG-0015: a damage region in one corner of the screen was making every
        // window on it re-blur its whole backdrop. A panel the clip does not
        // touch has no pixel a write could reach, so it must cost zero -- and
        // the count has to say zero, not "a panel's worth, discarded later".
        let mut c = textured(1280, 800);
        c.set_clip(Some(Rect::new(600.0, 560.0, 160.0, 48.0)));
        c.reset_pixels_visited();
        c.glass_backdrop(Rect::new(40.0, 40.0, 640.0, 480.0), 12.0, 6, 1.2);
        assert_eq!(
            c.pixels_visited(),
            0,
            "a panel outside the clip still walked pixels"
        );
    }

    #[test]
    fn a_clipped_glass_blur_visits_about_the_damage_region_not_the_panel() {
        // The measured claim, as a test: the working region is the clip grown by
        // the kernel's reach, so a 160x48 damage region costs thousands of
        // pixels, not the whole 640x480 panel behind it, and nowhere near the
        // 98x the frame used to hand the loops.
        let mut c = textured(1280, 800);
        let damage = Rect::new(600.0, 560.0, 160.0, 48.0);
        c.set_clip(Some(damage));
        c.reset_pixels_visited();
        // A panel that really does cover the damage region: the case that used to
        // re-blur all 307 200 pixels behind it.
        c.glass_backdrop(Rect::new(400.0, 400.0, 640.0, 480.0), 12.0, 6, 1.2);
        let visited = c.pixels_visited();
        let area = (damage.w * damage.h) as u64;
        assert!(visited > 0, "the overlapping panel must still blur");
        assert!(
            visited < area * 4,
            "{visited} px for a {area} px region is still panel-sized work"
        );
    }

    #[test]
    fn a_sampled_search_bills_the_samples_it_takes() {
        // The other half of the 98x. `brightest` reads every fourth pixel on
        // both axes; billing the whole rectangle made a 640x480 panel report
        // 307 200 visited pixels for 19 200 reads, which is what made the
        // partial-frame figure unusable for deciding where work was.
        let c = textured(1280, 800);
        c.reset_pixels_visited();
        let r = Rect::new(0.0, 0.0, 640.0, 480.0);
        let _ = c.brightest(r);
        let billed = c.pixels_visited();
        let area = (r.w * r.h) as u64;
        assert_eq!(
            billed,
            area.div_euclid(16),
            "a 4x4 sampling grid is one read in 16"
        );
    }

    #[test]
    fn the_pixel_bill_names_the_loop_that_asked() {
        let mut c = textured(200, 200);
        c.reset_pixels_visited();
        let prev = c.set_site(Site::Shadow);
        c.fill_round_rect(Rect::new(0.0, 0.0, 50.0, 50.0), 0.0, rgb(0xFFFFFF));
        c.set_site(prev);
        c.fill_round_rect(Rect::new(60.0, 0.0, 50.0, 50.0), 0.0, rgb(0xFFFFFF));
        let by = c.pixels_visited_by_site();
        assert_eq!(by.len(), 2, "two kinds were billed: {by:?}");
        assert_eq!(by.first().map(|(s, _)| *s), Some(Site::Draw));
        assert_eq!(by.get(1).map(|(s, _)| *s), Some(Site::Shadow));
        assert_eq!(c.pixels_visited(), by.iter().map(|(_, n)| n).sum::<u64>());
    }

    #[test]
    fn a_soft_shadow_is_billed_to_the_shadow_not_to_plain_fills() {
        let mut c = textured(200, 200);
        c.reset_pixels_visited();
        c.soft_shadow(Rect::new(50.0, 50.0, 60.0, 40.0), 8.0, 20.0, 4.0, 0.5);
        let by = c.pixels_visited_by_site();
        assert_eq!(by.first().map(|(s, _)| *s), Some(Site::Shadow), "{by:?}");
        assert!(by.first().is_some_and(|(_, n)| *n > 0));
    }

    #[test]
    fn damage_over_half_the_screen_plans_a_full_repaint() {
        assert_eq!(
            Repaint::plan(&[Rect::new(0.0, 0.0, 1280.0, 420.0)], 1280, 800),
            Repaint::Full
        );
        assert_ne!(
            Repaint::plan(&[Rect::new(0.0, 0.0, 1280.0, 380.0)], 1280, 800),
            Repaint::Full
        );
        assert_eq!(
            Repaint::Full.rects(1280, 800),
            vec![Rect::new(0.0, 0.0, 1280.0, 800.0)]
        );
    }
}
