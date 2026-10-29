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
        round_half_up(v).min(255) << shift
    };
    0xFF00_0000 | mix(16) | mix(8) | mix(0)
}

/// `v.round()` for a `v` that is never negative, as an integer.
///
/// `f32::round` is a library call on the targets this shell is built for -- no
/// SSE4.1 `roundss` is assumed -- and every pixel the renderer composites went
/// through it three times, which the profile priced at about a twelfth of a
/// frame. For a non-negative value, rounding half away from zero is: take the
/// integer part, and add one if the fraction is at least a half. The fraction
/// is `v` minus a nearby integer, which a float subtracts exactly, so this is
/// the same answer as `round` for every input, not an approximation of it.
#[inline]
fn round_half_up(v: f32) -> u32 {
    let whole = v as u32;
    if v - whole as f32 >= 0.5 {
        whole.saturating_add(1)
    } else {
        whole
    }
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
    let a = round_half_up(alpha.clamp(0.0, 1.0) * 255.0).min(255);
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

/// Half-width of a rounded rectangle's cross-section at `dy` from its centre.
///
/// Used to walk only the annulus a stroke can touch. A rounded rectangle grown
/// by `g` is exactly the same shape's signed distance shifted by `g`, so the
/// outer and inner edges of a stroke are rounded rectangles with every extent
/// and the radius moved by `half + 0.5`; one square root per row gives their
/// horizontal extent instead of one distance test per pixel of the interior.
/// Returns -1 when the row is outside the shape.
fn rounded_row_half_width(dy: f32, hw: f32, hh: f32, rad: f32) -> f32 {
    if hw.is_nan() || hh.is_nan() || hw <= 0.0 || hh <= 0.0 || dy >= hh {
        return -1.0;
    }
    if rad <= 0.0 || dy <= hh - rad {
        return hw;
    }
    let k = dy - (hh - rad);
    let inside = rad.mul_add(rad, -(k * k));
    if inside <= 0.0 {
        return hw - rad;
    }
    (hw - rad) + inside.sqrt()
}

/// Coverage of one pixel from the geometry [`Canvas::fill_round_rect`] derives
/// once per call.
///
/// Shared with the profiling counter so the reported pixel buckets are
/// classified by exactly the arithmetic the fill uses.
#[inline]
#[must_use]
fn edge_coverage(px: f32, cx: f32, qx_off: f32, qy: f32, ay: f32, rad: f32) -> f32 {
    let qx = (px - cx).abs() - qx_off;
    let ax = qx.max(0.0);
    coverage((ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad)
}

/// Nanoseconds since `start`, saturating instead of wrapping.
fn nanos_since(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
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

/// Opt-in call-level timers for the frame path.
///
/// This host has no sampling profiler (`perf`, `valgrind`, `heaptrack` and
/// `bpftrace` are all absent, and `perf_event_paranoid` is 2), so a frame is
/// attributed by timing the drawing entry points themselves. Profiling is off
/// by default: a call pays one flag read, and nothing is allocated until a
/// name is recorded the first time.
pub mod profile {
    use std::cell::{Cell, RefCell};
    use std::time::Instant;

    /// One entry point's accumulated cost since the last [`take`].
    #[derive(Clone, Copy)]
    pub struct Row {
        /// The drawing entry point this row times.
        pub name: &'static str,
        /// How many times it was called since the last [`take`].
        pub calls: u64,
        /// Total time inside it, in nanoseconds.
        pub nanos: u64,
        /// Pixels the calls were given, when the caller reported them.
        pub pixels: u64,
    }

    thread_local! {
        static ON: Cell<bool> = const { Cell::new(false) };
        static ROWS: RefCell<Vec<Row>> = const { RefCell::new(Vec::new()) };
        static DIMS: RefCell<Vec<(&'static str, u32, u32)>> = const { RefCell::new(Vec::new()) };
        static RECTS: RefCell<Vec<(&'static str, super::Rect)>> = const { RefCell::new(Vec::new()) };
    }

    /// Start timing calls on this thread.
    pub fn enable() {
        ON.with(|on| on.set(true));
    }

    /// Stop timing calls on this thread.
    pub fn disable() {
        ON.with(|on| on.set(false));
    }

    /// Whether calls are being timed.
    #[must_use]
    pub fn is_on() -> bool {
        ON.with(Cell::get)
    }

    fn record(name: &'static str, nanos: u64) {
        ROWS.with(|rows| {
            let mut rows = rows.borrow_mut();
            if let Some(row) = rows.iter_mut().find(|row| row.name == name) {
                row.calls = row.calls.saturating_add(1);
                row.nanos = row.nanos.saturating_add(nanos);
            } else {
                rows.push(Row {
                    name,
                    calls: 1,
                    nanos,
                    pixels: 0,
                });
            }
        });
    }

    /// Add the pixel area one call was given, once per call and only while
    /// profiling, so the report can divide time by pixels.
    pub fn note_pixels(name: &'static str, pixels: u64) {
        if !is_on() {
            return;
        }
        ROWS.with(|rows| {
            let mut rows = rows.borrow_mut();
            if let Some(row) = rows.iter_mut().find(|row| row.name == name) {
                row.pixels = row.pixels.saturating_add(pixels);
            } else {
                rows.push(Row {
                    name,
                    // The call counter belongs to the timer that is already
                    // running for this call; a placeholder of 1 here made every
                    // bucket that note_pixels created read one call high
                    // (2 panels + 1 reported as 3 + 2 against an outer 3).
                    calls: 0,
                    nanos: 0,
                    pixels,
                });
            }
        });
    }

    /// Remember the last rectangle a name was called with, for the report.
    pub fn note_dims(name: &'static str, w: u32, h: u32) {
        if !is_on() {
            return;
        }
        DIMS.with(|dims| {
            let mut dims = dims.borrow_mut();
            if let Some(entry) = dims.iter_mut().find(|entry| entry.0 == name) {
                entry.1 = w;
                entry.2 = h;
            } else {
                dims.push((name, w, h));
            }
        });
    }

    /// Take the remembered rectangles.
    #[must_use]
    pub fn take_dims() -> Vec<(&'static str, u32, u32)> {
        DIMS.with(|dims| std::mem::take(&mut *dims.borrow_mut()))
    }

    /// Remember the rectangle one call was given, in call order.
    ///
    /// A size bucket alone cannot say whether two panels of the same class
    /// cover the same pixels; the rectangles can. One push per glass call while
    /// profiling -- never inside a pixel loop -- and nothing at all on a timed
    /// frame, because [`is_on`] is false there.
    pub fn note_rect(name: &'static str, r: super::Rect) {
        if !is_on() {
            return;
        }
        RECTS.with(|rects| rects.borrow_mut().push((name, r)));
    }

    /// Take the remembered rectangles, leaving the list empty.
    #[must_use]
    pub fn take_rects() -> Vec<(&'static str, super::Rect)> {
        RECTS.with(|rects| std::mem::take(&mut *rects.borrow_mut()))
    }

    /// Take what has been recorded, leaving the counters empty.
    #[must_use]
    pub fn take() -> Vec<Row> {
        ROWS.with(|rows| std::mem::take(&mut *rows.borrow_mut()))
    }

    /// One measured slice of a drawing row's pixels.
    ///
    /// A [`Row`] says how long an entry point took; a bucket says which part of
    /// its pixel work that time went to. The slices are disjoint, so their
    /// pixels add up to the area the entry point walked.
    #[derive(Clone, Copy, Debug)]
    pub struct Bucket {
        /// The slice this bucket measures.
        pub name: &'static str,
        /// Pixels in the slice.
        pub pixels: u64,
        /// Time spent on the slice, in nanoseconds, when it was timed.
        pub nanos: u64,
    }

    /// A whole-call count that is not a pixel area.
    #[derive(Clone, Copy)]
    pub struct Counter {
        /// What is counted.
        pub name: &'static str,
        /// The accumulated value.
        pub value: u64,
    }

    thread_local! {
        static BUCKETS: RefCell<Vec<Bucket>> = const { RefCell::new(Vec::new()) };
        static COUNTERS: RefCell<Vec<Counter>> = const { RefCell::new(Vec::new()) };
    }

    /// Add one call's slice to a bucket. Nothing happens on a timed frame.
    pub fn note_bucket(name: &'static str, pixels: u64, nanos: u64) {
        if !is_on() {
            return;
        }
        BUCKETS.with(|buckets| {
            let mut buckets = buckets.borrow_mut();
            if let Some(b) = buckets.iter_mut().find(|b| b.name == name) {
                b.pixels = b.pixels.saturating_add(pixels);
                b.nanos = b.nanos.saturating_add(nanos);
            } else {
                buckets.push(Bucket {
                    name,
                    pixels,
                    nanos,
                });
            }
        });
    }

    /// Add `value` to a whole-call counter.
    pub fn note_counter(name: &'static str, value: u64) {
        if !is_on() {
            return;
        }
        COUNTERS.with(|counters| {
            let mut counters = counters.borrow_mut();
            if let Some(c) = counters.iter_mut().find(|c| c.name == name) {
                c.value = c.value.saturating_add(value);
            } else {
                counters.push(Counter { name, value });
            }
        });
    }

    /// Keep the largest `value` seen for a whole-call counter.
    pub fn note_max(name: &'static str, value: u64) {
        if !is_on() {
            return;
        }
        COUNTERS.with(|counters| {
            let mut counters = counters.borrow_mut();
            if let Some(c) = counters.iter_mut().find(|c| c.name == name) {
                c.value = c.value.max(value);
            } else {
                counters.push(Counter { name, value });
            }
        });
    }

    /// Take the buckets, leaving them empty.
    #[must_use]
    pub fn take_buckets() -> Vec<Bucket> {
        BUCKETS.with(|buckets| std::mem::take(&mut *buckets.borrow_mut()))
    }

    /// Take the whole-call counters, leaving them empty.
    #[must_use]
    pub fn take_counters() -> Vec<Counter> {
        COUNTERS.with(|counters| std::mem::take(&mut *counters.borrow_mut()))
    }

    /// Times its own lifetime when profiling is on.
    pub struct Timer {
        name: &'static str,
        start: Option<Instant>,
    }

    impl Timer {
        /// Start a timer for `name`, or do nothing when profiling is off.
        #[must_use]
        pub fn new(name: &'static str) -> Self {
            Self {
                name,
                start: is_on().then(Instant::now),
            }
        }
    }

    impl Drop for Timer {
        fn drop(&mut self) {
            if let Some(start) = self.start {
                let nanos = u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX);
                record(self.name, nanos);
            }
        }
    }
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

    /// Run `draw` with the text recorder paused, then put it back.
    ///
    /// For ink that is not a line of text anyone reads: the shadow under a word
    /// is the word again, a pixel lower, and recording it would report every
    /// word as overlapping itself.
    pub fn unrecorded(&mut self, draw: impl FnOnce(&mut Self)) {
        let runs = self.runs.take();
        draw(self);
        self.runs = runs;
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

    /// The attached backdrop as an owned handle, so a surface that swaps it for
    /// one of its own can put the previous one back.
    #[must_use]
    pub fn backdrop_owned(&self) -> Option<Rc<Self>> {
        self.backdrop.clone()
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

    /// Cross-fade towards another canvas of the same size: every pixel moves
    /// `t` (`0..=1`) of the way from what it is to what `src` has, inside the
    /// clip. Both are opaque, so this is a plain per-channel mix.
    ///
    /// This is how the boot surface hands over to the desktop. Drawing a
    /// translucent mark over the desktop would blend every overlapping pen
    /// segment twice and bead the joints; mixing two finished frames cannot.
    // The mix is `from * (256 - k) + to * k + 128` per channel with `from` and
    // `to` in `0..=255` and `k` in `0..=256`, a peak of 131 * 072 -- no term of
    // it can reach the end of a `u32`, and the halfway rounding is deliberate.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn mix_from(&mut self, src: &Self, t: f32) {
        if self.width != src.width || self.height != src.height {
            return;
        }
        let k = round_half_up(t.clamp(0.0, 1.0) * 256.0).min(256);
        if k == 0 {
            return;
        }
        let r = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);
        let (x0, y0, x1, y1) = self.span(&r);
        for y in y0..y1 {
            let row = (y as usize).saturating_mul(self.width as usize);
            for x in x0..x1 {
                let i = row.saturating_add(x as usize);
                let (Some(&b), Some(d)) = (src.px.get(i), self.px.get_mut(i)) else {
                    continue;
                };
                let mix = |shift: u32| {
                    let from = (*d >> shift) & 0xFF;
                    let to = (b >> shift) & 0xFF;
                    (from * (256 - k) + to * k + 128) >> 8
                };
                *d = 0xFF00_0000 | (mix(16) << 16) | (mix(8) << 8) | mix(0);
            }
        }
    }

    /// Copy a region from another canvas, inside the clip.
    ///
    /// This is how damage tracking erases: the wallpaper is rendered once into
    /// its own canvas, and repainting a region starts by restoring it here.
    /// Without it, a window that moves leaves a copy of itself behind. Between
    /// canvases of the same size each row is one slice copy.
    pub fn blit_from(&mut self, src: &Self, r: Rect) {
        let _prof = profile::Timer::new("blit_from");
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

    /// Paint decoded pixels into `dst`, scaling them to fit.
    ///
    /// `pixels` is one `0xFFRRGGBB` word per source pixel, the form the JPEG
    /// decoder produces. The mapping is nearest-neighbour for the same reason
    /// as [`Self::blit_bgra`]: the cost of a frame is bounded by the area on
    /// screen, never by the decoded picture's own size.
    pub fn blit_argb(&mut self, dst: Rect, pixels: &[u32], src_w: u32, src_h: u32) {
        if src_w == 0 || src_h == 0 || dst.w <= 0.0 || dst.h <= 0.0 {
            return;
        }
        let (x0, y0, x1, y1) = self.span(&dst);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let sw = i64::from(src_w);
        let dw = f64::from(dst.w);
        let dh = f64::from(dst.h);
        let mut drawn: u64 = 0;
        for y in y0..y1 {
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let sy = (((y as f64 + 0.5 - f64::from(dst.y)) / dh) * f64::from(src_h))
                .max(0.0)
                .min(f64::from(src_h.saturating_sub(1))) as i64;
            let row = sy.saturating_mul(sw);
            for x in x0..x1 {
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                let sx = (((x as f64 + 0.5 - f64::from(dst.x)) / dw) * f64::from(src_w))
                    .max(0.0)
                    .min(f64::from(src_w.saturating_sub(1))) as i64;
                let at = usize::try_from(row.saturating_add(sx)).unwrap_or(usize::MAX);
                let Some(&px) = pixels.get(at) else {
                    continue;
                };
                self.put_in(x, y, px | 0xFF00_0000);
                drawn = drawn.saturating_add(1);
            }
        }
        self.count_px(Site::Draw, drawn);
    }

    /// Paint a client's buffer into `dst`, scaling it to fit.
    ///
    /// `pixels` is what a Wayland client committed: rows tightly packed, four
    /// bytes a pixel, ordered `B, G, R, A` in memory (little-endian
    /// `0xAARRGGBB`). `src_w`/`src_h` are the buffer's logical size, so a
    /// client that renders at twice the density is drawn at the size it asked
    /// for rather than at the size of its allocation.
    ///
    /// `opaque` is for `XRGB8888`, where the fourth byte is padding the client
    /// is allowed to leave at zero: honouring it would make a whole window
    /// invisible.
    ///
    /// The mapping is nearest-neighbour. A window that is resized faster than
    /// its client repaints is shown stretched rather than blank, and the cost
    /// of a frame is bounded by the window's area on screen — never by the
    /// client's buffer, which can be far larger.
    pub fn blit_bgra(&mut self, dst: Rect, pixels: &[u8], src_w: u32, src_h: u32, opaque: bool) {
        if src_w == 0 || src_h == 0 || dst.w <= 0.0 || dst.h <= 0.0 {
            return;
        }
        let (x0, y0, x1, y1) = self.span(&dst);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let sw = i64::from(src_w);
        let dw = f64::from(dst.w);
        let dh = f64::from(dst.h);
        let mut drawn: u64 = 0;
        for y in y0..y1 {
            // Pixel centres, so a 2x downscale is not biased a whole pixel up.
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let sy = (((y as f64 + 0.5 - f64::from(dst.y)) / dh) * f64::from(src_h))
                .max(0.0)
                .min(f64::from(src_h.saturating_sub(1))) as i64;
            let row = sy.saturating_mul(sw);
            for x in x0..x1 {
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                let sx = (((x as f64 + 0.5 - f64::from(dst.x)) / dw) * f64::from(src_w))
                    .max(0.0)
                    .min(f64::from(src_w.saturating_sub(1))) as i64;
                let at = row.saturating_add(sx).saturating_mul(4);
                let at = usize::try_from(at).unwrap_or(usize::MAX);
                let Some(px) = pixels
                    .get(at..at.saturating_add(4))
                    .and_then(|quad| quad.first_chunk::<4>())
                else {
                    continue;
                };
                let mut word = u32::from_le_bytes(*px);
                if opaque {
                    word |= 0xFF00_0000;
                }
                self.put_in(x, y, word);
                drawn = drawn.saturating_add(1);
            }
        }
        self.count_px(Site::Draw, drawn);
    }

    /// Copy the pixels inside `r` out of the canvas, ignoring the clip.
    ///
    /// A software cursor is drawn into the frame itself, so moving it has to put
    /// back exactly the pixels it covered. Keeping those pixels is what turns a
    /// pointer move from a whole composition into a restore and a small draw.
    /// `None` when the rectangle misses the canvas entirely.
    #[must_use]
    pub fn snapshot(&self, r: Rect) -> Option<(Rect, Vec<u32>)> {
        let (x0, y0, x1, y1) = self.bounds_of(&r);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        let w = self.width as usize;
        let cols = usize::try_from(x1.saturating_sub(x0)).ok()?;
        let mut px =
            Vec::with_capacity(cols.saturating_mul(usize::try_from(y1.saturating_sub(y0)).ok()?));
        for y in y0..y1 {
            let row = (y as usize).saturating_mul(w);
            let range = row.saturating_add(x0 as usize)..row.saturating_add(x1 as usize);
            px.extend_from_slice(self.px.get(range)?);
        }
        #[allow(clippy::cast_precision_loss)]
        let rect = Rect::new(
            x0 as f32,
            y0 as f32,
            x1.saturating_sub(x0) as f32,
            y1.saturating_sub(y0) as f32,
        );
        Some((rect, px))
    }

    /// Put back pixels saved by [`Canvas::snapshot`], ignoring the clip.
    pub fn restore(&mut self, r: Rect, pixels: &[u32]) {
        let (x0, y0, x1, y1) = self.bounds_of(&r);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let w = self.width as usize;
        let cols = usize::try_from(x1.saturating_sub(x0)).unwrap_or(0);
        for (i, y) in (y0..y1).enumerate() {
            let row = (y as usize).saturating_mul(w);
            let range = row.saturating_add(x0 as usize)..row.saturating_add(x1 as usize);
            let from = i.saturating_mul(cols);
            let Some(src) = pixels.get(from..from.saturating_add(cols)) else {
                return;
            };
            if let Some(dst) = self.px.get_mut(range) {
                dst.copy_from_slice(src);
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
        let _prof = profile::Timer::new("fill_round_rect");
        let prof = profile::is_on();
        let bbox = r.inflate(1.5);
        // Pixels the inflated shape touches before the canvas and the clip cut
        // it, so the report can name the clip-cut slice. Only computed while
        // profiling; a timed frame pays one flag read.
        let clipped_px = if prof {
            let raw_w = (bbox.right().ceil() - bbox.x.floor()).max(0.0);
            let raw_h = (bbox.bottom().ceil() - bbox.y.floor()).max(0.0);
            let raw = (raw_w * raw_h) as u64;
            let (bx0, by0, bx1, by1) = self.bounds_of(&bbox);
            let (mut cx0, mut cy0, mut cx1, mut cy1) = (bx0, by0, bx1, by1);
            if let Some(c) = self.clip {
                let (k0, k1, k2, k3) = clip_pixels(&c);
                cx0 = cx0.max(k0);
                cy0 = cy0.max(k1);
                cx1 = cx1.min(k2);
                cy1 = cy1.min(k3);
            }
            let kept_w = u64::try_from(cx1.max(cx0).saturating_sub(cx0)).unwrap_or(0);
            let kept_h = u64::try_from(cy1.max(cy0).saturating_sub(cy0)).unwrap_or(0);
            raw.saturating_sub(kept_w.saturating_mul(kept_h))
        } else {
            0
        };
        let (x0, y0, x1, y1) = self.span(&bbox);
        let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
        // Everything the signed distance needs from the rectangle, derived once
        // per call rather than once per pixel. The old loop recomputed the
        // centre, both half-extents and the clamped radius -- two divides and a
        // chain of min/max -- for every pixel it visited, and then paid
        // `index()`'s bounds check and multiply again to write the result.
        let (cx, cy) = r.center();
        let qx_off = r.w / 2.0 - rad;
        let qy_off = r.h / 2.0 - rad;
        let width = self.width as usize;
        let opaque = (color >> 24) & 0xFF == 0xFF;
        // The three per-row classes come from the distance itself. A pixel's
        // centre is at most `u = |x + 0.5 - cx|` from the middle line, and
        // `d(u) = sqrt(max(u - qx_off, 0)^2 + ay^2) - rad`. It is exactly full
        // coverage where `d <= -0.5`, partial where `-0.5 < d < 0.5`, and
        // untouched beyond that. Solving both boundaries gives one square root
        // per row instead of one per edge pixel. The quarter-pixel margins keep
        // boundary pixels on the exact side: the full span is shrunk and the
        // partial band grown, so a misclassified boundary pixel still goes
        // through the same distance test and only an exactly-full one can be
        // written by the fast path.
        let full_cap = rad - 0.5;
        let paint_cap = rad + 0.5;
        // A translucent core cannot be a plain fill, but for a fixed colour and
        // full coverage a channel's result depends only on that channel's
        // destination value. 256 samples of `blend` therefore hold every
        // possible output, and the table is built *with* `blend`, so the pixels
        // are the same integers rather than an approximation of them.
        let full_est =
            ((r.w - 1.0).max(0.0) * (r.h - 1.0).max(0.0) - (4.0 - PI) * rad * rad).max(0.0);
        let mut blend_table = [0u32; 256];
        let use_table = !opaque && full_est >= 1024.0;
        if use_table {
            for d in 0..256_u32 {
                let sample = 0xFF00_0000 | (d << 16) | (d << 8) | d;
                if let Some(slot) = blend_table.get_mut(d as usize) {
                    *slot = blend(sample, color, 1.0);
                }
            }
        }
        let mut painted = 0u64;
        // Per-slice accounting, accumulated in registers and reported once per
        // call. Zero work on a timed frame: the timestamps are only taken when
        // `prof` is true, and the counting pass below does not run at all.
        let mut edge_nanos = 0u64;
        let mut core_nanos = 0u64;
        let mut edge_scanned = 0u64;
        let mut interior = 0u64;
        let mut rows_walked = 0u64;
        // Write a run whose coverage is exactly 1. `blend` returns an opaque
        // source unchanged at full coverage, and the table's entries were built
        // with `blend` at full coverage, so both are the same integers the
        // distance path would have written.
        let fill_run = |row: &mut [u32], from: i64, to: i64| {
            let Some(seg) = row.get_mut(from as usize..to as usize) else {
                return;
            };
            if opaque {
                seg.fill(color);
            } else if use_table {
                for p in seg.iter_mut() {
                    let v = *p;
                    let tr = blend_table
                        .get(((v >> 16) & 0xFF) as usize)
                        .copied()
                        .unwrap_or(0);
                    let tg = blend_table
                        .get(((v >> 8) & 0xFF) as usize)
                        .copied()
                        .unwrap_or(0);
                    let tb = blend_table.get((v & 0xFF) as usize).copied().unwrap_or(0);
                    *p = 0xFF00_0000 | (tr & 0x00FF_0000) | (tg & 0x0000_FF00) | (tb & 0xFF);
                }
            } else {
                for p in seg.iter_mut() {
                    *p = blend(*p, color, 1.0);
                }
            }
        };
        // Pixels outside the full span but inside the coverage ramp: the only
        // ones that need a square root.
        let edge_run = |row: &mut [u32], from: i64, to: i64, qy: f32, ay: f32| -> u64 {
            for x in from..to {
                let cov = edge_coverage(x as f32 + 0.5, cx, qx_off, qy, ay, rad);
                if cov > 0.0 {
                    if let Some(p) = row.get_mut(x as usize) {
                        *p = blend(*p, color, cov);
                    }
                }
            }
            u64::try_from(to.saturating_sub(from)).unwrap_or(0)
        };
        for y in y0..y1 {
            let base = (y as usize).saturating_mul(width);
            let Some(row) = self.px.get_mut(base..base.saturating_add(width)) else {
                continue;
            };
            if prof {
                rows_walked = rows_walked.saturating_add(1);
            }
            let py = y as f32 + 0.5;
            let qy = (py - cy).abs() - qy_off;
            let ay = qy.max(0.0);
            let full_half = if rad > 0.5 && ay <= full_cap {
                (full_cap * full_cap - ay * ay).max(0.0).sqrt()
            } else if rad <= 0.5 && qy <= -0.5 {
                rad - 0.5
            } else {
                -1.0
            };
            let paint_half = if ay < paint_cap {
                (paint_cap * paint_cap - ay * ay).max(0.0).sqrt()
            } else {
                -1.0
            };
            let (paint_from, paint_to) = if paint_half >= 0.0 {
                let u = qx_off + paint_half + 0.25;
                (
                    (((cx - u - 0.5).floor() as i64).saturating_add(1)).clamp(x0, x1),
                    ((cx + u - 0.5).ceil() as i64).clamp(x0, x1),
                )
            } else {
                (x0, x0)
            };
            let (full_from, full_to) = if full_half >= 0.0 {
                let u = (qx_off + full_half - 0.25).max(0.0);
                (
                    ((cx - u - 0.5).ceil() as i64).clamp(x0, x1),
                    (((cx + u - 0.5).floor() as i64).saturating_add(1)).clamp(x0, x1),
                )
            } else {
                (x0, x0)
            };
            let (left_from, left_to, fill_from, fill_to, right_from, right_to) =
                if full_from < full_to {
                    (paint_from, full_from, full_from, full_to, full_to, paint_to)
                } else {
                    (
                        paint_from, paint_from, paint_from, paint_from, paint_from, paint_to,
                    )
                };
            let edge_start = prof.then(std::time::Instant::now);
            let mut n_edge = edge_run(row, left_from, left_to, qy, ay);
            n_edge = n_edge.saturating_add(edge_run(row, right_from, right_to, qy, ay));
            if let Some(start) = edge_start {
                edge_nanos = edge_nanos.saturating_add(nanos_since(start));
            }
            let core_start = prof.then(std::time::Instant::now);
            let n_core = u64::try_from(fill_to.saturating_sub(fill_from)).unwrap_or(0);
            if fill_from < fill_to {
                fill_run(row, fill_from, fill_to);
            }
            if let Some(start) = core_start {
                core_nanos = core_nanos.saturating_add(nanos_since(start));
            }
            interior = interior.saturating_add(n_core);
            edge_scanned = edge_scanned.saturating_add(n_edge);
            painted = painted.saturating_add(n_core).saturating_add(n_edge);
        }
        if prof {
            // One untimed second pass classifies the pixels the partial bands
            // visited. It uses the same coverage expression as the fill, so a
            // bucket cannot say something the painted frame contradicts. It
            // runs only on the profiled frame, never on a timed one.
            let mut edge_painted = 0u64;
            for y in y0..y1 {
                let base = (y as usize).saturating_mul(width);
                if self.px.get(base..base.saturating_add(width)).is_none() {
                    continue;
                }
                let py = y as f32 + 0.5;
                let qy = (py - cy).abs() - qy_off;
                let ay = qy.max(0.0);
                let full_half = if rad > 0.5 && ay <= full_cap {
                    (full_cap * full_cap - ay * ay).max(0.0).sqrt()
                } else if rad <= 0.5 && qy <= -0.5 {
                    rad - 0.5
                } else {
                    -1.0
                };
                let paint_half = if ay < paint_cap {
                    (paint_cap * paint_cap - ay * ay).max(0.0).sqrt()
                } else {
                    -1.0
                };
                let (paint_from, paint_to) = if paint_half >= 0.0 {
                    let u = qx_off + paint_half + 0.25;
                    (
                        (((cx - u - 0.5).floor() as i64).saturating_add(1)).clamp(x0, x1),
                        ((cx + u - 0.5).ceil() as i64).clamp(x0, x1),
                    )
                } else {
                    (x0, x0)
                };
                let (full_from, full_to) = if full_half >= 0.0 {
                    let u = (qx_off + full_half - 0.25).max(0.0);
                    (
                        ((cx - u - 0.5).ceil() as i64).clamp(x0, x1),
                        (((cx + u - 0.5).floor() as i64).saturating_add(1)).clamp(x0, x1),
                    )
                } else {
                    (x0, x0)
                };
                let bands = if full_from < full_to {
                    [(paint_from, full_from), (full_to, paint_to)]
                } else {
                    [(paint_from, paint_from), (paint_from, paint_to)]
                };
                for (from, to) in bands {
                    for x in from..to {
                        if edge_coverage(x as f32 + 0.5, cx, qx_off, qy, ay, rad) > 0.0 {
                            edge_painted = edge_painted.saturating_add(1);
                        }
                    }
                }
            }
            let span_w = u64::try_from(x1.saturating_sub(x0)).unwrap_or(0);
            let span_h = u64::try_from(y1.saturating_sub(y0)).unwrap_or(0);
            let walked = span_w.saturating_mul(span_h);
            profile::note_bucket("fill_round_rect.interior", interior, core_nanos);
            profile::note_bucket("fill_round_rect.edge_scanned", edge_scanned, edge_nanos);
            profile::note_bucket("fill_round_rect.edge_painted", edge_painted, 0);
            profile::note_bucket(
                "fill_round_rect.edge_zero",
                edge_scanned.saturating_sub(edge_painted),
                0,
            );
            profile::note_bucket(
                "fill_round_rect.outside_shape",
                walked.saturating_sub(interior).saturating_sub(edge_scanned),
                0,
            );
            profile::note_bucket("fill_round_rect.clipped", clipped_px, 0);
            profile::note_counter("fill_round_rect.calls", 1);
            profile::note_counter("fill_round_rect.translucent_calls", u64::from(!opaque));
            profile::note_counter("fill_round_rect.rows", rows_walked);
            profile::note_counter("fill_round_rect.zero_calls", u64::from(painted == 0));
            profile::note_max("fill_round_rect.max_call_px", painted);
        }
        profile::note_pixels("fill_round_rect", painted);
    }

    /// A one-pixel-wide (or thicker) rounded outline.
    pub fn stroke_round_rect(&mut self, r: Rect, radius: f32, width: f32, color: u32) {
        let _prof = profile::Timer::new("stroke_round_rect");
        let prof = profile::is_on();
        let half = width / 2.0;
        let (x0, y0, x1, y1) = self.span(&r.inflate(width + 1.5));
        let mut walked = 0u64;

        let hw = r.w / 2.0;
        let hh = r.h / 2.0;
        let rad = radius.min(hw).min(hh).max(0.0);
        let (cx, cy) = r.center();
        let qx_off = hw - rad;
        let qy_off = hh - rad;
        // Coverage is non-zero only where |sd| < half + 0.5, i.e. inside the
        // rounded rectangle grown by `band` and outside the one eroded by it.
        let band = half + 0.5;
        let usable = hw.is_finite()
            && hh.is_finite()
            && rad.is_finite()
            && band.is_finite()
            && half >= 0.0
            && hw > 0.0
            && hh > 0.0
            && radius.is_finite();
        if usable {
            for y in y0..y1 {
                let py = y as f32 + 0.5;
                let dy = (py - cy).abs();
                let outer = rounded_row_half_width(dy, hw + band, hh + band, rad + band);
                if outer < 0.0 {
                    continue;
                }
                // The part of the row that is provably inside the eroded shape:
                // its signed distance is at most -band, so coverage is 0 there.
                // Shrinking it by two pixels keeps the skip away from the
                // boundary, where floating-point rounding lives.
                let inner = if hw > band && hh > band {
                    rounded_row_half_width(dy, hw - band, hh - band, (rad - band).max(0.0))
                } else {
                    -1.0
                };
                let out_lo = (((cx - outer - 0.5).floor() as i64).saturating_sub(1)).clamp(x0, x1);
                let out_hi = (((cx + outer - 0.5).ceil() as i64).saturating_add(1)).clamp(x0, x1);
                if out_lo >= out_hi {
                    continue;
                }
                let (skip_lo, skip_hi) = if inner > 2.0 {
                    let safe = inner - 2.0;
                    let lo = ((cx - safe - 0.5).ceil() as i64).clamp(out_lo, out_hi);
                    let hi = ((cx + safe - 0.5).floor() as i64)
                        .saturating_add(1)
                        .clamp(out_lo, out_hi);
                    (lo, hi)
                } else {
                    (out_hi, out_hi)
                };
                let qy = dy - qy_off;
                let ay = qy.max(0.0);
                for (from, to) in [(out_lo, skip_lo), (skip_hi.max(out_lo), out_hi)] {
                    for x in from..to {
                        let qx = ((x as f32 + 0.5) - cx).abs() - qx_off;
                        let ax = qx.max(0.0);
                        let sd = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad;
                        self.blend_in(x, y, color, coverage(sd.abs() - half));
                    }
                }
            }
        } else {
            // A negative or non-finite width or radius cannot describe a band;
            // keep the original walk so those inputs behave exactly as before.
            for y in y0..y1 {
                for x in x0..x1 {
                    let d = sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius).abs() - half;
                    self.blend_in(x, y, color, coverage(d));
                }
            }
        }
        if prof {
            // Where the walked pixels of a stroke go. The stroke has no
            // interior worth filling: coverage is zero anywhere the distance
            // is further than half a pixel from the outline, yet the loop above
            // visits every pixel of the inflated rectangle. This pass uses the
            // identical expression so the buckets cannot disagree with the
            // painted frame, and runs only on the profiled frame.
            let (mut zero, mut partial, mut full, mut rows) = (0u64, 0u64, 0u64, 0u64);
            for y in y0..y1 {
                rows = rows.saturating_add(1);
                for x in x0..x1 {
                    walked = walked.saturating_add(1);
                    let d = sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius).abs() - half;
                    let cov = coverage(d);
                    if cov <= 0.0 {
                        zero = zero.saturating_add(1);
                    } else if cov >= 1.0 {
                        full = full.saturating_add(1);
                    } else {
                        partial = partial.saturating_add(1);
                    }
                }
            }
            profile::note_bucket("stroke_round_rect.zero", zero, 0);
            profile::note_bucket("stroke_round_rect.partial", partial, 0);
            profile::note_bucket("stroke_round_rect.full", full, 0);
            profile::note_bucket("stroke_round_rect.walked", walked, 0);
            profile::note_counter("stroke_round_rect.rows", rows);
            profile::note_counter("stroke_round_rect.calls", 1);
            profile::note_counter("stroke_round_rect.zero_calls", u64::from(zero == walked));
            profile::note_max("stroke_round_rect.max_call_px", walked);
        }
    }

    /// A filled circle.
    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: u32) {
        let r = Rect::new(cx - radius, cy - radius, radius * 2.0, radius * 2.0);
        self.fill_round_rect(r, radius, color);
    }

    /// An anti-aliased line of the given width, with round caps.
    pub fn stroke_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, color: u32) {
        let _prof = profile::Timer::new("stroke_line");
        let prof = profile::is_on();
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
        if prof {
            // The row timer already includes this pass, so the diagnostic walk
            // is timed as its own row: without the split, the instrumentation's
            // own cost reads as renderer cost.
            let _attrib = profile::Timer::new("stroke_line_attrib");
            // Where a line's walked pixels go. Same expression as the painter,
            // in an untimed pass, so the buckets cannot disagree with it. Icons
            // issue thousands of short strokes, so this also records how many
            // calls are axis-aligned, zero-length or entirely clipped away.
            let (mut walked, mut zero, mut partial, mut full) = (0u64, 0u64, 0u64, 0u64);
            for y in by0..by1 {
                for x in bx0..bx1 {
                    walked = walked.saturating_add(1);
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let t = if len2 <= f32::EPSILON {
                        0.0
                    } else {
                        (((px - x0) * dx + (py - y0) * dy) / len2).clamp(0.0, 1.0)
                    };
                    let qx = px - (x0 + dx * t);
                    let qy = py - (y0 + dy * t);
                    let cov = coverage((qx * qx + qy * qy).sqrt() - half);
                    if cov <= 0.0 {
                        zero = zero.saturating_add(1);
                    } else if cov >= 1.0 {
                        full = full.saturating_add(1);
                    } else {
                        partial = partial.saturating_add(1);
                    }
                }
            }
            let raw_w = (bounds.right().ceil() - bounds.x.floor()).max(0.0);
            let raw_h = (bounds.bottom().ceil() - bounds.y.floor()).max(0.0);
            let raw = (raw_w * raw_h) as u64;
            profile::note_counter("stroke_line.calls", 1);
            profile::note_counter(
                "stroke_line.zero_len_calls",
                u64::from(len2 <= f32::EPSILON),
            );
            profile::note_counter("stroke_line.axis_calls", u64::from(dx == 0.0 || dy == 0.0));
            // The two axis classes separately: a horizontal rule and a vertical
            // divider have different capsule geometry, and only counts this fine
            // can say whether an axis fast path would reach the hot calls.
            profile::note_counter(
                "stroke_line.h_axis_calls",
                u64::from(dy == 0.0 && dx != 0.0),
            );
            profile::note_counter(
                "stroke_line.v_axis_calls",
                u64::from(dx == 0.0 && dy != 0.0),
            );
            // A pen no wider than a pixel has no fully covered interior column,
            // so a solid-run fast path could never fire on it.
            profile::note_counter("stroke_line.thin_calls", u64::from(width <= 1.0));
            // The whole call painted nothing. A non-zero count is a call that a
            // cheaper exit could have skipped.
            profile::note_counter(
                "stroke_line.zero_paint_calls",
                u64::from(walked > 0 && zero == walked),
            );
            profile::note_counter(
                "stroke_line.hidden_calls",
                u64::from(by1 <= by0 || bx1 <= bx0),
            );
            profile::note_counter("stroke_line.len_px", len2.sqrt().round() as u64);
            profile::note_bucket("stroke_line.walked", walked, 0);
            profile::note_bucket("stroke_line.zero", zero, 0);
            profile::note_bucket("stroke_line.partial", partial, 0);
            profile::note_bucket("stroke_line.full", full, 0);
            profile::note_bucket("stroke_line.clipped", raw.saturating_sub(walked), 0);
            profile::note_counter("stroke_line.clipped_calls", u64::from(walked < raw));
            profile::note_max("stroke_line.max_call_px", walked);
        }
    }

    /// Composite a `w` x `h` bitmap of straight-alpha ARGB pixels with its
    /// top-left corner at `(x, y)`, inside the clip.
    ///
    /// The artwork the shell embeds (the dock's icons) goes through here: each
    /// pixel is blended over what is already on the canvas by its own alpha,
    /// times `alpha`, so a whole icon can fade in.
    pub fn draw_argb(&mut self, x: i64, y: i64, w: u32, h: u32, px: &[u32], alpha: f32) {
        let _prof = profile::Timer::new("draw_argb");
        // The span is the bitmap's box cut to the canvas and the clip, so a
        // partial repaint blends the artwork only inside its own region: drawn
        // again outside it, a soft edge would be blended over itself.
        let bounds = Rect::new(x as f32, y as f32, w as f32, h as f32);
        let (sx0, sy0, sx1, sy1) = self.span(&bounds);
        for py in sy0..sy1 {
            let row = (py.saturating_sub(y)) as usize;
            for pxl in sx0..sx1 {
                let col = (pxl.saturating_sub(x)) as usize;
                let Some(&c) = px.get(row.saturating_mul(w as usize).saturating_add(col)) else {
                    continue;
                };
                self.blend_in(pxl, py, c, alpha);
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
        let _prof = profile::Timer::new("glass_backdrop");
        // Which panel this call is. Recorded for BOTH branches, because the
        // frame under test has no backdrop attached and therefore runs the
        // live-blur branch -- §10's buckets sat only in the attached branch and
        // so never printed a row.
        let bucket = if r.w * r.h >= 160_000.0 {
            "glass_panel_l"
        } else if r.w * r.h >= 60_000.0 {
            "glass_panel_m"
        } else {
            "glass_panel_s"
        };
        let _panel = profile::Timer::new(bucket);
        let px_area = u64::from(r.w.max(0.0) as u32).saturating_mul(u64::from(r.h.max(0.0) as u32));
        profile::note_pixels(bucket, px_area);
        profile::note_dims(bucket, r.w.max(0.0) as u32, r.h.max(0.0) as u32);
        profile::note_rect(bucket, r);
        if let Some(source) = self.backdrop.clone() {
            let _attached = profile::Timer::new("glass_stage_attached");
            let (x0, y0, x1, y1) = self.span(&r.inflate(1.5));
            let prof = profile::is_on();
            let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
            let (cx, cy) = r.center();
            let qx_off = r.w / 2.0 - rad;
            let qy_off = r.h / 2.0 - rad;
            let full_cap = rad - 0.5;
            let paint_cap = rad + 0.5;
            let width = self.width as usize;
            // The same three per-row classes as `fill_round_rect` (§19). At
            // full coverage `blend(dst, source, 1.0)` is the source with an
            // opaque alpha whatever is under it, so the whole interior is an
            // opaque copy of the backdrop; only the one-pixel-class band where
            // the rounded outline crosses the row needs the distance and the
            // blend. The geometry is hoisted out of the pixel loop.
            //
            // An earlier core fast path here (§9) skipped the distance but
            // kept a per-pixel lookup and blend, and the frame gain was inside
            // its spread, so it was reverted. This one writes whole spans from
            // the source and has the same proof obligation as the fill.
            let bands_of = |y: i64| -> (i64, i64, i64, i64, f32, f32) {
                let py = y as f32 + 0.5;
                let qy = (py - cy).abs() - qy_off;
                let ay = qy.max(0.0);
                let full_half = if rad > 0.5 && ay <= full_cap {
                    (full_cap * full_cap - ay * ay).max(0.0).sqrt()
                } else if rad <= 0.5 && qy <= -0.5 {
                    rad - 0.5
                } else {
                    -1.0
                };
                let paint_half = if ay < paint_cap {
                    (paint_cap * paint_cap - ay * ay).max(0.0).sqrt()
                } else {
                    -1.0
                };
                let (paint_from, paint_to) = if paint_half >= 0.0 {
                    let u = qx_off + paint_half + 0.25;
                    (
                        (((cx - u - 0.5).floor() as i64).saturating_add(1)).clamp(x0, x1),
                        ((cx + u - 0.5).ceil() as i64).clamp(x0, x1),
                    )
                } else {
                    (x0, x0)
                };
                let (full_from, full_to) = if full_half >= 0.0 {
                    let u = (qx_off + full_half - 0.25).max(0.0);
                    (
                        ((cx - u - 0.5).ceil() as i64).clamp(x0, x1),
                        (((cx + u - 0.5).floor() as i64).saturating_add(1)).clamp(x0, x1),
                    )
                } else {
                    (x0, x0)
                };
                (paint_from, full_from, full_to, paint_to, qy, ay)
            };
            let mut full_px = 0u64;
            let mut full_nanos = 0u64;
            let mut partial_px = 0u64;
            let mut partial_nanos = 0u64;
            let mut rows_walked = 0u64;
            for y in y0..y1 {
                let base = (y as usize).saturating_mul(width);
                let (Some(row), Some(srow)) = (
                    self.px.get_mut(base..base.saturating_add(width)),
                    source.px.get(base..base.saturating_add(width)),
                ) else {
                    continue;
                };
                if prof {
                    rows_walked = rows_walked.saturating_add(1);
                }
                let (paint_from, full_from, full_to, paint_to, qy, ay) = bands_of(y);
                let (left_from, left_to, fill_from, fill_to, right_from, right_to) =
                    if full_from < full_to {
                        (paint_from, full_from, full_from, full_to, full_to, paint_to)
                    } else {
                        (
                            paint_from, paint_from, paint_from, paint_from, paint_from, paint_to,
                        )
                    };
                let copy_start = prof.then(std::time::Instant::now);
                if fill_from < fill_to {
                    if let (Some(dst), Some(src)) = (
                        row.get_mut(fill_from as usize..fill_to as usize),
                        srow.get(fill_from as usize..fill_to as usize),
                    ) {
                        for (d, src_px) in dst.iter_mut().zip(src) {
                            *d = 0xFF00_0000 | (src_px & 0x00FF_FFFF);
                        }
                    }
                }
                if let Some(start) = copy_start {
                    full_nanos = full_nanos.saturating_add(nanos_since(start));
                }
                let edge_start = prof.then(std::time::Instant::now);
                for (from, to) in [(left_from, left_to), (right_from, right_to)] {
                    for x in from..to {
                        let cov = edge_coverage(x as f32 + 0.5, cx, qx_off, qy, ay, rad);
                        if cov > 0.0 {
                            if let (Some(p), Some(v)) =
                                (row.get_mut(x as usize), srow.get(x as usize))
                            {
                                *p = blend(*p, *v, cov);
                            }
                        }
                    }
                }
                if let Some(start) = edge_start {
                    partial_nanos = partial_nanos.saturating_add(nanos_since(start));
                }
                full_px = full_px
                    .saturating_add(u64::try_from(fill_to.saturating_sub(fill_from)).unwrap_or(0));
                partial_px = partial_px.saturating_add(
                    u64::try_from(left_to.saturating_sub(left_from))
                        .unwrap_or(0)
                        .saturating_add(
                            u64::try_from(right_to.saturating_sub(right_from)).unwrap_or(0),
                        ),
                );
            }
            if prof {
                // Classify the partial band with the same expression the blend
                // uses, in an untimed pass; a counter may not disagree with the
                // painted frame.
                let mut partial_painted = 0u64;
                for y in y0..y1 {
                    let (paint_from, full_from, full_to, paint_to, qy, ay) = bands_of(y);
                    let bands = if full_from < full_to {
                        [(paint_from, full_from), (full_to, paint_to)]
                    } else {
                        [(paint_from, paint_from), (paint_from, paint_to)]
                    };
                    for (from, to) in bands {
                        for x in from..to {
                            if edge_coverage(x as f32 + 0.5, cx, qx_off, qy, ay, rad) > 0.0 {
                                partial_painted = partial_painted.saturating_add(1);
                            }
                        }
                    }
                }
                let span_w = u64::try_from(x1.saturating_sub(x0)).unwrap_or(0);
                let span_h = u64::try_from(y1.saturating_sub(y0)).unwrap_or(0);
                let walked = span_w.saturating_mul(span_h);
                profile::note_bucket("glass_attached.full", full_px, full_nanos);
                profile::note_bucket("glass_attached.partial_scanned", partial_px, partial_nanos);
                profile::note_bucket("glass_attached.partial_painted", partial_painted, 0);
                profile::note_bucket(
                    "glass_attached.partial_zero",
                    partial_px.saturating_sub(partial_painted),
                    0,
                );
                profile::note_bucket(
                    "glass_attached.skipped",
                    walked.saturating_sub(full_px).saturating_sub(partial_px),
                    0,
                );
                profile::note_counter("glass_attached.rows", rows_walked);
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
        {
            let _stage = profile::Timer::new("glass_stage_copy");
            for y in y0..y1 {
                for x in x0..x1 {
                    buf.push(self.get(x, y).unwrap_or(0xFF00_0000));
                }
            }
        }

        if blur > 0 {
            let _stage = profile::Timer::new("glass_stage_blur");
            let mut tmp = buf.clone();
            for _ in 0..2 {
                box_blur_pass(&buf, &mut tmp, w, h, blur as usize, true);
                box_blur_pass(&tmp, &mut buf, w, h, blur as usize, false);
            }
        }
        if (saturation - 1.0).abs() > f32::EPSILON {
            let _stage = profile::Timer::new("glass_stage_saturate");
            for p in &mut buf {
                *p = saturate_px(*p, saturation);
            }
        }

        let _stage = profile::Timer::new("glass_stage_mask_blend");
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
    /// The shadow is twelve concentric rounded rectangles, the outermost grown
    /// by 11/12 of `spread` and the whole stack moved down by `dy`. Twelve steps
    /// is enough that the banding is below one 8-bit level, and no second
    /// full-size buffer is needed for something that is only ever a dark halo.
    ///
    /// It is composed in one pass, not twelve. Growing a rounded rectangle by
    /// `g` and its radius by `g` moves its edge out by exactly `g`, so every ring
    /// shares one signed distance per pixel and differs only in the `g` it is
    /// compared with. Drawing the rings as twelve fills walked the whole outer
    /// rectangle twelve times and, for a window, composited the area *under* the
    /// window twelve times too (BUG-0015: shadow was 8.77M of 11.36M pixel
    /// visits in a three-window frame). Two things are skipped:
    ///
    /// * rings whose edge has not reached the pixel, which contribute nothing;
    /// * the interior of `r` (one pixel in from its edge), but **only when a
    ///   glass backdrop is attached**. Then the panel the shadow belongs to
    ///   replaces those pixels from the backdrop at full coverage, so a shadow
    ///   under it is never seen. Without a backdrop the glass path blurs what is
    ///   already on the canvas, shadow included, so the pixels under the panel
    ///   are part of its look and are kept. The one-pixel margin leaves the
    ///   anti-aliased edge, which is blended and does show what is under it.
    pub fn soft_shadow(&mut self, r: Rect, radius: f32, spread: f32, dy: f32, alpha: f32) {
        let _prof = profile::Timer::new("soft_shadow");
        const STEPS: usize = 12;
        let prev = self.set_site(Site::Shadow);

        // Per ring: how far its edge is grown, and the colour it is drawn in.
        // Index 0 is the innermost ring, which is drawn last, on top.
        let mut grow = [0.0f32; STEPS];
        let mut tint = [0u32; STEPS];
        for (i, (g, c)) in grow.iter_mut().zip(tint.iter_mut()).enumerate() {
            let t = i as f32 / STEPS as f32;
            *g = spread * t;
            // Squared falloff: a linear one leaves a visible outer edge where
            // the last ring stops, which reads as a rectangle rather than a
            // shadow.
            let a = alpha * (1.0 - t) * (1.0 - t) / STEPS as f32 * 3.0;
            *c = with_alpha(rgba(0, 0, 0, 255), a);
        }
        let reach = grow.last().copied().unwrap_or(0.0);
        // Rings are `spread / 12` apart and coverage ramps over one pixel, so at
        // most `ceil(1 / step) + 1` of them can straddle the edge at any pixel;
        // the rest contribute exactly zero and `blend_in` would return without
        // touching a byte. One window frame ran 949,831 of those no-op
        // iterations for 64,056 real blends. The extra margin of three rings
        // keeps the bound conservative under floating-point rounding.
        let step = spread / STEPS as f32;
        let keep = if step > 0.0 {
            ((1.0 / step).ceil() as usize).saturating_add(3).min(STEPS)
        } else {
            STEPS
        };

        // The shadow's own geometry, derived once. Both distance tests below
        // share the horizontal term and the clamped radius; before this, every
        // painted pixel recomputed the centre, both half-extents and the radius
        // for the panel test and again for the shadow test. That is four divides
        // a pixel for values that do not change inside the call.
        let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
        let (cx, cy) = r.center();
        let qx_off = r.w / 2.0 - rad;
        let qy_off = r.h / 2.0 - rad;
        let base_cy = cy + dy;
        let shadow = Rect::new(r.x, r.y + dy, r.w, r.h);
        let (x0, y0, x1, y1) = self.span(&shadow.inflate(reach + 1.5));

        // What a run of fully covered rings does to one channel value, built
        // with the same `blend` the loop below uses. Most pixels of a shadow
        // lie inside several of the twelve rings at coverage 1.0, and composing
        // those one blend at a time was measured at 46 ms of an 86 ms frame
        // (`render_bench`, 1280x800, 600x420 panel). The composite of a fixed
        // order of fixed alphas is a pure function of the value it starts from,
        // so `chain[n][v]` is what the outermost `n` rings do to `v`, and a
        // pixel those rings cover fully costs one lookup instead of `n` blends.
        // `chain` is built by calling `blend` itself, so the shortcut cannot
        // drift from the loop it replaces.
        let mut chain = [[0_u8; 256]; STEPS.saturating_add(1)];
        if let Some(row) = chain.first_mut() {
            for (v, slot) in row.iter_mut().enumerate() {
                *slot = u8::try_from(v).unwrap_or(u8::MAX);
            }
        }
        for n in 1..=STEPS {
            let ring = STEPS.saturating_sub(n);
            let Some(color) = tint.get(ring).copied() else {
                continue;
            };
            let prev = chain.get(n.saturating_sub(1)).copied().unwrap_or([0; 256]);
            let Some(row) = chain.get_mut(n) else {
                continue;
            };
            for v in 0..256_usize {
                let start = prev.get(v).copied().unwrap_or(0);
                let out = blend(rgb(u32::from(start)), color, 1.0) & 0xFF;
                if let Some(cell) = row.get_mut(v) {
                    *cell = u8::try_from(out).unwrap_or(u8::MAX);
                }
            }
        }

        // A rectangle that lies wholly inside `r`'s rounded outline, so the
        // common case -- the middle of a window -- is skipped a row at a time
        // without a distance calculation per pixel.
        let covered = self.backdrop.is_some();
        let inset = radius.max(2.0);
        let hide = Rect::new(
            r.x + inset,
            r.y + inset,
            r.w - inset * 2.0,
            r.h - inset * 2.0,
        );
        let (hx0, hy0, hx1, hy1) = if covered && hide.w > 0.0 && hide.h > 0.0 {
            (
                hide.x.ceil() as i64,
                hide.y.ceil() as i64,
                hide.right().floor() as i64,
                hide.bottom().floor() as i64,
            )
        } else {
            (0, 0, 0, 0)
        };

        for y in y0..y1 {
            let py = y as f32 + 0.5;
            // The panel's interior -- the part of the shadow the panel itself
            // replaces -- as one interval for the whole row. The eroded rounded
            // rectangle is where the per-pixel test below is guaranteed to
            // answer "under the panel", so its columns can be skipped without a
            // distance calculation each; a one-pixel margin either side keeps
            // the boundary on the exact per-pixel test.
            let (ix0, ix1) = if covered {
                let hwi = rounded_row_half_width(
                    (py - cy).abs(),
                    r.w / 2.0 - 1.0,
                    r.h / 2.0 - 1.0,
                    rad - 1.0,
                );
                if hwi > 1.0 {
                    (
                        (cx - hwi + 1.0).ceil() as i64,
                        (cx + hwi - 1.0).floor() as i64,
                    )
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };
            let mut x = x0;
            while x < x1 {
                if y >= hy0 && y < hy1 && x >= hx0 && x < hx1 {
                    x = hx1;
                    continue;
                }
                if x >= ix0 && x < ix1 {
                    x = ix1;
                    continue;
                }
                let px = x as f32 + 0.5;
                let qx = (px - cx).abs() - qx_off;
                let ax = qx.max(0.0);
                // Under the panel, past its first pixel: never visible.
                if covered {
                    let qy = (py - cy).abs() - qy_off;
                    let ay = qy.max(0.0);
                    let panel = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad;
                    if panel < -1.0 {
                        x = x.saturating_add(1);
                        continue;
                    }
                }
                let qy = (py - base_cy).abs() - qy_off;
                let ay = qy.max(0.0);
                let d = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad;
                // Outermost ring first, as it was drawn. Every ring whose grown
                // edge has already passed this pixel covers it fully, so those
                // are applied as one table lookup; the rings straddling the
                // edge are blended one at a time in the same order.
                let full_from = grow.iter().position(|g| *g >= d + 0.5).unwrap_or(STEPS);
                let full = STEPS.saturating_sub(full_from);
                if full > 0 {
                    let row = chain.get(full).copied().unwrap_or([0; 256]);
                    if let Some(p) = self.get(x, y) {
                        let r = row.get(((p >> 16) & 0xFF) as usize).copied().unwrap_or(0);
                        let g = row.get(((p >> 8) & 0xFF) as usize).copied().unwrap_or(0);
                        let b = row.get((p & 0xFF) as usize).copied().unwrap_or(0);
                        self.put_in(
                            x,
                            y,
                            0xFF00_0000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b),
                        );
                    }
                }
                for i in (full_from.saturating_sub(keep)..full_from).rev() {
                    let (Some(g), Some(c)) = (grow.get(i), tint.get(i)) else {
                        continue;
                    };
                    let cov = coverage(d - *g);
                    if cov > 0.0 {
                        self.blend_in(x, y, *c, cov);
                    }
                }
                x = x.saturating_add(1);
            }
        }
        if profile::is_on() {
            // Where a shadow's pixels go. This is a diagnostic pass over the
            // same predicates the loop above uses, and it runs only on the
            // profiled frame; no timed frame pays for it.
            let mut hidden_px = 0u64;
            let mut under_px = 0u64;
            let mut all_full_px = 0u64;
            let mut mixed_px = 0u64;
            let mut partial_px = 0u64;
            let mut zero_px = 0u64;
            let mut rows = 0u64;
            let mut iters = 0u64;
            let mut blends = 0u64;
            for y in y0..y1 {
                rows = rows.saturating_add(1);
                let py = y as f32 + 0.5;
                // The same interval the painter computes, from the same call:
                // a column skipped here is one the painter skipped too, so the
                // buckets cannot report a pixel as painted that was not.
                let (ix0, ix1) = if covered {
                    let hwi = rounded_row_half_width(
                        (py - cy).abs(),
                        r.w / 2.0 - 1.0,
                        r.h / 2.0 - 1.0,
                        rad - 1.0,
                    );
                    if hwi > 1.0 {
                        (
                            (cx - hwi + 1.0).ceil() as i64,
                            (cx + hwi - 1.0).floor() as i64,
                        )
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                };
                let mut x = x0;
                while x < x1 {
                    if y >= hy0 && y < hy1 && x >= hx0 && x < hx1 {
                        let n =
                            u64::try_from((hx1.min(x1)).saturating_sub(x.max(hx0))).unwrap_or(0);
                        hidden_px = hidden_px.saturating_add(n);
                        x = hx1;
                        continue;
                    }
                    if x >= ix0 && x < ix1 {
                        // The interval subsumes the inset rectangle, so the
                        // split is kept explicit: pixels also inside that
                        // rectangle are the ones the backdrop hides, and the
                        // rest is the interior the panel covers at its edge.
                        let a = x.max(ix0);
                        let b = ix1.min(x1);
                        let n = u64::try_from(b.saturating_sub(a)).unwrap_or(0);
                        let hid = if y >= hy0 && y < hy1 {
                            u64::try_from(b.min(hx1).saturating_sub(a.max(hx0))).unwrap_or(0)
                        } else {
                            0
                        };
                        hidden_px = hidden_px.saturating_add(hid);
                        under_px = under_px.saturating_add(n.saturating_sub(hid));
                        x = ix1;
                        continue;
                    }
                    let px = x as f32 + 0.5;
                    if covered && sd_round_rect(px, py, &r, radius) < -1.0 {
                        under_px = under_px.saturating_add(1);
                        x = x.saturating_add(1);
                        continue;
                    }
                    let d = sd_round_rect(px, py, &shadow, radius);
                    let full_from = grow.iter().position(|g| *g >= d + 0.5).unwrap_or(STEPS);
                    iters = iters.saturating_add(u64::try_from(full_from).unwrap_or(0));
                    for i in (0..full_from).rev() {
                        if grow.get(i).is_some_and(|g| coverage(d - *g) > 0.0) {
                            blends = blends.saturating_add(1);
                        }
                    }
                    if full_from == 0 {
                        all_full_px = all_full_px.saturating_add(1);
                    } else if full_from < STEPS {
                        mixed_px = mixed_px.saturating_add(1);
                    } else if (0..STEPS)
                        .rev()
                        .any(|i| grow.get(i).is_some_and(|g| coverage(d - *g) > 0.0))
                    {
                        partial_px = partial_px.saturating_add(1);
                    } else {
                        zero_px = zero_px.saturating_add(1);
                    }
                    x = x.saturating_add(1);
                }
            }
            let span_w = u64::try_from(x1.saturating_sub(x0)).unwrap_or(0);
            let span_h = u64::try_from(y1.saturating_sub(y0)).unwrap_or(0);
            let walked = span_w.saturating_mul(span_h);
            let visited = walked.saturating_sub(hidden_px);
            profile::note_bucket("soft_shadow.hidden", hidden_px, 0);
            profile::note_bucket("soft_shadow.under_panel", under_px, 0);
            profile::note_bucket("soft_shadow.all_full", all_full_px, 0);
            profile::note_bucket("soft_shadow.mixed", mixed_px, 0);
            profile::note_bucket("soft_shadow.partial_only", partial_px, 0);
            profile::note_bucket("soft_shadow.zero", zero_px, 0);
            profile::note_bucket("soft_shadow.walked", walked, 0);
            profile::note_bucket("soft_shadow.partial_iters", iters, 0);
            profile::note_bucket("soft_shadow.partial_blends", blends, 0);
            profile::note_counter("soft_shadow.rows", rows);
            profile::note_counter("soft_shadow.zero_calls", u64::from(visited == hidden_px));
            profile::note_max("soft_shadow.max_call_px", visited);
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
    fn the_attached_glass_matches_the_per_pixel_distance() {
        // The attached path used to be one distance and one blend per pixel
        // inside the rounded shape. This is that implementation verbatim; the
        // row-split path must leave every pixel equal for straight, fractional,
        // circular, over-large-radius, degenerate and clipped panels.
        fn reference(dst: &mut Canvas, source: &Canvas, r: Rect, radius: f32) {
            let (x0, y0, x1, y1) = dst.span(&r.inflate(1.5));
            for y in y0..y1 {
                for x in x0..x1 {
                    let cov = coverage(sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius));
                    if cov > 0.0 {
                        if let Some(v) = source.get(x, y) {
                            dst.blend_in(x, y, v, cov);
                        }
                    }
                }
            }
        }
        let source = Rc::new(textured(160, 120));
        for (r, radius) in [
            (Rect::new(10.0, 10.0, 80.0, 50.0), 0.0),
            (Rect::new(10.5, 9.25, 80.75, 50.5), 14.0),
            (Rect::new(20.0, 20.0, 100.0, 30.0), 15.0),
            (Rect::new(20.3, 20.7, 30.0, 30.0), 500.0),
            (Rect::new(50.0, 30.0, 40.0, 40.0), 20.0),
            (Rect::new(40.0, 40.0, 0.4, 60.0), 0.0),
            (Rect::new(30.0, 30.0, 0.0, 0.0), 0.0),
            (Rect::new(30.0, 30.0, -10.0, 20.0), 4.0),
            (Rect::new(-40.0, -30.0, 100.0, 80.0), 12.0),
            (Rect::new(5.0, 5.0, 150.0, 110.0), 24.0),
        ] {
            for clip in [
                None,
                Some(Rect::new(25.0, 15.0, 90.5, 70.0)),
                Some(Rect::new(0.0, 0.0, 1.0, 1.0)),
                Some(Rect::new(100.0, 80.0, 0.0, 0.0)),
            ] {
                let mut want = textured(160, 120);
                let mut got = textured(160, 120);
                for c in [&mut want, &mut got] {
                    let _ = c.set_clip(clip);
                    assert!(c.set_backdrop(Some(Rc::clone(&source))));
                }
                reference(&mut want, &source, r, radius);
                got.glass_backdrop(r, radius, 9, 1.6);
                assert_eq!(
                    want.pixels(),
                    got.pixels(),
                    "{r:?} radius {radius} clip {clip:?}"
                );
            }
        }
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

    impl Canvas {
        fn soft_shadow_reference(
            &mut self,
            r: Rect,
            radius: f32,
            spread: f32,
            dy: f32,
            alpha: f32,
        ) {
            let _prof = profile::Timer::new("soft_shadow");
            const STEPS: usize = 12;
            let prev = self.set_site(Site::Shadow);

            // Per ring: how far its edge is grown, and the colour it is drawn in.
            // Index 0 is the innermost ring, which is drawn last, on top.
            let mut grow = [0.0f32; STEPS];
            let mut tint = [0u32; STEPS];
            for (i, (g, c)) in grow.iter_mut().zip(tint.iter_mut()).enumerate() {
                let t = i as f32 / STEPS as f32;
                *g = spread * t;
                // Squared falloff: a linear one leaves a visible outer edge where
                // the last ring stops, which reads as a rectangle rather than a
                // shadow.
                let a = alpha * (1.0 - t) * (1.0 - t) / STEPS as f32 * 3.0;
                *c = with_alpha(rgba(0, 0, 0, 255), a);
            }
            let reach = grow.last().copied().unwrap_or(0.0);

            // The shadow's own geometry, derived once. Both distance tests below
            // share the horizontal term and the clamped radius; before this, every
            // painted pixel recomputed the centre, both half-extents and the radius
            // for the panel test and again for the shadow test. That is four divides
            // a pixel for values that do not change inside the call.
            let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
            let (cx, cy) = r.center();
            let qx_off = r.w / 2.0 - rad;
            let qy_off = r.h / 2.0 - rad;
            let base_cy = cy + dy;
            let shadow = Rect::new(r.x, r.y + dy, r.w, r.h);
            let (x0, y0, x1, y1) = self.span(&shadow.inflate(reach + 1.5));

            // What a run of fully covered rings does to one channel value, built
            // with the same `blend` the loop below uses. Most pixels of a shadow
            // lie inside several of the twelve rings at coverage 1.0, and composing
            // those one blend at a time was measured at 46 ms of an 86 ms frame
            // (`render_bench`, 1280x800, 600x420 panel). The composite of a fixed
            // order of fixed alphas is a pure function of the value it starts from,
            // so `chain[n][v]` is what the outermost `n` rings do to `v`, and a
            // pixel those rings cover fully costs one lookup instead of `n` blends.
            // `chain` is built by calling `blend` itself, so the shortcut cannot
            // drift from the loop it replaces.
            let mut chain = [[0_u8; 256]; STEPS.saturating_add(1)];
            if let Some(row) = chain.first_mut() {
                for (v, slot) in row.iter_mut().enumerate() {
                    *slot = u8::try_from(v).unwrap_or(u8::MAX);
                }
            }
            for n in 1..=STEPS {
                let ring = STEPS.saturating_sub(n);
                let Some(color) = tint.get(ring).copied() else {
                    continue;
                };
                let prev = chain.get(n.saturating_sub(1)).copied().unwrap_or([0; 256]);
                let Some(row) = chain.get_mut(n) else {
                    continue;
                };
                for v in 0..256_usize {
                    let start = prev.get(v).copied().unwrap_or(0);
                    let out = blend(rgb(u32::from(start)), color, 1.0) & 0xFF;
                    if let Some(cell) = row.get_mut(v) {
                        *cell = u8::try_from(out).unwrap_or(u8::MAX);
                    }
                }
            }

            // A rectangle that lies wholly inside `r`'s rounded outline, so the
            // common case -- the middle of a window -- is skipped a row at a time
            // without a distance calculation per pixel.
            let covered = self.backdrop.is_some();
            let inset = radius.max(2.0);
            let hide = Rect::new(
                r.x + inset,
                r.y + inset,
                r.w - inset * 2.0,
                r.h - inset * 2.0,
            );
            let (hx0, hy0, hx1, hy1) = if covered && hide.w > 0.0 && hide.h > 0.0 {
                (
                    hide.x.ceil() as i64,
                    hide.y.ceil() as i64,
                    hide.right().floor() as i64,
                    hide.bottom().floor() as i64,
                )
            } else {
                (0, 0, 0, 0)
            };

            for y in y0..y1 {
                let py = y as f32 + 0.5;
                let mut x = x0;
                while x < x1 {
                    if y >= hy0 && y < hy1 && x >= hx0 && x < hx1 {
                        x = hx1;
                        continue;
                    }
                    let px = x as f32 + 0.5;
                    let qx = (px - cx).abs() - qx_off;
                    let ax = qx.max(0.0);
                    // Under the panel, past its first pixel: never visible.
                    if covered {
                        let qy = (py - cy).abs() - qy_off;
                        let ay = qy.max(0.0);
                        let panel = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad;
                        if panel < -1.0 {
                            x = x.saturating_add(1);
                            continue;
                        }
                    }
                    let qy = (py - base_cy).abs() - qy_off;
                    let ay = qy.max(0.0);
                    let d = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad;
                    // Outermost ring first, as it was drawn. Every ring whose grown
                    // edge has already passed this pixel covers it fully, so those
                    // are applied as one table lookup; the rings straddling the
                    // edge are blended one at a time in the same order.
                    let full_from = grow.iter().position(|g| *g >= d + 0.5).unwrap_or(STEPS);
                    let full = STEPS.saturating_sub(full_from);
                    if full > 0 {
                        let row = chain.get(full).copied().unwrap_or([0; 256]);
                        if let Some(p) = self.get(x, y) {
                            let r = row.get(((p >> 16) & 0xFF) as usize).copied().unwrap_or(0);
                            let g = row.get(((p >> 8) & 0xFF) as usize).copied().unwrap_or(0);
                            let b = row.get((p & 0xFF) as usize).copied().unwrap_or(0);
                            self.put_in(
                                x,
                                y,
                                0xFF00_0000
                                    | (u32::from(r) << 16)
                                    | (u32::from(g) << 8)
                                    | u32::from(b),
                            );
                        }
                    }
                    for i in (0..full_from).rev() {
                        let (Some(g), Some(c)) = (grow.get(i), tint.get(i)) else {
                            continue;
                        };
                        let cov = coverage(d - *g);
                        if cov > 0.0 {
                            self.blend_in(x, y, *c, cov);
                        }
                    }
                    x = x.saturating_add(1);
                }
            }
            if profile::is_on() {
                // Where a shadow's pixels go. This is a diagnostic pass over the
                // same predicates the loop above uses, and it runs only on the
                // profiled frame; no timed frame pays for it.
                let mut hidden_px = 0u64;
                let mut under_px = 0u64;
                let mut all_full_px = 0u64;
                let mut mixed_px = 0u64;
                let mut partial_px = 0u64;
                let mut zero_px = 0u64;
                let mut rows = 0u64;
                for y in y0..y1 {
                    rows = rows.saturating_add(1);
                    let py = y as f32 + 0.5;
                    let mut x = x0;
                    while x < x1 {
                        if y >= hy0 && y < hy1 && x >= hx0 && x < hx1 {
                            let n = u64::try_from((hx1.min(x1)).saturating_sub(x.max(hx0)))
                                .unwrap_or(0);
                            hidden_px = hidden_px.saturating_add(n);
                            x = hx1;
                            continue;
                        }
                        let px = x as f32 + 0.5;
                        if covered && sd_round_rect(px, py, &r, radius) < -1.0 {
                            under_px = under_px.saturating_add(1);
                            x = x.saturating_add(1);
                            continue;
                        }
                        let d = sd_round_rect(px, py, &shadow, radius);
                        let full_from = grow.iter().position(|g| *g >= d + 0.5).unwrap_or(STEPS);
                        if full_from == 0 {
                            all_full_px = all_full_px.saturating_add(1);
                        } else if full_from < STEPS {
                            mixed_px = mixed_px.saturating_add(1);
                        } else if (0..STEPS)
                            .rev()
                            .any(|i| grow.get(i).is_some_and(|g| coverage(d - *g) > 0.0))
                        {
                            partial_px = partial_px.saturating_add(1);
                        } else {
                            zero_px = zero_px.saturating_add(1);
                        }
                        x = x.saturating_add(1);
                    }
                }
                let span_w = u64::try_from(x1.saturating_sub(x0)).unwrap_or(0);
                let span_h = u64::try_from(y1.saturating_sub(y0)).unwrap_or(0);
                let walked = span_w.saturating_mul(span_h);
                let visited = walked.saturating_sub(hidden_px);
                profile::note_bucket("soft_shadow.hidden", hidden_px, 0);
                profile::note_bucket("soft_shadow.under_panel", under_px, 0);
                profile::note_bucket("soft_shadow.all_full", all_full_px, 0);
                profile::note_bucket("soft_shadow.mixed", mixed_px, 0);
                profile::note_bucket("soft_shadow.partial_only", partial_px, 0);
                profile::note_bucket("soft_shadow.zero", zero_px, 0);
                profile::note_bucket("soft_shadow.walked", walked, 0);
                profile::note_counter("soft_shadow.rows", rows);
                profile::note_counter("soft_shadow.zero_calls", u64::from(visited == hidden_px));
                profile::note_max("soft_shadow.max_call_px", visited);
            }
            self.set_site(prev);
        }
    }

    /// The shadow as it was drawn before it was composed in one pass: twelve
    /// grown rounded rectangles, outermost first. Kept here as the reference the
    /// single-pass version must equal, pixel for pixel.
    fn twelve_ring_shadow(c: &mut Canvas, r: Rect, radius: f32, spread: f32, dy: f32, alpha: f32) {
        const STEPS: u32 = 12;
        for i in (0..STEPS).rev() {
            let t = f32::from(i as u16) / f32::from(STEPS as u16);
            let grow = spread * t;
            let a = alpha * (1.0 - t) * (1.0 - t) / f32::from(STEPS as u16) * 3.0;
            c.fill_round_rect(
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
    }

    /// `blend` as it was written before the integer rounding: the reference
    /// the faster one must equal for every input.
    fn blend_with_round(dst: u32, src: u32, extra: f32) -> u32 {
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

    #[test]
    fn integer_rounding_blends_exactly_as_float_rounding_did() {
        // A deterministic sweep, not a sample: every source alpha, a spread of
        // channel values at both ends and in the middle, and coverages that
        // land on, just under and just over a half.
        let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let extras = [
            0.0_f32, 0.001, 0.25, 0.4999, 0.5, 0.5001, 0.75, 0.999, 1.0, 1.5,
        ];
        for _ in 0..400_000 {
            let r = next();
            let dst = 0xFF00_0000 | (r as u32 & 0x00FF_FFFF);
            let src = (next() >> 11) as u32;
            let extra = extras
                .get((r >> 40) as usize % extras.len())
                .copied()
                .unwrap_or(0.0);
            assert_eq!(
                blend(dst, src, extra),
                blend_with_round(dst, src, extra),
                "dst {dst:#010x} src {src:#010x} extra {extra}"
            );
        }
        for a in 0..=255u32 {
            for v in [0u32, 1, 127, 128, 254, 255] {
                let src = (a << 24) | (v << 16) | (v << 8) | v;
                let dst = 0xFF00_0000 | ((255 - v) << 16) | (v << 8) | 17;
                assert_eq!(blend(dst, src, 1.0), blend_with_round(dst, src, 1.0));
            }
        }
    }

    #[test]
    fn mixing_two_frames_is_a_per_channel_cross_fade() {
        let mut a = Canvas::new(2, 1, rgb(0x00_00_00));
        let b = Canvas::new(2, 1, rgb(0xFF_80_20));
        a.mix_from(&b, 0.0);
        assert_eq!(a.get(0, 0), Some(rgb(0x00_00_00)));
        a.mix_from(&b, 1.0);
        assert_eq!(a.get(1, 0), Some(rgb(0xFF_80_20)));
        let mut half = Canvas::new(2, 1, rgb(0x00_00_00));
        half.mix_from(&b, 0.5);
        assert_eq!(half.get(0, 0), Some(rgb(0x80_40_10)));
    }

    #[test]
    fn with_alpha_rounds_the_way_it_always_did() {
        for i in 0..=10_000 {
            let alpha = i as f32 / 10_000.0;
            let want = (alpha.clamp(0.0, 1.0) * 255.0).round().clamp(0.0, 255.0) as u32;
            assert_eq!(with_alpha(0x00AB_CDEF, alpha) >> 24, want, "alpha {alpha}");
        }
    }

    #[test]
    fn the_filled_core_skips_the_distance_and_changes_no_pixel() {
        // Plain rectangles, pills, fractional edges and a radius larger than the
        // shape: each must leave the same pixels as filling every pixel with its
        // own distance.
        fn reference(c: &mut Canvas, r: Rect, radius: f32, colour: u32) {
            let (x0, y0, x1, y1) = c.span(&r.inflate(1.5));
            for y in y0..y1 {
                for x in x0..x1 {
                    let d = sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius);
                    c.blend_in(x, y, colour, coverage(d));
                }
            }
        }
        for (r, radius) in [
            (Rect::new(10.0, 10.0, 80.0, 50.0), 0.0),
            (Rect::new(10.5, 9.25, 80.75, 50.5), 14.0),
            (Rect::new(20.0, 20.0, 100.0, 30.0), 15.0),
            (Rect::new(20.3, 20.7, 30.0, 30.0), 500.0),
            (Rect::new(5.0, 5.0, 3.0, 3.0), 1.0),
            (Rect::new(40.0, 40.0, 0.4, 60.0), 0.0),
            (Rect::new(-10.0, -6.0, 60.0, 40.0), 9.0),
            // Degenerate: nothing to paint, and a rectangle entirely off the
            // canvas. The runs must be empty rather than write a stray pixel.
            (Rect::new(30.0, 30.0, 0.0, 0.0), 0.0),
            (Rect::new(30.0, 30.0, -10.0, 20.0), 4.0),
            (Rect::new(-200.0, -200.0, 50.0, 50.0), 8.0),
            // Large enough that the translucent core takes the lookup table,
            // and clipped by the canvas on two sides.
            (Rect::new(15.0, 12.0, 130.0, 100.0), 20.0),
            // A circle: radius equal to half the extent, so the full-coverage
            // span is a disc rather than a rectangle and the per-row boundary
            // is the circle's, not a straight edge.
            (Rect::new(50.0, 30.0, 40.0, 40.0), 20.0),
        ] {
            for colour in [
                rgb(0x336699),
                with_alpha(rgb(0xFFFFFF), 0.001),
                with_alpha(rgb(0xFFFFFF), 0.05),
                with_alpha(rgb(0xFFFFFF), 0.37),
                with_alpha(rgb(0x123456), 0.5),
                with_alpha(rgb(0x000000), 0.999),
                // A fully transparent source must leave every pixel alone.
                with_alpha(rgb(0xFF00FF), 0.0),
            ] {
                // No clip, a fractional partial clip, a one-pixel clip and an
                // empty clip: the reference clips through `span` and so must
                // the runs.
                for clip in [
                    None,
                    Some(Rect::new(25.0, 15.0, 90.5, 70.0)),
                    Some(Rect::new(0.0, 0.0, 1.0, 1.0)),
                    Some(Rect::new(100.0, 80.0, 0.0, 0.0)),
                ] {
                    let mut want = textured(160, 120);
                    let mut got = textured(160, 120);
                    let _ = want.set_clip(clip);
                    let _ = got.set_clip(clip);
                    reference(&mut want, r, radius, colour);
                    got.fill_round_rect(r, radius, colour);
                    assert_eq!(
                        want.pixels(),
                        got.pixels(),
                        "{r:?} radius {radius} colour {colour:#010x} clip {clip:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_fill_buckets_account_for_every_pixel_it_walks() {
        // The profiled fill reports disjoint slices. Every pixel whose
        // reference coverage is above zero must be either written by the
        // full-coverage span or visited by the partial band, and the buckets
        // must add up to the area the fill walked. A counter that disagrees
        // with the renderer would send the next optimization at the wrong
        // pixels.
        let r = Rect::new(15.0, 12.0, 130.0, 100.0);
        let radius = 20.0_f32;
        let colour = with_alpha(rgb(0x336699), 0.37);
        // The clip must reach the shape's left and top edges, or every clipped
        // row is fully covered and the partial band is legitimately empty.
        let clip = Some(Rect::new(10.0, 8.0, 100.0, 80.0));
        let mut c = textured(160, 120);
        let _ = c.set_clip(clip);
        let (x0, y0, x1, y1) = c.span(&r.inflate(1.5));
        let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
        let mut painted_ref = 0u64;
        for y in y0..y1 {
            for x in x0..x1 {
                if coverage(sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, rad)) > 0.0 {
                    painted_ref = painted_ref.saturating_add(1);
                }
            }
        }
        let walked = u64::try_from(x1 - x0).unwrap() * u64::try_from(y1 - y0).unwrap();
        profile::enable();
        let _ = profile::take_buckets();
        c.fill_round_rect(r, radius, colour);
        let buckets = profile::take_buckets();
        let _ = profile::take();
        let _ = profile::take_counters();
        profile::disable();
        let get = |name: &str| -> u64 {
            buckets
                .iter()
                .find(|b| b.name == name)
                .map_or(0, |b| b.pixels)
        };
        let interior = get("fill_round_rect.interior");
        let edge_painted = get("fill_round_rect.edge_painted");
        let edge_scanned = get("fill_round_rect.edge_scanned");
        assert_eq!(
            interior.saturating_add(edge_painted),
            painted_ref,
            "every painted pixel is accounted for once: {buckets:?}"
        );
        assert_eq!(
            get("fill_round_rect.edge_zero"),
            edge_scanned.saturating_sub(edge_painted),
            "{buckets:?}"
        );
        assert_eq!(
            get("fill_round_rect.outside_shape"),
            walked.saturating_sub(interior).saturating_sub(edge_scanned),
            "{buckets:?}"
        );
        assert!(
            interior > 0 && edge_painted > 0,
            "the test rectangle exercises both paths: {buckets:?}"
        );
    }

    #[test]
    fn the_bounded_ring_scan_equals_the_unbounded_one() {
        // The pre-change soft_shadow is kept verbatim above as
        // soft_shadow_reference. The matrix spans ring spacing from far wider
        // than the coverage ramp to far narrower, fractional geometry, radius
        // zero and larger than the half-extent, alpha 0 and 1, three clips and
        // both an attached and a detached backdrop.
        let cases = [
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 30.0, 12.0, 0.60),
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 30.5, 12.0, 0.60),
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 120.0, 12.0, 0.60),
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 12.0, 12.0, 0.60),
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 2.5, 12.0, 0.60),
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 0.5, 12.0, 0.60),
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 30.0, 12.0, 0.0),
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 30.0, 12.0, 1.0),
            (Rect::new(11.25, 9.75, 120.5, 70.25), 9.0, 22.0, 6.5, 0.5),
            (Rect::new(70.0, 40.0, 60.0, 60.0), 0.0, 18.0, 7.0, 0.4),
            (Rect::new(70.0, 40.0, 60.0, 60.0), 500.0, 24.0, 10.0, 0.35),
            (Rect::new(-40.0, -30.0, 100.0, 80.0), 12.0, 20.0, 8.0, 0.6),
        ];
        let clips = [
            None,
            Some(Rect::new(40.5, 30.25, 100.0, 70.0)),
            Some(Rect::new(0.0, 0.0, 0.0, 0.0)),
        ];
        for backdrop in [false, true] {
            for (rect, radius, spread, dy, alpha) in cases {
                for clip in clips {
                    let mut want = textured(260, 200);
                    let mut got = textured(260, 200);
                    if backdrop {
                        let source = Rc::new(textured(260, 200));
                        assert!(want.set_backdrop(Some(source.clone())));
                        assert!(got.set_backdrop(Some(source)));
                    }
                    let _ = want.set_clip(clip);
                    let _ = got.set_clip(clip);
                    want.soft_shadow_reference(rect, radius, spread, dy, alpha);
                    got.soft_shadow(rect, radius, spread, dy, alpha);
                    assert_eq!(
                        want.pixels(),
                        got.pixels(),
                        "{rect:?} r {radius} spread {spread} dy {dy} alpha {alpha} clip {clip:?} backdrop {backdrop}"
                    );
                }
            }
        }
    }

    /// `stroke_line` at fa9c17f, verbatim: one distance test and one blend for
    /// every pixel of the inflated bounds. Kept as the reference the current
    /// implementation must equal, pixel for pixel.
    fn stroke_line_reference(
        c: &mut Canvas,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        width: f32,
        color: u32,
    ) {
        let half = width / 2.0;
        let minx = x0.min(x1) - half - 1.5;
        let miny = y0.min(y1) - half - 1.5;
        let maxx = x0.max(x1) + half + 1.5;
        let maxy = y0.max(y1) + half + 1.5;
        let bounds = Rect::new(minx, miny, maxx - minx, maxy - miny);
        let (bx0, by0, bx1, by1) = c.span(&bounds);
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
                c.blend_in(x, y, color, coverage(d));
            }
        }
    }

    #[test]
    fn the_stroke_line_equals_the_per_pixel_distance() {
        // Horizontal, vertical, diagonal, fractional, zero-length, off-canvas,
        // single-pixel, hairline and thick pens, over four clips (including an
        // empty one) and opaque, translucent and zero-alpha colours. Every
        // pixel of both canvases must agree.
        let cases: [(f32, f32, f32, f32, f32); 12] = [
            (30.0, 30.0, 90.0, 30.0, 1.0),
            (30.0, 30.0, 30.0, 90.0, 2.0),
            (30.0, 30.0, 90.0, 70.0, 1.0),
            (30.25, 30.75, 90.5, 70.25, 1.5),
            (50.0, 40.0, 50.0, 40.0, 1.0),
            (50.0, 40.0, 50.0, 40.0, 9.0),
            (60.0, 60.0, 60.5, 60.0, 0.5),
            (-40.0, -20.0, 200.0, 160.0, 3.0),
            (40.0, 150.0, 40.0, -50.0, 2.0),
            (10.0, 10.0, 200.0, 10.0, 12.0),
            (80.0, 20.0, 21.5, 140.5, 7.0),
            (f32::NAN, 10.0, 40.0, 10.0, 1.0),
        ];
        let clips = [
            None,
            Some(Rect::new(30.5, 20.25, 100.0, 70.0)),
            Some(Rect::new(55.0, 45.0, 1.0, 1.0)),
            Some(Rect::new(0.0, 0.0, 0.0, 0.0)),
        ];
        let colors = [
            rgb(0xFFFFFF),
            with_alpha(rgb(0x3A7BD5), 0.35),
            with_alpha(rgb(0xFFFFFF), 0.0),
        ];
        let mut compared = 0_u32;
        for (x0, y0, x1, y1, width) in cases {
            for clip in clips {
                for color in colors {
                    let mut want = textured(220, 180);
                    let mut got = textured(220, 180);
                    let _ = want.set_clip(clip);
                    let _ = got.set_clip(clip);
                    stroke_line_reference(&mut want, x0, y0, x1, y1, width, color);
                    got.stroke_line(x0, y0, x1, y1, width, color);
                    assert_eq!(
                        want.pixels(),
                        got.pixels(),
                        "{x0},{y0} -> {x1},{y1} width {width} clip {clip:?} colour {color:#010X}"
                    );
                    compared = compared.saturating_add(1);
                }
            }
        }
        assert_eq!(compared, 144, "every case in the matrix was compared");
    }

    #[test]
    fn the_shadow_panel_interval_changes_no_pixel() {
        // `soft_shadow_reference` is the implementation before the ring bound
        // and before this row interval: it applies the per-pixel panel test to
        // every pixel of the shadow's box. The interval must skip exactly the
        // pixels that test would have thrown away, so both canvases agree on
        // every pixel, with and without a backdrop attached (the backdrop is
        // what makes the panel hide its own shadow), across clips including an
        // empty one and spreads from 0 to 36.
        let cases: [(Rect, f32, f32, f32, f32); 11] = [
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 30.0, 12.0, 0.60),
            (Rect::new(40.0, 120.0, 150.0, 30.0), 15.0, 40.0, 14.0, 0.45),
            (Rect::new(70.0, 40.0, 60.0, 60.0), 40.0, 24.0, 10.0, 0.35),
            (Rect::new(10.5, 20.25, 120.0, 70.0), 0.0, 18.0, 7.0, 0.40),
            (Rect::new(30.0, 30.0, 100.0, 80.0), 8.0, 0.0, 0.0, 0.5),
            (Rect::new(30.0, 30.0, 100.0, 80.0), 8.0, 0.5, 4.0, 0.5),
            (Rect::new(30.0, 30.0, 100.0, 80.0), 8.0, 1.0, 4.0, 1.0),
            (Rect::new(30.0, 30.0, 100.0, 80.0), 500.0, 36.0, 10.0, 0.5),
            (Rect::new(20.0, 20.0, 0.0, 0.0), 4.0, 12.0, 6.0, 0.5),
            (Rect::new(30.0, 30.0, 60.0, 40.0), 6.0, 36.0, 10.0, 0.0),
            (Rect::new(-40.0, -30.0, 120.0, 90.0), 12.0, 36.0, 10.0, 0.6),
        ];
        let clips = [
            None,
            Some(Rect::new(35.0, 25.0, 90.0, 70.0)),
            Some(Rect::new(0.0, 0.0, 0.0, 0.0)),
        ];
        let mut compared = 0_u32;
        for (r, radius, spread, dy, alpha) in cases {
            for clip in clips {
                for backdrop in [false, true] {
                    let mut want = textured(220, 180);
                    let mut got = textured(220, 180);
                    let _ = want.set_clip(clip);
                    let _ = got.set_clip(clip);
                    if backdrop {
                        let source = std::rc::Rc::new(textured(220, 180));
                        let _ = want.set_backdrop(Some(std::rc::Rc::clone(&source)));
                        let _ = got.set_backdrop(Some(source));
                    }
                    want.soft_shadow_reference(r, radius, spread, dy, alpha);
                    got.soft_shadow(r, radius, spread, dy, alpha);
                    assert_eq!(
                        want.pixels(),
                        got.pixels(),
                        "{r:?} radius {radius} spread {spread} dy {dy} alpha {alpha} \
                         clip {clip:?} backdrop {backdrop}"
                    );
                    compared = compared.saturating_add(1);
                }
            }
        }
        assert_eq!(compared, 66, "every case in the matrix was compared");
    }

    #[test]
    fn the_one_pass_shadow_equals_the_twelve_ring_shadow() {
        // Panels the shell really draws: a window, a pill dock, a dialog.
        for (r, radius, spread, dy, alpha) in [
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 30.0, 12.0, 0.60),
            (Rect::new(40.0, 120.0, 150.0, 30.0), 15.0, 40.0, 14.0, 0.45),
            (Rect::new(70.0, 40.0, 60.0, 60.0), 40.0, 24.0, 10.0, 0.35),
            (Rect::new(10.5, 20.25, 120.0, 70.0), 0.0, 18.0, 7.0, 0.40),
        ] {
            let mut want = textured(260, 200);
            let mut got = textured(260, 200);
            twelve_ring_shadow(&mut want, r, radius, spread, dy, alpha);
            got.soft_shadow(r, radius, spread, dy, alpha);
            assert_eq!(
                want.pixels(),
                got.pixels(),
                "shadow differs for {r:?} radius {radius} spread {spread}"
            );
        }
    }

    fn stroke_round_rect_reference(c: &mut Canvas, r: Rect, radius: f32, width: f32, color: u32) {
        // The implementation at f1c174d, verbatim: every pixel of the inflated
        // rectangle gets a distance test and a blend. Only used by the test.
        let half = width / 2.0;
        let (x0, y0, x1, y1) = c.span(&r.inflate(width + 1.5));
        for y in y0..y1 {
            for x in x0..x1 {
                let d = sd_round_rect(x as f32 + 0.5, y as f32 + 0.5, &r, radius).abs() - half;
                c.blend_in(x, y, color, coverage(d));
            }
        }
    }

    #[test]
    fn the_stroke_band_walk_equals_the_full_distance_walk() {
        // Every pixel of both canvases must agree. The matrix covers fractional
        // geometry, radius 0 / half the extent / larger than it, thin and thick
        // pens, a circle, a one-pixel rectangle, negative and zero extents, a
        // rectangle off the canvas, four clips (including empty) and opaque,
        // translucent and zero-alpha colours.
        let cases = [
            (Rect::new(40.0, 30.0, 140.0, 100.0), 14.0, 1.0),
            (Rect::new(40.25, 29.75, 140.5, 100.25), 9.0, 1.0),
            (Rect::new(60.0, 50.0, 100.0, 80.0), 0.0, 1.0),
            (Rect::new(60.0, 50.0, 100.0, 80.0), 50.0, 1.0),
            (Rect::new(60.0, 50.0, 100.0, 80.0), 500.0, 1.0),
            (Rect::new(60.0, 50.0, 100.0, 80.0), 20.0, 0.5),
            (Rect::new(60.0, 50.0, 100.0, 80.0), 20.0, 9.0),
            (Rect::new(70.0, 50.0, 80.0, 80.0), 40.0, 1.0),
            (Rect::new(80.5, 50.0, 1.0, 70.0), 0.5, 1.0),
            (Rect::new(30.0, 30.0, 0.0, 0.0), 4.0, 1.0),
            (Rect::new(30.0, 30.0, -10.0, 20.0), 4.0, 1.0),
            (Rect::new(-40.0, -30.0, 100.0, 80.0), 12.0, 2.0),
            (Rect::new(-20.0, -20.0, 300.0, 260.0), 30.0, 1.0),
        ];
        let clips = [
            None,
            Some(Rect::new(30.5, 20.25, 100.0, 70.0)),
            Some(Rect::new(55.0, 45.0, 1.0, 1.0)),
            Some(Rect::new(0.0, 0.0, 0.0, 0.0)),
        ];
        let colors = [
            rgb(0xFFFFFF),
            with_alpha(rgb(0x3A7BD5), 0.35),
            with_alpha(rgb(0xFFFFFF), 0.0),
        ];
        for (rect, radius, width) in cases {
            for clip in clips {
                for color in colors {
                    let mut want = textured(220, 180);
                    let mut got = textured(220, 180);
                    let _ = want.set_clip(clip);
                    let _ = got.set_clip(clip);
                    stroke_round_rect_reference(&mut want, rect, radius, width, color);
                    got.stroke_round_rect(rect, radius, width, color);
                    assert_eq!(
                        want.pixels(),
                        got.pixels(),
                        "{rect:?} radius {radius} width {width} clip {clip:?} colour {color:#010X}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_hoisted_shadow_geometry_equals_the_distance_function() {
        // soft_shadow now derives the centre, half-extents and clamped radius
        // once per call instead of once per pixel, and shares the horizontal
        // term between the two distance tests. This pins the inlined math to
        // sd_round_rect bit for bit over every pixel-class sample of every
        // geometry, including fractional, degenerate and off-canvas rectangles.
        let cases: [(Rect, f32, f32); 8] = [
            (Rect::new(60.0, 50.0, 140.0, 90.0), 14.0, 12.0),
            (Rect::new(11.25, 9.75, 120.5, 70.25), 9.0, 6.5),
            (Rect::new(70.0, 40.0, 60.0, 60.0), 40.0, 10.0),
            (Rect::new(70.0, 40.0, 60.0, 60.0), 500.0, 10.0),
            (Rect::new(10.5, 20.25, 120.0, 70.0), 0.0, 7.0),
            (Rect::new(30.0, 30.0, 0.0, 0.0), 4.0, 8.0),
            (Rect::new(30.0, 30.0, -10.0, 20.0), 4.0, 8.0),
            (Rect::new(-40.0, -30.0, 100.0, 80.0), 12.0, 8.0),
        ];
        for (rect, radius, dy) in cases {
            let shadow = Rect::new(rect.x, rect.y + dy, rect.w, rect.h);
            let rad = radius.min(rect.w / 2.0).min(rect.h / 2.0).max(0.0);
            let (cx, cy) = rect.center();
            let (qx_off, qy_off) = (rect.w / 2.0 - rad, rect.h / 2.0 - rad);
            let mut y = -10.0f32;
            while y < 190.0 {
                let py = y + 0.5;
                let mut x = -10.0f32;
                while x < 240.0 {
                    let px = x + 0.5;
                    let qx = (px - cx).abs() - qx_off;
                    let ax = qx.max(0.0);
                    let qy = (py - cy).abs() - qy_off;
                    let ay = qy.max(0.0);
                    let panel = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad;
                    assert_eq!(
                        panel.to_bits(),
                        sd_round_rect(px, py, &rect, radius).to_bits(),
                        "{rect:?} radius {radius} at ({px}, {py})"
                    );
                    let qy = (py - (cy + dy)).abs() - qy_off;
                    let ay = qy.max(0.0);
                    let d = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - rad;
                    assert_eq!(
                        d.to_bits(),
                        sd_round_rect(px, py, &shadow, radius).to_bits(),
                        "{shadow:?} radius {radius} at ({px}, {py})"
                    );
                    x += 1.0;
                }
                y += 1.0;
            }
        }
    }

    #[test]
    fn the_shadow_buckets_account_for_every_walked_pixel() {
        // The diagnostic split must be disjoint and complete: the bucket areas
        // add up to the rectangle the call walked, and both the covered and the
        // visible paths are exercised.
        let r = Rect::new(60.0, 50.0, 140.0, 90.0);
        let mut c = textured(260, 200);
        assert!(c.set_backdrop(Some(Rc::new(textured(260, 200)))));
        profile::enable();
        let _ = profile::take_buckets();
        let _ = profile::take();
        c.soft_shadow(r, 14.0, 30.0, 12.0, 0.6);
        let buckets = profile::take_buckets();
        let _ = profile::take();
        let _ = profile::take_counters();
        profile::disable();
        let get = |name: &str| -> u64 {
            buckets
                .iter()
                .find(|b| b.name == name)
                .map_or(0, |b| b.pixels)
        };
        let walked = get("soft_shadow.walked");
        let sum = get("soft_shadow.hidden")
            + get("soft_shadow.under_panel")
            + get("soft_shadow.all_full")
            + get("soft_shadow.mixed")
            + get("soft_shadow.partial_only")
            + get("soft_shadow.zero");
        assert_eq!(sum, walked, "{buckets:?}");
        assert!(get("soft_shadow.hidden") > 0, "{buckets:?}");
        assert!(get("soft_shadow.mixed") > 0, "{buckets:?}");
    }

    #[test]
    fn a_shadow_under_an_opaque_panel_with_a_backdrop_leaves_the_same_frame() {
        // With a backdrop attached the shadow skips the pixels under its own
        // panel. That is only allowed if the panel then covers them completely,
        // so the finished frame -- shadow, then the panel -- must match.
        let r = Rect::new(60.0, 50.0, 140.0, 90.0);
        let radius = 14.0;
        let backdrop = std::rc::Rc::new(textured(260, 200));
        let paint_panel = |c: &mut Canvas| {
            c.glass_backdrop(r, radius, 3, 1.0);
            c.fill_round_rect(r, radius, with_alpha(rgb(0x101820), 0.6));
        };
        let mut want = textured(260, 200);
        assert!(want.set_backdrop(Some(backdrop.clone())));
        twelve_ring_shadow(&mut want, r, radius, 30.0, 12.0, 0.6);
        paint_panel(&mut want);

        let mut got = textured(260, 200);
        assert!(got.set_backdrop(Some(backdrop)));
        got.soft_shadow(r, radius, 30.0, 12.0, 0.6);
        paint_panel(&mut got);
        assert_eq!(want.pixels(), got.pixels());
    }

    #[test]
    fn the_one_pass_shadow_visits_far_fewer_pixels_under_a_window() {
        let r = Rect::new(60.0, 50.0, 140.0, 90.0);
        let backdrop = std::rc::Rc::new(textured(260, 200));
        let mut c = textured(260, 200);
        assert!(c.set_backdrop(Some(backdrop)));
        c.reset_pixels_visited();
        c.soft_shadow(r, 14.0, 30.0, 12.0, 0.6);
        let one_pass = c.pixels_visited();
        let mut d = textured(260, 200);
        d.reset_pixels_visited();
        let prev = d.set_site(Site::Shadow);
        twelve_ring_shadow(&mut d, r, 14.0, 30.0, 12.0, 0.6);
        d.set_site(prev);
        assert!(
            one_pass * 3 < d.pixels_visited(),
            "{one_pass} vs {}",
            d.pixels_visited()
        );
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

    #[test]
    fn a_client_frame_is_scaled_to_the_window_and_xrgb_is_made_opaque() {
        let mut canvas = Canvas::new(8, 8, 0);
        // A 2x2 client frame: one red, one green, one blue, one white.
        let pixels = [
            0, 0, 255, 0, // blue, alpha 0 (padding in XRGB)
            0, 255, 0, 0, // green
            255, 0, 0, 0, // red
            255, 255, 255, 0,
        ];
        canvas.blit_bgra(Rect::new(0.0, 0.0, 8.0, 8.0), &pixels, 2, 2, true);
        assert_eq!(
            canvas.get(0, 0),
            Some(0xFFFF_0000),
            "top-left is blue, opaque"
        );
        assert_eq!(canvas.get(7, 0), Some(0xFF00_FF00), "top-right is green");
        assert_eq!(canvas.get(0, 7), Some(0xFF00_00FF), "bottom-left is red");
        assert_eq!(canvas.get(7, 7), Some(0xFFFF_FFFF));
        // Every pixel is opaque even though the client sent zeros.
        assert!(
            (0..8).all(|y| (0..8).all(|x| canvas.get(x, y).is_some_and(|p| p >> 24 == 0xFF))),
            "XRGB alpha is forced opaque"
        );
    }

    #[test]
    fn a_client_frame_never_paints_outside_its_window() {
        let mut canvas = Canvas::new(16, 16, 0);
        let pixels = [0u8, 0, 255, 255];
        canvas.blit_bgra(Rect::new(4.0, 4.0, 4.0, 4.0), &pixels, 1, 1, false);
        assert_eq!(canvas.get(3, 4), Some(0), "left of the window is untouched");
        assert_eq!(canvas.get(4, 4), Some(0xFFFF_0000));
        assert_eq!(canvas.get(7, 7), Some(0xFFFF_0000));
        assert_eq!(
            canvas.get(8, 8),
            Some(0),
            "right of the window is untouched"
        );
        // A zero-sized or oversized mapping is not a crash.
        canvas.blit_bgra(Rect::new(0.0, 0.0, 0.0, 0.0), &pixels, 1, 1, false);
        canvas.blit_bgra(Rect::new(0.0, 0.0, 4.0, 4.0), &[], 0, 0, false);
        assert_eq!(canvas.get(0, 0), Some(0));
    }
}
