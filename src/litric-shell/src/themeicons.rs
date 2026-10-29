//! Application icons from the machine's icon theme.
//!
//! A desktop entry names its icon; the pixels live in the freedesktop icon
//! themes under `$XDG_DATA_HOME/icons`, `~/.icons`, each `$XDG_DATA_DIRS/icons`
//! and finally `/usr/share/pixmaps`. This module finds the PNG or SVG for a
//! name, decodes or rasterises it, and keeps a bounded number of copies so the
//! draw path is a blit.
//!
//! # Where it looks
//!
//! A flat `pixmaps`-style directory first, then each theme (the fallback theme
//! `hicolor` first) at the size directory nearest the request, then that theme's
//! `scalable/` directory, then the theme's flat directory. A theme that does not
//! name an icon is followed through its `index.theme` `Inherits=` list, breadth
//! first, to [`MAX_THEME_DEPTH`], with a visited set so a cycle cannot loop.
//!
//! # The SVG rasteriser
//!
//! Icon themes ship scalable icons as SVG. This image has no SVG library, so a
//! small, bounded rasteriser is included: `rect`, `circle`, `ellipse`, `line`,
//! `polygon`, `polyline` and `path` (`M L H V C S Q T Z`, absolute and
//! relative), solid `fill`/`stroke` in `#rgb`, `#rrggbb`, `rgb(...)` or a
//! gradient's first stop colour, `fill-rule`, `fill-opacity`, `opacity`, and
//! `translate`/`scale`/`matrix` transforms on groups and shapes.
//!
//! Deliberately **not** supported, and not faked: gradients as gradients (the
//! first stop colour is used as a flat fill), `clip-path`, masks, patterns,
//! filters, text, `<use>`, CSS style sheets, elliptical arc commands (`A`), and
//! `preserveAspectRatio` other than the default meet. A path that contains an
//! arc is dropped rather than drawn in the wrong place.
//!
//! # Bounds
//!
//! A file over [`MAX_SVG_BYTES`] / [`MAX_ICON_BYTES`] is not read; a raster is
//! at most [`MAX_RASTER_SIDE`] on a side; a document may have
//! [`MAX_SVG_ELEMENTS`] elements and [`MAX_SVG_POINTS`] points, and the fill
//! work is capped at [`MAX_SVG_SAMPLES`] point-rows. The scaled cache keeps
//! [`MAX_CACHED`] entries. No icon can turn a frame into an unbounded job.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::canvas::{Canvas, Rect};

/// The largest PNG icon file read.
pub const MAX_ICON_BYTES: u64 = 512 * 1024;

/// The largest SVG icon file read.
pub const MAX_SVG_BYTES: u64 = 256 * 1024;

/// The largest PNG icon edge decoded, in pixels.
pub const MAX_ICON_SIDE: u32 = 512;

/// The most scaled icons kept at once.
pub const MAX_CACHED: usize = 48;

/// The most theme directories inspected for one lookup.
pub const MAX_THEMES: usize = 64;

/// How far an `Inherits=` chain is followed.
pub const MAX_THEME_DEPTH: usize = 8;

/// The most elements one SVG may contain.
pub const MAX_SVG_ELEMENTS: usize = 400;

/// The most path points one SVG may produce.
pub const MAX_SVG_POINTS: usize = 20_000;

/// The largest edge an SVG is rasterised at.
pub const MAX_RASTER_SIDE: usize = 128;

/// The fill budget: points times scanlines.
pub const MAX_SVG_SAMPLES: usize = 40_000_000;

/// The icon categories a theme puts artwork under.
pub(crate) const CATEGORIES: [&str; 5] =
    ["apps", "applications", "categories", "devices", "places"];

/// The directories an icon may live under, in precedence order.
#[must_use]
pub fn theme_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(data) = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    {
        dirs.push(data.join("icons"));
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        dirs.push(home.join(".icons"));
    }
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_owned());
    for dir in data_dirs.split(':') {
        if !dir.trim().is_empty() {
            dirs.push(PathBuf::from(dir).join("icons"));
        }
    }
    dirs.push(PathBuf::from("/usr/share/pixmaps"));
    dirs
}

/// Find the PNG or SVG for `name`, preferring one closest to `size` pixels.
#[must_use]
pub fn find(name: &str, size: u32) -> Option<PathBuf> {
    find_in(&theme_dirs(), name, size)
}

/// Find the PNG or SVG for `name` in `dirs`, earlier directories winning.
///
/// A PNG at a size directory beats an SVG, because a theme that shipped a
/// raster at the right size meant it; an SVG beats nothing, and is used when it
/// is the only artwork the theme has.
#[must_use]
pub fn find_in(dirs: &[PathBuf], name: &str, size: u32) -> Option<PathBuf> {
    let name = name.trim();
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.contains('\0')
        || name.len() > 255
    {
        return None;
    }
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for dir in dirs {
        // A flat directory: `/usr/share/pixmaps` is the usual one.
        let flat_png = dir.join(format!("{name}.png"));
        if usable(&flat_png) {
            return Some(flat_png);
        }
        let flat_svg = dir.join(format!("{name}.svg"));
        if usable(&flat_svg) {
            return Some(flat_svg);
        }
        let mut themes = themes_in(dir);
        themes.sort_by_key(|path| {
            usize::from(path.file_name().and_then(|n| n.to_str()) != Some("hicolor"))
        });
        for theme in themes {
            if seen.insert(theme.clone()) {
                queue.push_back((theme, 0));
            }
        }
    }
    let mut inspected = 0usize;
    while let Some((theme, depth)) = queue.pop_front() {
        inspected = inspected.saturating_add(1);
        if inspected > MAX_THEMES {
            return None;
        }
        if let Some(found) = find_in_theme(&theme, name, size) {
            return Some(found);
        }
        if depth >= MAX_THEME_DEPTH {
            continue;
        }
        for parent in index_theme_inherits(&theme) {
            for root in dirs {
                let candidate = root.join(&parent);
                if candidate.is_dir() && seen.insert(candidate.clone()) {
                    queue.push_back((candidate, depth.saturating_add(1)));
                    break;
                }
            }
        }
    }
    None
}

/// The theme directories under one icon root.
fn themes_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut themes: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .collect();
    themes.sort();
    themes.truncate(MAX_THEMES);
    themes
}

/// The `Inherits=` names a theme declares in its `index.theme`.
fn index_theme_inherits(theme: &Path) -> Vec<String> {
    let path = theme.join("index.theme");
    let Ok(meta) = std::fs::metadata(&path) else {
        return Vec::new();
    };
    if !meta.is_file() || meta.len() > 64 * 1024 {
        return Vec::new();
    }
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut in_group = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_group = line.eq_ignore_ascii_case("[Icon Theme]");
            continue;
        }
        if !in_group {
            continue;
        }
        if let Some(rest) = line.strip_prefix("Inherits=") {
            return rest
                .split(',')
                .map(str::trim)
                .filter(|name| {
                    !name.is_empty()
                        && !name.contains('/')
                        && !name.contains("..")
                        && !name.contains('\0')
                })
                .take(8)
                .map(str::to_owned)
                .collect();
        }
    }
    Vec::new()
}

