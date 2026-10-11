//! The Linux framebuffer, reached without a single `ioctl`.
//!
//! # Why this way
//!
//! The usual way to drive `/dev/fb0` is `FBIOGET_VSCREENINFO` for the geometry
//! and `mmap` for the pixels. Both are `ioctl`/`mmap`, both need `unsafe`, and
//! the workspace forbids `unsafe` outright.
//!
//! It turns out neither is necessary. The kernel already publishes the geometry
//! as text under `/sys/class/graphics/fb0/`, and the fbdev character device
//! implements `write()`, so a frame can be delivered with `seek` and
//! `write_all`. The whole display path is therefore ordinary safe Rust over
//! ordinary files:
//!
//! ```text
//!   /sys/class/graphics/fb0/virtual_size     "1280,800"
//!   /sys/class/graphics/fb0/bits_per_pixel   "32"
//!   /sys/class/graphics/fb0/stride           "5120"
//!   /dev/fb0                                 the pixels
//! ```
//!
//! `write()` costs one copy compared with `mmap`. At 1280x800 that is 4 MB per
//! frame, which is nothing for a desktop that repaints when something changes
//! rather than sixty times a second, and it buys a display driver with no
//! `unsafe` in it at all. If a compositing path ever needs the frame rate,
//! `mmap` can be added then, measured, and confined to one module.
//!
//! # What is NOT known here
//!
//! sysfs does not publish the channel offsets, so the byte order has to be
//! assumed. Every 32-bit fbdev on x86 in practice is `XRGB8888` little-endian
//! (byte 0 blue, byte 1 green, byte 2 red). That assumption is written down in
//! [`PixelFormat`] and is overridable, because an assumption that cannot be
//! corrected from the command line is one that has to be right the first time.

use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::canvas::{Canvas, Rect};

/// How the framebuffer expects a pixel to be laid out in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PixelFormat {
    /// 32 bits: blue, green, red, unused. The near-universal x86 fbdev layout.
    Xrgb8888,
    /// 32 bits: red, green, blue, unused. Some big-endian and some ARM setups.
    Xbgr8888,
    /// 16 bits: 5 red, 6 green, 5 blue, little-endian.
    Rgb565,
}

impl PixelFormat {
    /// Bytes each pixel occupies.
    #[must_use]
    pub const fn bytes(self) -> usize {
        match self {
            Self::Xrgb8888 | Self::Xbgr8888 => 4,
            Self::Rgb565 => 2,
        }
    }

    /// The default for a given bit depth.
    #[must_use]
    pub const fn for_depth(bits: u32) -> Option<Self> {
        match bits {
            32 | 24 => Some(Self::Xrgb8888),
            16 => Some(Self::Rgb565),
            _ => None,
        }
    }

    /// Parse a name, for the `--format` option.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "xrgb8888" => Some(Self::Xrgb8888),
            "xbgr8888" => Some(Self::Xbgr8888),
            "rgb565" => Some(Self::Rgb565),
            _ => None,
        }
    }

    /// Write one `0xAARRGGBB` pixel into `out`, which must be [`Self::bytes`]
    /// long. A short slice writes nothing rather than panicking: a display
    /// driver that aborts on a truncated row takes the desktop with it.
    pub fn encode(self, px: u32, out: &mut [u8]) {
        let r = ((px >> 16) & 0xFF) as u8;
        let g = ((px >> 8) & 0xFF) as u8;
        let b = (px & 0xFF) as u8;
        match self {
            Self::Xrgb8888 => {
                if let Some(s) = out.get_mut(0..4) {
                    s.copy_from_slice(&[b, g, r, 0xFF]);
                }
            }
            Self::Xbgr8888 => {
                if let Some(s) = out.get_mut(0..4) {
                    s.copy_from_slice(&[r, g, b, 0xFF]);
                }
            }
            Self::Rgb565 => {
                let v: u16 = ((u16::from(r) & 0xF8) << 8)
                    | ((u16::from(g) & 0xFC) << 3)
                    | (u16::from(b) >> 3);
                if let Some(s) = out.get_mut(0..2) {
                    s.copy_from_slice(&v.to_le_bytes());
                }
            }
        }
    }
}

