//! The dock's application icons: original raster artwork, embedded.
//!
//! The artwork is painted by `tools/gen_dock_icons.py` and lives in
//! `assets/icons/dock/`. Each icon is a 256-pixel tile with its own shading,
//! rim light and shadow, so the depth is in the pixels and the shell draws no
//! gradient to get it.
//!
//! A tile is decoded once and scaled once per size it is drawn at: the dock
//! asks for two sizes (resting and hovered), so every frame after the first is a
//! blit of a ready bitmap.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::canvas::{Canvas, Rect};
use crate::shortcuts::App;

/// One icon in the set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    /// The launcher.
    Launcher,
    /// An application.
    App(App),
    /// The LitricOS mark: white on transparent, to be tinted.
    Mark,
}

const fn bytes(slot: Slot) -> &'static [u8] {
    match slot {
        Slot::Mark => include_bytes!("../../../assets/brand/litric-mark.png"),
        Slot::Launcher => include_bytes!("../../../assets/icons/dock/launcher.png"),
        Slot::App(App::Files) => include_bytes!("../../../assets/icons/dock/files.png"),
        Slot::App(App::Browser) => include_bytes!("../../../assets/icons/dock/browser.png"),
        Slot::App(App::Terminal) => include_bytes!("../../../assets/icons/dock/terminal.png"),
        Slot::App(App::Settings) => include_bytes!("../../../assets/icons/dock/settings.png"),
        Slot::App(App::Activity) => include_bytes!("../../../assets/icons/dock/activity.png"),
        Slot::App(App::Defender) => include_bytes!("../../../assets/icons/dock/defender.png"),
        Slot::App(App::Environments) => {
            include_bytes!("../../../assets/icons/dock/environments.png")
        }
        Slot::App(App::Trash) => include_bytes!("../../../assets/icons/dock/trash.png"),
    }
}

/// A decoded, straight-alpha RGBA bitmap.
struct Bitmap {
    w: usize,
    h: usize,
    /// `[r, g, b, a]` per pixel.
    rgba: Vec<u8>,
}

fn decode(png_bytes: &[u8]) -> Option<Bitmap> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(png_bytes))
        .read_info()
        .ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.bit_depth != png::BitDepth::Eight || info.color_type != png::ColorType::Rgba {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some(Bitmap {
        w: info.width as usize,
        h: info.height as usize,
        rgba: buf,
    })
}

/// Scale to `size` x `size` by area averaging in premultiplied alpha, so the
/// soft edge and shadow of a tile do not pick up a dark fringe.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn scale(src: &Bitmap, size: usize) -> Vec<u32> {
    let size = size.max(1);
    let ratio_x = src.w as f32 / size as f32;
    let ratio_y = src.h as f32 / size as f32;
    let mut out = Vec::with_capacity(size.saturating_mul(size));
    for dy in 0..size {
        let (y0, y1) = (dy as f32 * ratio_y, (dy + 1) as f32 * ratio_y);
        for dx in 0..size {
            let (x0, x1) = (dx as f32 * ratio_x, (dx + 1) as f32 * ratio_x);
            let (mut r, mut g, mut b, mut a, mut wsum) = (0.0_f32, 0.0, 0.0, 0.0, 0.0);
            let mut sy = y0 as usize;
            while (sy as f32) < y1 && sy < src.h {
                let wy = (y1.min((sy + 1) as f32) - y0.max(sy as f32)).max(0.0);
                let mut sx = x0 as usize;
                while (sx as f32) < x1 && sx < src.w {
                    let wx = (x1.min((sx + 1) as f32) - x0.max(sx as f32)).max(0.0);
                    let weight = wx * wy;
                    let i = sy.saturating_mul(src.w).saturating_add(sx).saturating_mul(4);
                    if let Some(p) = src.rgba.get(i..i.saturating_add(4)) {
                        let pa = f32::from(p[3]) / 255.0;
                        r += f32::from(p[0]) * pa * weight;
                        g += f32::from(p[1]) * pa * weight;
                        b += f32::from(p[2]) * pa * weight;
                        a += pa * weight;
                        wsum += weight;
                    }
                    sx += 1;
                }
                sy += 1;
            }
            if wsum <= 0.0 || a <= 0.0 {
                out.push(0);
                continue;
            }
            let alpha = (a / wsum).clamp(0.0, 1.0);
            let un = |v: f32| ((v / a).round().clamp(0.0, 255.0)) as u32;
            let al = (alpha * 255.0).round() as u32;
            out.push((al << 24) | (un(r) << 16) | (un(g) << 8) | un(b));
        }
    }
    out
}