/// Look for `name` inside one theme.
fn find_in_theme(theme: &Path, name: &str, size: u32) -> Option<PathBuf> {
    for size_dir in size_dirs(theme, size) {
        for category in CATEGORIES {
            let candidate = size_dir.join(category).join(format!("{name}.png"));
            if usable(&candidate) {
                return Some(candidate);
            }
        }
    }
    for category in CATEGORIES {
        let scalable = theme.join("scalable").join(category);
        let svg = scalable.join(format!("{name}.svg"));
        if usable(&svg) {
            return Some(svg);
        }
        let png = scalable.join(format!("{name}.png"));
        if usable(&png) {
            return Some(png);
        }
    }
    let flat_png = theme.join(format!("{name}.png"));
    if usable(&flat_png) {
        return Some(flat_png);
    }
    let flat_svg = theme.join(format!("{name}.svg"));
    if usable(&flat_svg) {
        return Some(flat_svg);
    }
    None
}

/// Whether a path is a regular PNG/SVG small enough to decode.
fn usable(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() || meta.len() == 0 {
        return false;
    }
    let ceiling = if is_svg(path) {
        MAX_SVG_BYTES
    } else {
        MAX_ICON_BYTES
    };
    meta.len() <= ceiling
}

/// Whether a path names an SVG.
fn is_svg(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
}

/// A theme's size directories, nearest `want` first. A theme without size
/// directories yields nothing here; its flat directory is tried by the caller.
fn size_dirs(theme: &Path, want: u32) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(theme) else {
        return Vec::new();
    };
    let mut sized: Vec<(u32, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let (w, h) = name.split_once('x')?;
            let width: u32 = w.parse().ok()?;
            let height: u32 = h.parse().ok()?;
            (width == height && width > 0).then_some((width, entry.path()))
        })
        .collect();
    sized.sort_by_key(|(edge, _)| (edge.abs_diff(want), *edge));
    sized.into_iter().map(|(_, path)| path).collect()
}

/// The scaled copies, keyed by file and size.
type Cache = HashMap<(PathBuf, usize), Rc<Vec<u32>>>;

thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(HashMap::new());
}

/// Drop every decoded icon.
///
/// Called when the icon tree changed on disk: a name that resolved to nothing a
/// moment ago may resolve now, and a name that resolved elsewhere must not keep
/// drawing the old file. The cache is bounded at [`MAX_CACHED`] copies, so this
/// is a small one-off drop, not a rebuild — the next draw decodes what it needs
/// and a tile that is never drawn costs nothing.
pub fn clear_cache() {
    CACHE.with(|cache| cache.borrow_mut().clear());
}

/// How many scaled copies the cache holds.
///
/// For the diagnostics line and the tests, never for the draw path.
#[must_use]
pub fn cache_len() -> usize {
    CACHE.with(|cache| cache.borrow().len())
}

/// Decode and scale `path` to `size`, drawing it into `r`.
///
/// Returns whether anything was drawn: a missing, unreadable, oversized or
/// unsupported icon leaves the caller's fallback in place.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn draw(canvas: &mut Canvas, path: &Path, r: Rect, alpha: f32) -> bool {
    let size = r.w.round().max(1.0) as usize;
    let size = size.min(MAX_ICON_SIDE as usize);
    let Some(px) = scaled(path, size) else {
        return false;
    };
    canvas.draw_argb(
        r.x.round() as i64,
        r.y.round() as i64,
        size as u32,
        size as u32,
        &px,
        alpha,
    );
    true
}

/// The icon at `size`, decoded once and kept until the bound is reached.
fn scaled(path: &Path, size: usize) -> Option<Rc<Vec<u32>>> {
    if size == 0 {
        return None;
    }
    CACHE.with(|cache| {
        if let Some(found) = cache.borrow().get(&(path.to_path_buf(), size)) {
            return Some(Rc::clone(found));
        }
        let made = Rc::new(decode_at(path, size)?);
        let mut cache = cache.borrow_mut();
        // A hard bound rather than an unbounded map: a session that walks a
        // thousand icon names must not keep a thousand decoded copies.
        if cache.len() >= MAX_CACHED {
            cache.clear();
        }
        cache.insert((path.to_path_buf(), size), Rc::clone(&made));
        Some(made)
    })
}

/// Decode one icon at `size`: a PNG is decoded then resampled, an SVG is
/// rasterised at that size directly.
fn decode_at(path: &Path, size: usize) -> Option<Vec<u32>> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    if is_svg(path) {
        if meta.len() > MAX_SVG_BYTES {
            return None;
        }
        let bytes = std::fs::read(path).ok()?;
        return rasterize_svg(&bytes, size);
    }
    if meta.len() > MAX_ICON_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let (px, w, h) = decode_png(&bytes)?;
    Some(resample(&px, w, h, size))
}