/// What the kernel says about a framebuffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FbInfo {
    /// Visible width in pixels.
    pub width: u32,
    /// Visible height in pixels.
    pub height: u32,
    /// Bits per pixel.
    pub bits_per_pixel: u32,
    /// Bytes per row. Not always `width * bytes_per_pixel`: the driver may pad.
    pub stride: u32,
    /// The driver's name for it, e.g. `bochs-drmdrmfb`.
    pub name: String,
    /// Assumed byte layout.
    pub format: PixelFormat,
}

impl FbInfo {
    /// Build from the raw contents of the sysfs attributes.
    ///
    /// Returns `None` when the geometry is missing or nonsensical. A zero-sized
    /// or unsupported-depth framebuffer must be refused here rather than
    /// producing a frame that gets written to the device and does nothing.
    #[must_use]
    pub fn parse(virtual_size: &str, bpp: &str, stride: &str, name: &str) -> Option<Self> {
        let (w, h) = virtual_size.trim().split_once(',')?;
        let width: u32 = w.trim().parse().ok()?;
        let height: u32 = h.trim().parse().ok()?;
        if width == 0 || height == 0 {
            return None;
        }
        let bits: u32 = bpp.trim().parse().ok()?;
        let format = PixelFormat::for_depth(bits)?;

        // A driver that does not publish a stride is assumed unpadded, which is
        // the common case; a wrong stride shears the picture diagonally, which
        // is at least unmistakable rather than subtle.
        let stride = stride
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|s| *s > 0)
            .unwrap_or_else(|| width.saturating_mul(u32::try_from(format.bytes()).unwrap_or(4)));

        Some(Self {
            width,
            height,
            bits_per_pixel: bits,
            stride,
            name: name.trim().to_owned(),
            format,
        })
    }

    /// Total bytes of one full frame.
    #[must_use]
    pub const fn frame_bytes(&self) -> u64 {
        (self.stride as u64).saturating_mul(self.height as u64)
    }

    /// A one-line summary for the console and the status file.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{}x{} {}bpp stride={} format={:?} driver={}",
            self.width, self.height, self.bits_per_pixel, self.stride, self.format, self.name
        )
    }
}

fn read_attr(dir: &Path, name: &str) -> Option<String> {
    fs::read_to_string(dir.join(name)).ok()
}

/// Read a framebuffer's geometry from a sysfs directory.
#[must_use]
pub fn info_from_sysfs(dir: &Path) -> Option<FbInfo> {
    FbInfo::parse(
        &read_attr(dir, "virtual_size")?,
        &read_attr(dir, "bits_per_pixel")?,
        &read_attr(dir, "stride").unwrap_or_default(),
        &read_attr(dir, "name").unwrap_or_default(),
    )
}

/// Find the first usable framebuffer under `root` (normally `/`).
///
/// Returns the sysfs directory, the device path, and the geometry. `fb0` is
/// tried first and then a small range, because a machine with a second adapter
/// can present the usable one at `fb1`.
#[must_use]
pub fn discover(root: &Path) -> Option<(PathBuf, PathBuf, FbInfo)> {
    for n in 0..4u32 {
        let sys = root.join(format!("sys/class/graphics/fb{n}"));
        let dev = root.join(format!("dev/fb{n}"));
        if let Some(info) = info_from_sysfs(&sys) {
            if dev.exists() {
                return Some((sys, dev, info));
            }
        }
    }
    None
}

/// An open framebuffer, ready to be written to.
#[derive(Debug)]
pub struct Framebuffer {
    info: FbInfo,
    file: fs::File,
    row: Vec<u8>,
}

impl Framebuffer {
    /// Open a framebuffer device for writing.
    ///
    /// # Errors
    /// Any I/O error from opening the device.
    pub fn open(dev: &Path, info: FbInfo) -> std::io::Result<Self> {
        let file = fs::OpenOptions::new().write(true).open(dev)?;
        let row = vec![0u8; info.stride as usize];
        Ok(Self { info, file, row })
    }

    /// The geometry in use.
    #[must_use]
    pub const fn info(&self) -> &FbInfo {
        &self.info
    }