thread_local! {
    static SCALED: RefCell<HashMap<(Slot, usize), Rc<Vec<u32>>>> = RefCell::new(HashMap::new());
}

/// The icon scaled to `size`, made on the first request and kept.
fn scaled(slot: Slot, size: usize) -> Option<Rc<Vec<u32>>> {
    SCALED.with(|cache| {
        if let Some(found) = cache.borrow().get(&(slot, size)) {
            return Some(Rc::clone(found));
        }
        let bitmap = decode(bytes(slot))?;
        let made = Rc::new(scale(&bitmap, size));
        cache.borrow_mut().insert((slot, size), Rc::clone(&made));
        Some(made)
    })
}

/// Draw `slot` into `r`, which should be square. The tile's own margin and
/// shadow are part of the artwork, so `r` is the artwork's box, not the tile's.
/// `alpha` fades the whole icon.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn draw(canvas: &mut Canvas, slot: Slot, r: Rect, alpha: f32) {
    let size = r.w.round().max(1.0) as usize;
    if let Some(px) = scaled(slot, size) {
        canvas.draw_argb(
            r.x.round() as i64,
            r.y.round() as i64,
            size as u32,
            size as u32,
            &px,
            alpha,
        );
    }
}

/// Draw `slot` in one colour: its shape is its alpha, `color` is what it is
/// painted with. This is how the mark takes the theme's text colour.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn draw_tinted(canvas: &mut Canvas, slot: Slot, r: Rect, color: u32) {
    let size = r.w.round().max(1.0) as usize;
    if let Some(px) = scaled(slot, size) {
        let tinted: Vec<u32> = px
            .iter()
            .map(|p| (p & 0xFF00_0000) | (color & 0x00FF_FFFF))
            .collect();
        canvas.draw_argb(
            r.x.round() as i64,
            r.y.round() as i64,
            size as u32,
            size as u32,
            &tinted,
            1.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::rgb;

    #[test]
    fn every_icon_decodes_as_256_square_rgba() {
        let all = [Slot::Launcher, Slot::Mark].into_iter().chain(App::ALL.iter().map(|a| Slot::App(*a)));
        for slot in all {
            let b = decode(bytes(slot)).unwrap_or_else(|| panic!("{slot:?} did not decode"));
            assert_eq!((b.w, b.h), (256, 256), "{slot:?}");
        }
    }

    #[test]
    fn an_icon_is_drawn_inside_its_box_and_nowhere_else() {
        let field = rgb(0x20_24_30);
        let mut c = Canvas::new(120, 120, field);
        draw(&mut c, Slot::Launcher, Rect::new(30.0, 30.0, 60.0, 60.0), 1.0);
        let centre = c.get(60, 60).unwrap_or(0);
        assert_ne!(centre, field, "the icon left its centre untouched");
        assert_eq!(c.get(10, 10), Some(field));
        assert_eq!(c.get(110, 110), Some(field));
    }

    #[test]
    fn scaling_keeps_a_transparent_corner_transparent() {
        let b = decode(bytes(Slot::Launcher)).expect("decodes");
        let px = scale(&b, 48);
        assert_eq!(px.first().map(|p| p >> 24), Some(0), "the corner is not empty");
        assert!(px.iter().any(|p| p >> 24 == 255), "nothing is opaque");
    }

    #[test]
    fn a_tinted_mark_takes_the_colour_and_keeps_its_shape() {
        let field = rgb(0x00_00_00);
        let ink = rgb(0xF4_F2_ED);
        let mut c = Canvas::new(40, 40, field);
        draw_tinted(&mut c, Slot::Mark, Rect::new(4.0, 4.0, 32.0, 32.0), ink);
        assert_eq!(c.get(0, 0), Some(field), "ink outside the mark's box");
        let inked = c.pixels().iter().filter(|&&p| p == ink).count();
        assert!(inked > 20, "the mark left no solid ink ({inked})");
    }

    #[test]
    fn the_same_size_is_scaled_once() {
        let a = scaled(Slot::App(App::Files), 40).expect("scaled");
        let b = scaled(Slot::App(App::Files), 40).expect("scaled");
        assert!(Rc::ptr_eq(&a, &b));
    }
}