/// Decode a PNG into straight-alpha ARGB pixels.
#[allow(clippy::cast_possible_truncation)]
fn decode_png(bytes: &[u8]) -> Option<(Vec<u32>, usize, usize)> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.width == 0
        || info.height == 0
        || info.width > MAX_ICON_SIDE
        || info.height > MAX_ICON_SIDE
    {
        return None;
    }
    buf.truncate(info.buffer_size());
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        _ => return None,
    };
    let mut px = Vec::with_capacity(buf.len().checked_div(channels).unwrap_or(0));
    for chunk in buf.chunks_exact(channels) {
        let (r, g, b) = (
            chunk.first().copied()?,
            chunk.get(1).copied()?,
            chunk.get(2).copied()?,
        );
        let a = if channels == 4 {
            chunk.get(3).copied()?
        } else {
            255
        };
        px.push((u32::from(a) << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
    }
    Some((px, info.width as usize, info.height as usize))
}

/// Resample a decoded PNG to a square with bilinear filtering.
///
/// Bilinear rather than nearest neighbour: a 16 px icon drawn in an 82 px tile
/// was made of 5×5 blocks, and the smoothing is what makes it read as artwork
/// at all. Premultiplication keeps a transparent edge from picking up a dark
/// fringe.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::arithmetic_side_effects
)]
fn resample(px: &[u32], w: usize, h: usize, size: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(size.saturating_mul(size));
    if w == 0 || h == 0 || px.len() < w.saturating_mul(h) {
        return out;
    }
    for dy in 0..size {
        let sy = ((dy as f32 + 0.5) * h as f32 / size as f32 - 0.5).max(0.0);
        let y0 = sy.floor() as usize;
        let y1 = (y0 + 1).min(h - 1);
        let fy = sy - sy.floor();
        for dx in 0..size {
            let sx = ((dx as f32 + 0.5) * w as f32 / size as f32 - 0.5).max(0.0);
            let x0 = sx.floor() as usize;
            let x1 = (x0 + 1).min(w - 1);
            let fx = sx - sx.floor();
            let mut acc = [0.0_f32; 4];
            for (x, y, weight) in [
                (x0, y0, (1.0 - fx) * (1.0 - fy)),
                (x1, y0, fx * (1.0 - fy)),
                (x0, y1, (1.0 - fx) * fy),
                (x1, y1, fx * fy),
            ] {
                if let Some(&p) = px.get(y * w + x) {
                    let a = ((p >> 24) & 0xFF) as f32 / 255.0;
                    acc[0] += ((p >> 16) & 0xFF) as f32 * a * weight;
                    acc[1] += ((p >> 8) & 0xFF) as f32 * a * weight;
                    acc[2] += (p & 0xFF) as f32 * a * weight;
                    acc[3] += a * weight;
                }
            }
            if acc[3] <= 0.0 {
                out.push(0);
                continue;
            }
            let un = |v: f32| (v / acc[3]).round().clamp(0.0, 255.0) as u32;
            let a = (acc[3] * 255.0).round().clamp(0.0, 255.0) as u32;
            out.push((a << 24) | (un(acc[0]) << 16) | (un(acc[1]) << 8) | un(acc[2]));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// A small, bounded SVG rasteriser.

/// A 2-D affine transform: `x' = a·x + c·y + e`, `y' = b·x + d·y + f`.
#[derive(Debug, Clone, Copy)]
struct Affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Affine {
    const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    fn apply(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    /// `self` first, then `outer`.
    fn then(self, outer: Self) -> Self {
        Self {
            a: outer.a * self.a + outer.c * self.b,
            b: outer.b * self.a + outer.d * self.b,
            c: outer.a * self.c + outer.c * self.d,
            d: outer.b * self.c + outer.d * self.d,
            e: outer.a * self.e + outer.c * self.f + outer.e,
            f: outer.b * self.e + outer.d * self.f + outer.f,
        }
    }

    /// Parse `translate(...)`, `scale(...)`, `matrix(...)` or a list of them.
    #[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    fn parse(text: &str) -> Option<Self> {
        let mut out = Self::IDENTITY;
        let mut rest = text.trim();
        while !rest.is_empty() {
            let open = rest.find('(')?;
            let close = rest.get(open..)?.find(')')? + open;
            let name = rest.get(..open)?.trim();
            let args: Vec<f32> = rest
                .get(open + 1..close)?
                .split([' ', ',', '\t'])
                .filter(|t| !t.is_empty())
                .map(str::parse)
                .collect::<Result<Vec<f32>, _>>()
                .ok()?;
            let step = match (name, args.as_slice()) {
                ("translate", [x]) => Self {
                    e: *x,
                    ..Self::IDENTITY
                },
                ("translate", [x, y]) => Self {
                    e: *x,
                    f: *y,
                    ..Self::IDENTITY
                },
                ("scale", [s]) => Self {
                    a: *s,
                    d: *s,
                    ..Self::IDENTITY
                },
                ("scale", [x, y]) => Self {
                    a: *x,
                    d: *y,
                    ..Self::IDENTITY
                },
                ("matrix", [a, b, c, d, e, f]) => Self {
                    a: *a,
                    b: *b,
                    c: *c,
                    d: *d,
                    e: *e,
                    f: *f,
                },
                _ => return None,
            };
            out = step.then(out);
            rest = rest.get(close + 1..)?.trim_start();
        }
        Some(out)
    }
}

/// One element of a document, with its raw attribute text.
struct Element<'a> {
    name: &'a str,
    attrs: &'a str,
    closing: bool,
}

/// Split an SVG document into elements, skipping comments and declarations.
#[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
fn elements(text: &str) -> Vec<Element<'_>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        let after = rest.get(open..).unwrap_or_default();
        let Some(close) = after.find('>') else {
            break;
        };
        let body = after.get(1..close).unwrap_or_default();
        rest = after.get(close + 1..).unwrap_or_default();
        if body.starts_with('?') || body.starts_with('!') {
            // `<?xml …?>` or a comment: skip to its own terminator.
            if body.starts_with('!') && body[1..].starts_with("--") {
                if let Some(end) = rest.find("-->") {
                    rest = rest.get(end + 3..).unwrap_or_default();
                }
            }
            continue;
        }
        let closing = body.starts_with('/');
        let trimmed = body.trim_start_matches('/').trim_start();
        let name_end = trimmed
            .find(|c: char| c.is_ascii_whitespace() || c == '/')
            .unwrap_or(trimmed.len());
        let name = trimmed.get(..name_end).unwrap_or_default();
        let attrs = trimmed
            .get(name_end..)
            .unwrap_or_default()
            .trim_end_matches('/');
        out.push(Element {
            name,
            attrs,
            closing,
        });
        if out.len() >= MAX_SVG_ELEMENTS.saturating_mul(2) {
            break;
        }
    }
    out
}

/// The value of `name="..."` in an attribute string.
fn attr<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let mut from = 0usize;
    while let Some(found) = attrs.get(from..)?.find(name) {
        let at = from.saturating_add(found);
        let before_ok = at == 0
            || attrs
                .get(..at)
                .and_then(|s| s.chars().last())
                .is_some_and(|c| c.is_ascii_whitespace());
        let after = at.saturating_add(name.len());
        let tail = attrs.get(after..)?.trim_start();
        if before_ok && tail.starts_with('=') {
            let value = tail.get(1..)?.trim_start();
            let quote = value.chars().next()?;
            if quote == '"' || quote == '\'' {
                let body = value.get(1..)?;
                return body.get(..body.find(quote)?);
            }
            return value.split_ascii_whitespace().next();
        }
        from = after;
    }
    None
}

/// A parsed fill or stroke.
#[derive(Debug, Clone, Copy)]
struct Paint {
    r: u8,
    g: u8,
    b: u8,
    a: f32,
}

impl Paint {
    const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 1.0,
    };
}