    /// Send a canvas to the screen.
    ///
    /// The canvas may be a different size from the display; the overlap is
    /// drawn and the rest of each row is left as it was. Scaling silently would
    /// hide a geometry bug behind a slightly soft picture.
    ///
    /// # Errors
    /// Any I/O error from seeking or writing.
    pub fn present(&mut self, canvas: &Canvas) -> std::io::Result<()> {
        let bytes = self.info.format.bytes();
        let rows = self.info.height.min(canvas.height());
        let cols = self.info.width.min(canvas.width());

        self.file.seek(SeekFrom::Start(0))?;
        for y in 0..rows {
            for b in self.row.iter_mut() {
                *b = 0;
            }
            for x in 0..cols {
                let px = canvas
                    .get(i64::from(x), i64::from(y))
                    .unwrap_or(0xFF00_0000);
                let off = (x as usize).saturating_mul(bytes);
                if let Some(slot) = self.row.get_mut(off..off.saturating_add(bytes)) {
                    self.info.format.encode(px, slot);
                }
            }
            let offset = u64::from(self.info.stride).saturating_mul(u64::from(y));
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(&self.row)?;
        }
        self.file.flush()?;
        Ok(())
    }
}

impl Framebuffer {
    /// Send only the rows covered by `damage`.
    ///
    /// A full frame at 1280x800 is 4 MB through `write()`. Most changes touch a
    /// fraction of that: a clock that ticks needs about sixty rows, a hovered
    /// menu item about thirty. Presenting only what changed is where the
    /// difference between an idle desktop that costs nothing and one that costs
    /// a core actually comes from.
    ///
    /// Returns how many rows were written, so the diagnostics can show the
    /// saving rather than assert it.
    ///
    /// # Errors
    /// Any I/O error from seeking or writing.
    pub fn present_damage(&mut self, canvas: &Canvas, damage: &[Rect]) -> std::io::Result<u32> {
        if damage.is_empty() {
            return Ok(0);
        }
        // Rows rather than rectangles: the device is written a row at a time
        // anyway, so merging to a row set is both simpler and no slower than
        // tracking overlapping rectangles.
        let height = self.info.height.min(canvas.height());
        let mut rows = vec![false; height as usize];
        for r in damage {
            let y0 = (r.y.floor().max(0.0) as u32).min(height);
            let y1 = (r.bottom().ceil().max(0.0) as u32).min(height);
            for y in y0..y1 {
                if let Some(slot) = rows.get_mut(y as usize) {
                    *slot = true;
                }
            }
        }

        let bytes = self.info.format.bytes();
        let cols = self.info.width.min(canvas.width());
        let mut written = 0u32;
        for y in 0..height {
            if rows.get(y as usize) != Some(&true) {
                continue;
            }
            for b in self.row.iter_mut() {
                *b = 0;
            }
            for x in 0..cols {
                let px = canvas
                    .get(i64::from(x), i64::from(y))
                    .unwrap_or(0xFF00_0000);
                let off = (x as usize).saturating_mul(bytes);
                if let Some(slot) = self.row.get_mut(off..off.saturating_add(bytes)) {
                    self.info.format.encode(px, slot);
                }
            }
            let offset = u64::from(self.info.stride).saturating_mul(u64::from(y));
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(&self.row)?;
            written = written.saturating_add(1);
        }
        self.file.flush()?;
        Ok(written)
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::canvas::rgb;

    #[test]
    fn sysfs_geometry_parses() {
        let i = FbInfo::parse("1280,800\n", "32\n", "5120\n", "bochs-drmdrmfb\n").expect("parses");
        assert_eq!((i.width, i.height), (1280, 800));
        assert_eq!(i.bits_per_pixel, 32);
        assert_eq!(i.stride, 5120);
        assert_eq!(i.format, PixelFormat::Xrgb8888);
        assert_eq!(i.name, "bochs-drmdrmfb");
        assert_eq!(i.frame_bytes(), 4_096_000);
    }

    #[test]
    fn a_padded_stride_is_believed_rather_than_recomputed() {
        // The driver knows about its own padding. Recomputing width*4 over a
        // padded framebuffer shears the picture one pixel further right on
        // every row, which looks like a broken cable.
        let i = FbInfo::parse("1024,768", "32", "4224", "x").expect("parses");
        assert_eq!(i.stride, 4224);
        assert_ne!(i.stride, i.width * 4);
    }

    #[test]
    fn a_missing_stride_falls_back_to_an_unpadded_row() {
        let i = FbInfo::parse("800,600", "32", "", "x").expect("parses");
        assert_eq!(i.stride, 3200);
        let i = FbInfo::parse("800,600", "16", "0", "x").expect("parses");
        assert_eq!(i.stride, 1600);
    }

    #[test]
    fn nonsense_geometry_is_refused_instead_of_defaulted() {
        assert!(FbInfo::parse("0,800", "32", "", "").is_none(), "zero width");
        assert!(
            FbInfo::parse("1280,0", "32", "", "").is_none(),
            "zero height"
        );
        assert!(FbInfo::parse("1280", "32", "", "").is_none(), "no comma");
        assert!(FbInfo::parse("a,b", "32", "", "").is_none(), "not numbers");
        assert!(
            FbInfo::parse("1280,800", "8", "", "").is_none(),
            "8bpp unsupported"
        );
        assert!(FbInfo::parse("1280,800", "", "", "").is_none(), "no depth");
    }

    #[test]
    fn xrgb_puts_blue_in_the_first_byte() {
        // The assumption this whole module rests on. If it is ever wrong the
        // desktop comes out with red and blue swapped, which is why the first
        // screenshot of a new machine is worth looking at.
        let mut out = [0u8; 4];
        PixelFormat::Xrgb8888.encode(rgb(0x102030), &mut out);
        assert_eq!(out, [0x30, 0x20, 0x10, 0xFF]);
    }

    #[test]
    fn xbgr_is_the_other_order() {
        let mut out = [0u8; 4];
        PixelFormat::Xbgr8888.encode(rgb(0x102030), &mut out);
        assert_eq!(out, [0x10, 0x20, 0x30, 0xFF]);
    }

    #[test]
    fn rgb565_packs_into_two_bytes() {
        let mut out = [0u8; 2];
        PixelFormat::Rgb565.encode(rgb(0xFF0000), &mut out);
        assert_eq!(u16::from_le_bytes(out), 0xF800, "pure red");
        PixelFormat::Rgb565.encode(rgb(0x00FF00), &mut out);
        assert_eq!(u16::from_le_bytes(out), 0x07E0, "pure green");
        PixelFormat::Rgb565.encode(rgb(0x0000FF), &mut out);
        assert_eq!(u16::from_le_bytes(out), 0x001F, "pure blue");
    }

    #[test]
    fn encoding_into_a_short_slice_writes_nothing_rather_than_panicking() {
        let mut out = [0u8; 2];
        PixelFormat::Xrgb8888.encode(rgb(0xFFFFFF), &mut out);
        assert_eq!(out, [0, 0], "a truncated row must not take the shell down");
    }

    #[test]
    fn format_names_round_trip() {
        for (name, want) in [
            ("xrgb8888", PixelFormat::Xrgb8888),
            ("xbgr8888", PixelFormat::Xbgr8888),
            ("rgb565", PixelFormat::Rgb565),
        ] {
            assert_eq!(PixelFormat::parse(name), Some(want));
        }
        assert_eq!(PixelFormat::parse("bgra"), None);
    }

    #[test]
    fn discovery_of_a_fixture_root_finds_the_framebuffer() {
        // Proves the sysfs walk on a host that has no framebuffer at all.
        let root = std::env::temp_dir().join("litric-shell-fb-fixture");
        let sys = root.join("sys/class/graphics/fb0");
        let dev = root.join("dev");
        fs::create_dir_all(&sys).expect("sysfs fixture");
        fs::create_dir_all(&dev).expect("dev fixture");
        fs::write(sys.join("virtual_size"), "640,480\n").expect("size");
        fs::write(sys.join("bits_per_pixel"), "32\n").expect("bpp");
        fs::write(sys.join("stride"), "2560\n").expect("stride");
        fs::write(sys.join("name"), "fixturefb\n").expect("name");
        fs::write(dev.join("fb0"), b"").expect("device");

        let found = discover(&root).expect("discovers fb0");
        assert_eq!(found.2.width, 640);
        assert_eq!(found.2.name, "fixturefb");

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn discovery_of_a_root_without_a_framebuffer_is_none_not_a_guess() {
        assert!(discover(Path::new("definitely-not-a-root-8a1c")).is_none());
    }

    #[test]
    fn presenting_writes_exactly_one_frame_of_the_declared_size() {
        // Writes to a regular file standing in for /dev/fb0, then checks the
        // bytes. This is the only test that proves the pixels leaving the
        // canvas are the pixels the device would receive.
        let dir = std::env::temp_dir().join("litric-shell-fb-present");
        fs::create_dir_all(&dir).expect("dir");
        let dev = dir.join("fb");
        fs::write(&dev, b"").expect("create");

        let info = FbInfo::parse("4,2", "32", "16", "test").expect("info");
        let mut fb = Framebuffer::open(&dev, info).expect("open");
        let mut c = Canvas::new(4, 2, rgb(0x102030));
        c.put(0, 0, rgb(0xFF0000));
        fb.present(&c).expect("present");
        drop(fb);

        let bytes = fs::read(&dev).expect("read back");
        assert_eq!(bytes.len(), 16 * 2, "wrote {} bytes", bytes.len());
        assert_eq!(
            &bytes[0..4],
            &[0x00, 0x00, 0xFF, 0xFF],
            "first pixel is red"
        );
        assert_eq!(
            &bytes[4..8],
            &[0x30, 0x20, 0x10, 0xFF],
            "second is the fill"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn presenting_damage_writes_only_the_rows_that_changed() {
        let dir = std::env::temp_dir().join("litric-shell-fb-damage");
        fs::create_dir_all(&dir).expect("dir");
        let dev = dir.join("fb");
        fs::write(&dev, vec![0u8; 16 * 4]).expect("create");

        let info = FbInfo::parse("4,4", "32", "16", "test").expect("info");
        let mut fb = Framebuffer::open(&dev, info).expect("open");
        let mut c = Canvas::new(4, 4, rgb(0x000000));
        c.put(0, 2, rgb(0xFF0000));

        // Only row 2 is damaged.
        let n = fb
            .present_damage(&c, &[Rect::new(0.0, 2.0, 4.0, 1.0)])
            .expect("present");
        assert_eq!(n, 1, "wrote {n} rows for a one-row change");
        drop(fb);

        let bytes = fs::read(&dev).expect("read back");
        assert_eq!(
            &bytes[32..36],
            &[0x00, 0x00, 0xFF, 0xFF],
            "row 2 was written"
        );
        assert_eq!(&bytes[0..4], &[0, 0, 0, 0], "row 0 was left alone");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn presenting_no_damage_writes_nothing() {
        let dir = std::env::temp_dir().join("litric-shell-fb-nodamage");
        fs::create_dir_all(&dir).expect("dir");
        let dev = dir.join("fb");
        fs::write(&dev, vec![0u8; 16 * 4]).expect("create");
        let info = FbInfo::parse("4,4", "32", "16", "test").expect("info");
        let mut fb = Framebuffer::open(&dev, info).expect("open");
        let c = Canvas::new(4, 4, rgb(0xFFFFFF));
        assert_eq!(fb.present_damage(&c, &[]).expect("present"), 0);
        drop(fb);
        assert!(
            fs::read(&dev).expect("read").iter().all(|b| *b == 0),
            "an undamaged frame still wrote to the device"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn damage_outside_the_screen_is_clipped_rather_than_refused() {
        let dir = std::env::temp_dir().join("litric-shell-fb-clip");
        fs::create_dir_all(&dir).expect("dir");
        let dev = dir.join("fb");
        fs::write(&dev, vec![0u8; 16 * 4]).expect("create");
        let info = FbInfo::parse("4,4", "32", "16", "test").expect("info");
        let mut fb = Framebuffer::open(&dev, info).expect("open");
        let c = Canvas::new(4, 4, rgb(0xFFFFFF));
        let n = fb
            .present_damage(&c, &[Rect::new(-100.0, -100.0, 1000.0, 1000.0)])
            .expect("present");
        assert_eq!(n, 4, "clipped damage should still cover the screen");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_canvas_smaller_than_the_screen_is_placed_not_stretched() {
        let dir = std::env::temp_dir().join("litric-shell-fb-small");
        fs::create_dir_all(&dir).expect("dir");
        let dev = dir.join("fb");
        fs::write(&dev, b"").expect("create");

        let info = FbInfo::parse("4,2", "32", "16", "test").expect("info");
        let mut fb = Framebuffer::open(&dev, info).expect("open");
        let c = Canvas::new(2, 1, rgb(0xFFFFFF));
        fb.present(&c).expect("present");
        drop(fb);

        let bytes = fs::read(&dev).expect("read back");
        assert_eq!(&bytes[0..4], &[0xFF, 0xFF, 0xFF, 0xFF], "covered pixel");
        assert_eq!(&bytes[8..12], &[0, 0, 0, 0], "uncovered pixel left alone");

        fs::remove_dir_all(&dir).ok();
    }
}
