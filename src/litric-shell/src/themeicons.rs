//! Application icons from the machine's icon theme.
//!
//! A desktop entry names its icon; the pixels live in the freedesktop icon
//! themes under `$XDG_DATA_HOME/icons`, `~/.icons`, each `$XDG_DATA_DIRS/icons`
//! and finally `/usr/share/pixmaps`. This module finds the PNG for a name,
//! decodes it with the same `png` crate the dock artwork uses, and keeps a
//! bounded number of scaled copies so the draw path is a blit.
//!
//! # What it refuses
//!
//! A name containing `/`, `..` or a NUL; anything that is not a regular file;
//! anything over [`MAX_ICON_BYTES`]; and a picture over [`MAX_ICON_SIDE`] on
//! either edge. SVG is not decoded: this image has no SVG renderer, so an
//! icon that exists only as `.svg` falls back to the generated glyph rather
//! than to a fabricated picture.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::canvas::{Canvas, Rect};

/// The largest icon file read.
pub const MAX_ICON_BYTES: u64 = 512 * 1024;

/// The largest icon edge decoded, in pixels.
pub const MAX_ICON_SIDE: u32 = 512;

/// The most scaled icons kept at once.
pub const MAX_CACHED: usize = 48;

/// The most theme directories inspected for one lookup.
pub const MAX_THEMES: usize = 64;

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

/// Find the PNG for `name`, preferring one closest to `size` pixels.
#[must_use]
pub fn find(name: &str, size: u32) -> Option<PathBuf> {
    find_in(&theme_dirs(), name, size)
}

/// Find the PNG for `name` in `dirs`, earlier directories winning.
///
/// A flat `pixmaps`-style directory is tried first, then each theme, `hicolor`
/// before the others, with the size directory nearest `size` first.
#[must_use]
pub fn find_in(dirs: &[PathBuf], name: &str, size: u32) -> Option<PathBuf> {
    let name = name.trim();
    if name.is_empty()
        || name.contains('/')
        || name.contains("..")
        || name.contains('\0')
        || name.len() > 255
    {
        return None;
    }
    let mut themes_seen = 0usize;
    for dir in dirs {
        let flat = dir.join(format!("{name}.png"));
        if usable(&flat) {
            return Some(flat);
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut themes: Vec<PathBuf> = entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.path())
            .collect();
        themes.sort();
        // `hicolor` is the theme every other theme names as its fallback.
        themes.sort_by_key(|path| {
            usize::from(path.file_name().and_then(|n| n.to_str()) != Some("hicolor"))
        });
        for theme in themes {
            themes_seen = themes_seen.saturating_add(1);
            if themes_seen > MAX_THEMES {
                return None;
            }
            for size_dir in size_dirs(&theme, size) {
                for category in ["apps", "applications", "categories", "devices", "places"] {
                    let candidate = size_dir.join(category).join(format!("{name}.png"));
                    if usable(&candidate) {
                        return Some(candidate);
                    }
                }
            }
            let candidate = theme.join(format!("{name}.png"));
            if usable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Whether a path is a regular PNG small enough to decode.
fn usable(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    meta.is_file() && meta.len() > 0 && meta.len() <= MAX_ICON_BYTES
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

/// Decode and scale `path` to `size`, drawing it into `r`.
///
/// Returns whether anything was drawn: a missing, unreadable or oversized icon
/// leaves the caller's fallback in place.
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
        let (px, w, h) = decode(path)?;
        let made = Rc::new(scale(&px, w, h, size));
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

/// Decode a PNG into straight-alpha ARGB pixels.
fn decode(path: &Path) -> Option<(Vec<u32>, usize, usize)> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_ICON_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
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

/// Scale to a square with nearest-neighbour sampling. Icon artwork is drawn at
/// 16–64 pixels, where the difference from a filter is not visible and the
/// cost is not worth a frame.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn scale(px: &[u32], w: usize, h: usize, size: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(size.saturating_mul(size));
    if w == 0 || h == 0 {
        return out;
    }
    for dy in 0..size {
        let sy = dy
            .saturating_mul(h)
            .checked_div(size)
            .unwrap_or(0)
            .min(h.saturating_sub(1));
        for dx in 0..size {
            let sx = dx
                .saturating_mul(w)
                .checked_div(size)
                .unwrap_or(0)
                .min(w.saturating_sub(1));
            out.push(
                px.get(sy.saturating_mul(w).saturating_add(sx))
                    .copied()
                    .unwrap_or(0),
            );
        }
    }
    out
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

    #[test]
    fn a_real_png_is_found_decoded_and_drawn() {
        let root = scratch("draw");
        let planted = plant(&root, "marker-icon");
        let found = find_in(std::slice::from_ref(&root), "marker-icon", 48).expect("found");
        assert_eq!(found, planted);
        let (px, w, h) = decode(&found).expect("decodes");
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
    fn a_name_that_could_escape_the_theme_is_refused() {
        let root = scratch("escape");
        let _ = plant(&root, "real");
        for bad in ["../real", "a/b", "", "a\0b"] {
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
    fn the_same_icon_is_decoded_once_and_the_cache_is_bounded() {
        let root = scratch("cache");
        let planted = plant(&root, "marker-icon");
        let a = scaled(&planted, 32).expect("scaled");
        let b = scaled(&planted, 32).expect("scaled");
        assert!(Rc::ptr_eq(&a, &b), "decoded twice");
        assert_eq!(a.len(), 32 * 32);
        assert_eq!(MAX_CACHED, 48);
        assert_eq!(MAX_ICON_BYTES, 512 * 1024);
        let _ = std::fs::remove_dir_all(&root);
    }
}