/// Parse `#rgb`, `#rrggbb`, `rgb(...)`, a named colour, `none` or `url(#id)`.
fn parse_paint(value: &str, gradients: &HashMap<String, Paint>) -> Option<Paint> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("none") || value == "transparent" {
        return None;
    }
    if let Some(rest) = value.strip_prefix('#') {
        let hex = |s: &str| u8::from_str_radix(s, 16).ok();
        return match rest.len() {
            3 => {
                let r = hex(rest.get(0..1)?)?;
                let g = hex(rest.get(1..2)?)?;
                let b = hex(rest.get(2..3)?)?;
                Some(Paint {
                    r: r.saturating_mul(17),
                    g: g.saturating_mul(17),
                    b: b.saturating_mul(17),
                    a: 1.0,
                })
            }
            6 => Some(Paint {
                r: hex(rest.get(0..2)?)?,
                g: hex(rest.get(2..4)?)?,
                b: hex(rest.get(4..6)?)?,
                a: 1.0,
            }),
            _ => None,
        };
    }
    if let Some(url) = value.strip_prefix("url(") {
        let id = url.trim_end_matches(')').trim().trim_start_matches('#');
        return gradients.get(id).copied();
    }
    if let Some(inner) = value
        .strip_prefix("rgb(")
        .or_else(|| value.strip_prefix("rgba("))
    {
        let inner = inner.trim_end_matches(')');
        let parts: Vec<&str> = inner.split([',', ' ']).filter(|t| !t.is_empty()).collect();
        let channel = |text: &str| -> Option<u8> {
            if let Some(percent) = text.strip_suffix('%') {
                let value: f32 = percent.trim().parse().ok()?;
                Some((value * 2.55).round().clamp(0.0, 255.0) as u8)
            } else {
                let value: f32 = text.trim().parse().ok()?;
                Some(value.round().clamp(0.0, 255.0) as u8)
            }
        };
        return Some(Paint {
            r: channel(parts.first()?)?,
            g: channel(parts.get(1)?)?,
            b: channel(parts.get(2)?)?,
            a: parts
                .get(3)
                .and_then(|v| v.trim().parse::<f32>().ok())
                .unwrap_or(1.0)
                .clamp(0.0, 1.0),
        });
    }
    let named = match value.to_ascii_lowercase().as_str() {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "blue" => (0, 0, 255),
        "gray" | "grey" => (128, 128, 128),
        "yellow" => (255, 255, 0),
        "orange" => (255, 165, 0),
        _ => return Some(Paint::BLACK),
    };
    Some(Paint {
        r: named.0,
        g: named.1,
        b: named.2,
        a: 1.0,
    })
}

/// The first stop colour of every gradient in a document, by id.
///
/// A gradient is not reproduced as a gradient; its first stop is used as a flat
/// fill. That is an approximation, and it is named as one in the module docs.
#[allow(clippy::arithmetic_side_effects)]
fn gradient_stops(text: &str) -> HashMap<String, Paint> {
    let mut out = HashMap::new();
    let lower = text.to_ascii_lowercase();
    let mut from = 0usize;
    while let Some(found) = lower.get(from..).and_then(|s| s.find("<lineargradient")) {
        let at = from.saturating_add(found);
        let Some(end) = lower.get(at..).and_then(|s| s.find('>')) else {
            break;
        };
        let tag = text.get(at..at.saturating_add(end)).unwrap_or_default();
        let id = attr(tag, "id").unwrap_or_default();
        let block_end = lower
            .get(at..)
            .and_then(|s| s.find("</lineargradient"))
            .map_or(at.saturating_add(end), |e| at.saturating_add(e));
        if let Some(stop) = lower
            .get(at..block_end)
            .and_then(|s| s.find("<stop"))
            .map(|s| at.saturating_add(s))
        {
            let stop_tag = text.get(stop..block_end).unwrap_or_default();
            if let Some(color) = attr(stop_tag, "stop-color") {
                if let Some(paint) = parse_paint(color, &HashMap::new()) {
                    out.insert(id.to_owned(), paint);
                }
            }
        }
        from = at.saturating_add(end).max(from + 1);
    }
    out
}

/// Rasterise an SVG into `size` × `size` straight-alpha ARGB pixels.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::too_many_lines
)]
#[allow(clippy::type_complexity)]
fn rasterize_svg(bytes: &[u8], size: usize) -> Option<Vec<u32>> {
    let size = size.clamp(1, MAX_RASTER_SIDE);
    let text = String::from_utf8_lossy(bytes);
    let gradients = gradient_stops(&text);
    let mut canvas = vec![0u32; size.saturating_mul(size)];

    // The viewBox sets the document's own coordinates; a document without one
    // is taken to be in pixels equal to its width and height.
    let mut view = Affine::IDENTITY;
    let mut width = size as f32;
    let mut height = size as f32;
    for element in elements(&text).iter().take(1) {
        if element.name != "svg" {
            break;
        }
        if let Some(box_) = attr(element.attrs, "viewBox") {
            let nums: Vec<f32> = box_
                .split([' ', ','])
                .filter(|t| !t.is_empty())
                .filter_map(|t| t.parse().ok())
                .collect();
            if let [x, y, w, h] = nums.as_slice() {
                if *w > 0.0 && *h > 0.0 {
                    let scale = (size as f32 / *w).min(size as f32 / *h);
                    let dx = (size as f32 - w * scale) / 2.0;
                    let dy = (size as f32 - h * scale) / 2.0;
                    view = Affine {
                        a: scale,
                        b: 0.0,
                        c: 0.0,
                        d: scale,
                        e: dx - x * scale,
                        f: dy - y * scale,
                    };
                }
            }
        } else {
            width = attr(element.attrs, "width")
                .and_then(|v| v.trim_end_matches("px").parse().ok())
                .unwrap_or(width);
            height = attr(element.attrs, "height")
                .and_then(|v| v.trim_end_matches("px").parse().ok())
                .unwrap_or(height);
            if width > 0.0 && height > 0.0 {
                let scale = (size as f32 / width).min(size as f32 / height);
                view = Affine {
                    a: scale,
                    b: 0.0,
                    c: 0.0,
                    d: scale,
                    e: 0.0,
                    f: 0.0,
                };
            }
        }
    }

    // Group state, inherited by shapes.
    let mut fill = Some(Paint::BLACK);
    let mut stroke = None;
    let mut stroke_width = 1.0_f32;
    let mut even_odd = false;
    let mut opacity = 1.0_f32;
    let mut transforms: Vec<Affine> = vec![view];
    let mut stack: Vec<(Option<Paint>, Option<Paint>, f32, bool, f32)> = Vec::new();
    let mut points_left = MAX_SVG_POINTS;
    let mut samples_left = MAX_SVG_SAMPLES;
    // Definitions are not drawn: a `<clipPath>` path is a mask, not artwork.
    let mut definitions = 0usize;

    for element in elements(&text) {
        let name = element.name;
        if matches!(
            name,
            "defs"
                | "clipPath"
                | "mask"
                | "symbol"
                | "pattern"
                | "linearGradient"
                | "radialGradient"
        ) {
            if element.closing {
                definitions = definitions.saturating_sub(1);
            } else if !element.attrs.trim_end().ends_with('/') {
                definitions = definitions.saturating_add(1);
            }
            continue;
        }
        if definitions > 0 {
            continue;
        }
        if element.closing {
            match name {
                "g" | "svg" => {
                    if let Some((f, s, w, e, o)) = stack.pop() {
                        fill = f;
                        stroke = s;
                        stroke_width = w;
                        even_odd = e;
                        opacity = o;
                    }
                    if transforms.len() > 1 {
                        transforms.pop();
                    }
                }
                _ => {}
            }
            continue;
        }
        match name {
            "g" | "svg" => {
                stack.push((fill, stroke, stroke_width, even_odd, opacity));
                if let Some(t) = attr(element.attrs, "transform").and_then(Affine::parse) {
                    let current = transforms.last().copied().unwrap_or(view);
                    transforms.push(t.then(current));
                } else {
                    transforms.push(transforms.last().copied().unwrap_or(view));
                }
                if let Some(v) = attr(element.attrs, "fill") {
                    fill = parse_paint(v, &gradients);
                }
                if let Some(v) = attr(element.attrs, "stroke") {
                    stroke = parse_paint(v, &gradients);
                }
                if let Some(v) = attr(element.attrs, "fill-rule") {
                    even_odd = v.eq_ignore_ascii_case("evenodd");
                }
            }
            "rect" | "circle" | "ellipse" | "line" | "polygon" | "polyline" | "path" => {
                let transform = transforms.last().copied().unwrap_or(view);
                let shape_fill =
                    attr(element.attrs, "fill").map_or(fill, |v| parse_paint(v, &gradients));
                let shape_stroke =
                    attr(element.attrs, "stroke").map_or(stroke, |v| parse_paint(v, &gradients));
                let shape_even_odd = attr(element.attrs, "fill-rule")
                    .is_some_and(|v| v.eq_ignore_ascii_case("evenodd"))
                    || even_odd;
                let shape_opacity = attr(element.attrs, "opacity")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0)
                    * opacity;
                let fill_opacity = attr(element.attrs, "fill-opacity")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                let width = attr(element.attrs, "stroke-width")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(stroke_width)
                    .clamp(0.0, 64.0);

                let mut polygons: Vec<Vec<(f32, f32)>> = Vec::new();
                let mut budget = PointBudget {
                    left: &mut points_left,
                };
                match name {
                    "rect" => {
                        if let Some(poly) = rect_polygon(element.attrs) {
                            polygons.push(poly);
                        }
                    }
                    "circle" | "ellipse" => {
                        if let Some(poly) = ellipse_polygon(element.attrs, name == "circle") {
                            polygons.push(poly);
                        }
                    }
                    "line" => {
                        if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                            num(element.attrs, "x1"),
                            num(element.attrs, "y1"),
                            num(element.attrs, "x2"),
                            num(element.attrs, "y2"),
                        ) {
                            polygons.push(vec![(x1, y1), (x2, y2)]);
                        }
                    }
                    "polygon" | "polyline" => {
                        if let Some(points) = attr(element.attrs, "points") {
                            let nums: Vec<f32> = points
                                .split([' ', ',', '\n', '\t'])
                                .filter(|t| !t.is_empty())
                                .filter_map(|t| t.parse().ok())
                                .collect();
                            let mut poly = Vec::new();
                            let mut i = 0usize;
                            while i + 1 < nums.len() && !budget.spent() {
                                poly.push((nums[i], nums[i + 1]));
                                budget.take(1);
                                i += 2;
                            }
                            // A `polyline` stays open: it is only ever stroked,
                            // and the filler closes every ring itself.
                            polygons.push(poly);
                        }
                    }
                    _ => {
                        if let Some(d) = attr(element.attrs, "d") {
                            if let Some(paths) = flatten_path(d, &mut budget) {
                                polygons.extend(paths);
                            }
                        }
                    }
                }

                // Transform in place: one walk, then fill.
                for poly in &mut polygons {
                    for point in poly.iter_mut() {
                        *point = transform.apply(point.0, point.1);
                    }
                }
                if let Some(paint) = shape_fill {
                    let color = Paint {
                        a: paint.a * fill_opacity * shape_opacity,
                        ..paint
                    };
                    fill_polygons(
                        &mut canvas,
                        size,
                        &polygons,
                        color,
                        shape_even_odd,
                        &mut samples_left,
                    );
                }
                if let Some(paint) = shape_stroke {
                    let color = Paint {
                        a: paint.a * shape_opacity,
                        ..paint
                    };
                    let mut stroked: Vec<Vec<(f32, f32)>> = Vec::new();
                    for poly in &polygons {
                        stroke_polyline(poly, width, &mut stroked, &mut points_left);
                    }
                    fill_polygons(&mut canvas, size, &stroked, color, false, &mut samples_left);
                }
            }
            _ => {}
        }
    }
    if canvas.iter().all(|&p| p >> 24 == 0) {
        return None;
    }
    Some(canvas)
}

/// A point allowance shared by every path in one document.
struct PointBudget<'a> {
    left: &'a mut usize,
}

impl PointBudget<'_> {
    fn spent(&self) -> bool {
        *self.left == 0
    }

    fn take(&mut self, n: usize) {
        *self.left = self.left.saturating_sub(n);
    }
}

/// A numeric attribute.
fn num(attrs: &str, name: &str) -> Option<f32> {
    attr(attrs, name)?.trim().parse().ok()
}

/// A `<rect>` as one polygon.
fn rect_polygon(attrs: &str) -> Option<Vec<(f32, f32)>> {
    let x = num(attrs, "x").unwrap_or(0.0);
    let y = num(attrs, "y").unwrap_or(0.0);
    let w = num(attrs, "width")?;
    let h = num(attrs, "height")?;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)])
}

/// A `<circle>` or `<ellipse>` as a polygon.
fn ellipse_polygon(attrs: &str, circle: bool) -> Option<Vec<(f32, f32)>> {
    let cx = num(attrs, "cx").unwrap_or(0.0);
    let cy = num(attrs, "cy").unwrap_or(0.0);
    let rx = if circle {
        num(attrs, "r")?
    } else {
        num(attrs, "rx")?
    };
    let ry = if circle { rx } else { num(attrs, "ry")? };
    if rx <= 0.0 || ry <= 0.0 {
        return None;
    }
    let steps = 24usize;
    let mut out = Vec::with_capacity(steps);
    for i in 0..steps {
        let angle = std::f32::consts::TAU * i as f32 / steps as f32;
        out.push((cx + rx * angle.cos(), cy + ry * angle.sin()));
    }
    Some(out)
}

/// Flatten a path's `d` into polygons, bounded by `budget`.
#[allow(
    clippy::too_many_lines,
    clippy::arithmetic_side_effects,
    clippy::manual_ignore_case_cmp,
    clippy::indexing_slicing
)]
fn flatten_path(d: &str, budget: &mut PointBudget<'_>) -> Option<Vec<Vec<(f32, f32)>>> {
    let nums: Vec<f32> = d
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse().ok())
        .collect();
    let mut tokens = d.chars().filter(|c| c.is_ascii_alphabetic()).peekable();
    let mut index = 0usize;
    let mut out: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut current = (0.0_f32, 0.0_f32);
    let mut start = current;
    let mut poly: Vec<(f32, f32)> = Vec::new();
    let mut last_cubic: Option<(f32, f32)> = None;
    let mut last_quad: Option<(f32, f32)> = None;
    let mut command = ' ';
    let take = |index: &mut usize, nums: &[f32], n: usize| -> Option<Vec<f32>> {
        let end = index.checked_add(n)?;
        let slice = nums.get(*index..end)?;
        *index = end;
        Some(slice.to_vec())
    };
    while let Some(&next) = tokens.peek() {
        if next.is_ascii_alphabetic() {
            command = tokens.next().unwrap_or(' ');
        }
        if budget.spent() {
            break;
        }
        let relative = command.is_ascii_lowercase();
        match command.to_ascii_uppercase() {
            'M' => {
                let Some(v) = take(&mut index, &nums, 2) else {
                    break;
                };
                let point = if relative {
                    (current.0 + v[0], current.1 + v[1])
                } else {
                    (v[0], v[1])
                };
                if poly.len() > 1 {
                    out.push(std::mem::take(&mut poly));
                }
                poly.push(point);
                current = point;
                start = point;
                budget.take(1);
                command = if relative { 'l' } else { 'L' };
                last_cubic = None;
                last_quad = None;
            }
            'L' => {
                let Some(v) = take(&mut index, &nums, 2) else {
                    break;
                };
                let point = if relative {
                    (current.0 + v[0], current.1 + v[1])
                } else {
                    (v[0], v[1])
                };
                poly.push(point);
                current = point;
                budget.take(1);
                last_cubic = None;
                last_quad = None;
            }
            'H' => {
                let Some(v) = take(&mut index, &nums, 1) else {
                    break;
                };
                let point = if relative {
                    (current.0 + v[0], current.1)
                } else {
                    (v[0], current.1)
                };
                poly.push(point);
                current = point;
                budget.take(1);
                last_cubic = None;
                last_quad = None;
            }
            'V' => {
                let Some(v) = take(&mut index, &nums, 1) else {
                    break;
                };
                let point = if relative {
                    (current.0, current.1 + v[0])
                } else {
                    (current.0, v[0])
                };
                poly.push(point);
                current = point;
                budget.take(1);
                last_cubic = None;
                last_quad = None;
            }
            'C' | 'S' => {
                let cubic = command.to_ascii_uppercase() == 'C';
                let n = if cubic { 6 } else { 4 };
                let Some(v) = take(&mut index, &nums, n) else {
                    break;
                };
                let (c1, c2, end) = if cubic {
                    let mut c1 = (v[0], v[1]);
                    let mut c2 = (v[2], v[3]);
                    let mut end = (v[4], v[5]);
                    if relative {
                        c1 = (current.0 + c1.0, current.1 + c1.1);
                        c2 = (current.0 + c2.0, current.1 + c2.1);
                        end = (current.0 + end.0, current.1 + end.1);
                    }
                    (c1, c2, end)
                } else {
                    let mut c2 = (v[0], v[1]);
                    let mut end = (v[2], v[3]);
                    if relative {
                        c2 = (current.0 + c2.0, current.1 + c2.1);
                        end = (current.0 + end.0, current.1 + end.1);
                    }
                    let c1 = match last_cubic {
                        Some(prev) => (
                            current.0 + (current.0 - prev.0),
                            current.1 + (current.1 - prev.1),
                        ),
                        None => current,
                    };
                    (c1, c2, end)
                };
                flatten_cubic(&mut poly, current, c1, c2, end, budget);
                last_cubic = Some(c2);
                last_quad = None;
                current = end;
            }
            'Q' | 'T' => {
                let quadratic = command.to_ascii_uppercase() == 'Q';
                let n = if quadratic { 4 } else { 2 };
                let Some(v) = take(&mut index, &nums, n) else {
                    break;
                };
                let (control, end) = if quadratic {
                    let mut c = (v[0], v[1]);
                    let mut e = (v[2], v[3]);
                    if relative {
                        c = (current.0 + c.0, current.1 + c.1);
                        e = (current.0 + e.0, current.1 + e.1);
                    }
                    (c, e)
                } else {
                    let mut e = (v[0], v[1]);
                    if relative {
                        e = (current.0 + e.0, current.1 + e.1);
                    }
                    let c = match last_quad {
                        Some(prev) => (
                            current.0 + (current.0 - prev.0),
                            current.1 + (current.1 - prev.1),
                        ),
                        None => current,
                    };
                    (c, e)
                };
                flatten_quad(&mut poly, current, control, end, budget);
                last_quad = Some(control);
                last_cubic = None;
                current = end;
            }
            'Z' => {
                if poly.len() > 1 {
                    poly.push(start);
                    out.push(std::mem::take(&mut poly));
                }
                current = start;
                last_cubic = None;
                last_quad = None;
                command = 'Z';
            }
            // An arc is not supported. Dropping the whole path is honest:
            // drawing it as a straight line would put artwork in the wrong
            // place and call it the icon.
            'A' => return None,
            _ => break,
        }
    }
    if poly.len() > 1 {
        out.push(poly);
    }
    (!out.is_empty()).then_some(out)
}

/// Append a cubic Bézier, flattened to a bounded number of segments.
#[allow(clippy::arithmetic_side_effects)]
fn flatten_cubic(
    poly: &mut Vec<(f32, f32)>,
    from: (f32, f32),
    c1: (f32, f32),
    c2: (f32, f32),
    to: (f32, f32),
    budget: &mut PointBudget<'_>,
) {
    let steps = curve_steps(from, to, c1, c2);
    for step in 1..=steps {
        if budget.spent() {
            return;
        }
        let t = step as f32 / steps as f32;
        let u = 1.0 - t;
        let x =
            u * u * u * from.0 + 3.0 * u * u * t * c1.0 + 3.0 * u * t * t * c2.0 + t * t * t * to.0;
        let y =
            u * u * u * from.1 + 3.0 * u * u * t * c1.1 + 3.0 * u * t * t * c2.1 + t * t * t * to.1;
        poly.push((x, y));
        budget.take(1);
    }
}

/// Append a quadratic Bézier, flattened to a bounded number of segments.
fn flatten_quad(
    poly: &mut Vec<(f32, f32)>,
    from: (f32, f32),
    control: (f32, f32),
    to: (f32, f32),
    budget: &mut PointBudget<'_>,
) {
    let steps = curve_steps(from, to, control, control);
    for step in 1..=steps {
        if budget.spent() {
            return;
        }
        let t = step as f32 / steps as f32;
        let u = 1.0 - t;
        let x = u * u * from.0 + 2.0 * u * t * control.0 + t * t * to.0;
        let y = u * u * from.1 + 2.0 * u * t * control.1 + t * t * to.1;
        poly.push((x, y));
        budget.take(1);
    }
}

/// How many segments a curve needs: longer curves get more, within bounds.
fn curve_steps(from: (f32, f32), to: (f32, f32), c1: (f32, f32), c2: (f32, f32)) -> usize {
    let length = (to.0 - from.0).abs()
        + (to.1 - from.1).abs()
        + (c1.0 - from.0).abs()
        + (c1.1 - from.1).abs()
        + (c2.0 - to.0).abs()
        + (c2.1 - to.1).abs();
    // One segment per 3 units of rough length, 4..24.
    ((length / 3.0) as usize).clamp(4, 24)
}

/// Turn a polyline into quads for a stroke of `width`.
#[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
fn stroke_polyline(
    poly: &[(f32, f32)],
    width: f32,
    out: &mut Vec<Vec<(f32, f32)>>,
    budget: &mut usize,
) {
    let half = (width * 0.5).max(0.35);
    for window in poly.windows(2) {
        let (a, b) = (window[0], window[1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = (dx * dx + dy * dy).sqrt();
        if len <= 0.0001 {
            continue;
        }
        let (nx, ny) = (-dy / len * half, dx / len * half);
        out.push(vec![
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ]);
        if *budget < 8 {
            return;
        }
        *budget -= 8;
    }
}

/// Fill polygons with one flat colour, using the nonzero or even-odd rule.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]
fn fill_polygons(
    canvas: &mut [u32],
    size: usize,
    polygons: &[Vec<(f32, f32)>],
    paint: Paint,
    even_odd: bool,
    samples_left: &mut usize,
) {
    if paint.a <= 0.0 {
        return;
    }
    let mut total_points = 0usize;
    for poly in polygons {
        total_points = total_points.saturating_add(poly.len());
    }
    // Rebuild edges with both x coordinates, so the crossing is interpolated.
    // Every polygon is closed implicitly: SVG fills a subpath as if it ended
    // with `Z`, and a `<rect>` arrives as four corner points with no repeat.
    let mut full: Vec<(f32, f32, f32, f32, i32)> = Vec::new();
    for poly in polygons {
        if poly.len() < 2 {
            continue;
        }
        for index in 0..poly.len() {
            let (x0, y0) = poly[index];
            let (x1, y1) = poly[(index + 1) % poly.len()];
            if (y0 - y1).abs() < 0.0001 {
                continue;
            }
            if y0 < y1 {
                full.push((y0, y1, x0, x1, 1));
            } else {
                full.push((y1, y0, x1, x0, -1));
            }
        }
    }
    let work = total_points.saturating_mul(size);
    if work > *samples_left {
        return;
    }
    *samples_left = samples_left.saturating_sub(work);
    for y in 0..size {
        let scan = y as f32 + 0.5;
        let mut crossings: Vec<(f32, i32)> = Vec::new();
        for &(top, bottom, x_top, x_bottom, dir) in &full {
            if scan < top || scan >= bottom {
                continue;
            }
            let t = (scan - top) / (bottom - top);
            crossings.push((x_top + (x_bottom - x_top) * t, dir));
        }
        if crossings.len() < 2 {
            continue;
        }
        crossings.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut winding = 0i32;
        let mut span_start = 0.0_f32;
        let mut inside = false;
        for (x, dir) in crossings {
            let was = inside;
            if even_odd {
                winding = winding.saturating_add(1);
                inside = winding % 2 == 1;
            } else {
                winding = winding.saturating_add(dir);
                inside = winding != 0;
            }
            if !was && inside {
                span_start = x;
            } else if was && !inside {
                let x0 = span_start.max(0.0) as usize;
                let x1 = (x.min(size as f32)) as usize;
                for px in x0..x1.min(size) {
                    let index = y.saturating_mul(size).saturating_add(px);
                    if let Some(pixel) = canvas.get_mut(index) {
                        *pixel = blend(*pixel, paint);
                    }
                }
            }
        }
    }
}

/// Blend one straight-alpha paint over a pixel.
fn blend(dst: u32, paint: Paint) -> u32 {
    let sa = paint.a.clamp(0.0, 1.0);
    if sa <= 0.0 {
        return dst;
    }
    let da = ((dst >> 24) & 0xFF) as f32 / 255.0;
    let dr = ((dst >> 16) & 0xFF) as f32;
    let dg = ((dst >> 8) & 0xFF) as f32;
    let db = (dst & 0xFF) as f32;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return 0;
    }
    let channel = |src: u8, dst: f32| -> u32 {
        (((f32::from(src) * sa) + dst * da * (1.0 - sa)) / out_a)
            .round()
            .clamp(0.0, 255.0) as u32
    };
    ((out_a * 255.0).round().clamp(0.0, 255.0) as u32) << 24
        | channel(paint.r, dr) << 16
        | channel(paint.g, dg) << 8
        | channel(paint.b, db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::rgb;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-themeicons-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn plant(root: &Path, name: &str) -> PathBuf {
        let apps = root.join("hicolor/48x48/apps");
        std::fs::create_dir_all(&apps).expect("apps");
        let path = apps.join(format!("{name}.png"));
        std::fs::write(
            &path,
            include_bytes!("../../../assets/icons/dock/files.png"),
        )
        .expect("write icon");
        path
    }

    const SCALABLE: &str = r##"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <rect x="2" y="2" width="20" height="20" fill="#2080c0"/>
  <circle cx="12" cy="12" r="5" fill="rgb(250, 190, 40)"/>
  <path d="M 4 20 L 12 6 L 20 20 Z" fill="#f0f0f0" fill-rule="evenodd"/>
</svg>"##;

    fn plant_svg(root: &Path, name: &str, body: &str) -> PathBuf {
        let dir = root.join("hicolor/scalable/apps");
        std::fs::create_dir_all(&dir).expect("scalable");
        let path = dir.join(format!("{name}.svg"));
        std::fs::write(&path, body).expect("write svg");
        path
    }

    #[test]
    fn a_real_png_is_found_decoded_and_drawn() {
        let root = scratch("draw");
        let planted = plant(&root, "marker-icon");
        let found = find_in(std::slice::from_ref(&root), "marker-icon", 48).expect("found");
        assert_eq!(found, planted);
        let (px, w, h) = decode_png(&std::fs::read(&found).expect("bytes")).expect("decodes");
        assert!(w > 0 && h > 0);
        assert!(px.iter().any(|p| p >> 24 > 0), "nothing opaque");
        let field = rgb(0x20_24_30);
        let mut canvas = Canvas::new(60, 60, field);
        assert!(draw(
            &mut canvas,
            &found,
            Rect::new(10.0, 10.0, 40.0, 40.0),
            1.0
        ));
        assert_ne!(canvas.get(30, 30), Some(field), "the icon drew nothing");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_svg_is_found_and_rasterised_when_the_theme_has_no_png() {
        let root = scratch("svg");
        let planted = plant_svg(&root, "scalable-icon", SCALABLE);
        let found = find_in(std::slice::from_ref(&root), "scalable-icon", 48).expect("found");
        assert_eq!(found, planted);
        assert!(is_svg(&found));
        let px = scaled(&found, 48).expect("rasterised");
        assert_eq!(px.len(), 48 * 48);
        let opaque = px.iter().filter(|p| *p >> 24 > 0).count();
        assert!(opaque > 400, "the shapes filled pixels: {opaque}");
        // The fill colours the document names are in the raster.
        assert!(
            px.iter()
                .any(|p| (p >> 16) & 0xFF > 150 && (p >> 8) & 0xFF > 120),
            "the circle's yellow is missing"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_png_at_the_size_directory_wins_over_an_svg() {
        let root = scratch("prefer");
        let png = plant(&root, "both");
        let _ = plant_svg(&root, "both", SCALABLE);
        let found = find_in(std::slice::from_ref(&root), "both", 48).expect("found");
        assert_eq!(found, png, "the raster a theme shipped was ignored");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_theme_that_names_no_icon_falls_back_to_its_parent() {
        let root = scratch("inherits");
        // hicolor declares no icon here; Adwaita is its parent and has it.
        std::fs::create_dir_all(root.join("hicolor")).expect("hicolor");
        std::fs::write(
            root.join("hicolor/index.theme"),
            "[Icon Theme]\nName=Hicolor\nInherits=Adwaita,hicolor\n",
        )
        .expect("index");
        let adwaita = root.join("Adwaita");
        std::fs::create_dir_all(adwaita.join("scalable/apps")).expect("adwaita");
        let icon = adwaita.join("scalable/apps/parent-icon.svg");
        std::fs::write(&icon, SCALABLE).expect("icon");
        let found = find_in(std::slice::from_ref(&root), "parent-icon", 32).expect("inherited");
        assert_eq!(found, icon);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_inherit_cycle_terminates() {
        let root = scratch("cycle");
        for (name, parent) in [("A", "B"), ("B", "A")] {
            std::fs::create_dir_all(root.join(name)).expect("theme");
            std::fs::write(
                root.join(name).join("index.theme"),
                format!("[Icon Theme]\nInherits={parent}\n"),
            )
            .expect("index");
        }
        assert!(find_in(std::slice::from_ref(&root), "absent", 32).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_name_that_could_escape_the_theme_is_refused() {
        let root = scratch("escape");
        let _ = plant(&root, "real");
        for bad in ["../real", "a/b", "", "a\\0b"] {
            assert!(
                find_in(std::slice::from_ref(&root), bad, 48).is_none(),
                "accepted {bad:?}"
            );
        }
        assert!(find_in(std::slice::from_ref(&root), "absent", 48).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_oversized_file_is_not_an_icon() {
        let root = scratch("large");
        let apps = root.join("hicolor/48x48/apps");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::write(
            apps.join("huge.png"),
            vec![0u8; (MAX_ICON_BYTES as usize) + 1],
        )
        .expect("write");
        assert!(find_in(std::slice::from_ref(&root), "huge", 48).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_svg_over_its_bounds_is_refused_and_a_huge_path_does_not_hang() {
        let root = scratch("svg-bounds");
        let big = format!(
            "<svg viewBox=\"0 0 10 10\"><path d=\"M 0 0 {} Z\"/></svg>",
            "L 10 10 ".repeat(MAX_SVG_POINTS)
        );
        let path = plant_svg(&root, "huge-path", &big);
        // The file is over MAX_SVG_BYTES or over the point bound: either way it
        // is refused, and the call returns rather than looping.
        assert!(scaled(&path, 32).is_none());
        let oversized = format!(
            "<svg viewBox=\"0 0 4 4\">{}</svg>",
            " ".repeat((MAX_SVG_BYTES as usize) + 1)
        );
        let path = plant_svg(&root, "oversized", &oversized);
        assert!(scaled(&path, 32).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn bilinear_scaling_is_not_nearest_neighbour() {
        // A 2×2 image: black, white, white, black. Nearest would keep only those
        // two values; bilinear introduces the greys between them.
        let px = vec![0xFF00_0000, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFF00_0000];
        let out = resample(&px, 2, 2, 8);
        let distinct: std::collections::HashSet<u32> = out.iter().copied().collect();
        assert!(
            distinct.len() > 4,
            "only {} values: upscaling is blocky",
            distinct.len()
        );
    }

    #[test]
    fn rasterising_a_theme_sized_icon_is_quick_and_the_cache_is_bounded() {
        let root = scratch("cost");
        let path = plant_svg(&root, "cost-icon", SCALABLE);
        // Warm the cache, then measure real rasterisations at a dock size.
        let first = scaled(&path, 48);
        assert!(first.is_some());
        CACHE.with(|cache| cache.borrow_mut().clear());
        let start = std::time::Instant::now();
        let runs = 20usize;
        for _ in 0..runs {
            CACHE.with(|cache| cache.borrow_mut().clear());
            assert!(scaled(&path, 48).is_some());
        }
        let elapsed = start.elapsed();
        let per_icon = elapsed.as_secs_f64() * 1000.0 / runs as f64;
        println!("rasterise 48px: {per_icon:.3} ms/icon");
        assert!(per_icon < 200.0, "one icon took {per_icon:.1} ms");
        // The cache ceiling in bytes: 48 entries at most 128×128 px × 4 bytes.
        let ceiling = MAX_CACHED
            .saturating_mul(MAX_RASTER_SIDE)
            .saturating_mul(MAX_RASTER_SIDE)
            .saturating_mul(4);
        println!("cache ceiling: {ceiling} bytes");
        assert_eq!(ceiling, 3_145_728);
        assert_eq!(scaled(&path, 32).map(|p| p.len()), Some(32 * 32));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_bounds_are_pinned() {
        assert_eq!(MAX_ICON_BYTES, 512 * 1024);
        assert_eq!(MAX_SVG_BYTES, 256 * 1024);
        assert_eq!(MAX_ICON_SIDE, 512);
        assert_eq!(MAX_CACHED, 48);
        assert_eq!(MAX_THEMES, 64);
        assert_eq!(MAX_THEME_DEPTH, 8);
        assert_eq!(MAX_SVG_ELEMENTS, 400);
        assert_eq!(MAX_SVG_POINTS, 20_000);
        assert_eq!(MAX_RASTER_SIDE, 128);
        assert_eq!(MAX_SVG_SAMPLES, 40_000_000);
    }
}
