//! Sequential JPEG decoding, written from ITU-T T.81 (09/92).
//!
//! Chromium delivers screencast frames as JPEG, and those bytes are shaped by
//! web content. This decoder exists so the broker can turn them into raw
//! pixels without linking a C library. It implements exactly what a screencast
//! needs and refuses everything else by name:
//!
//! | Supported | Refused with [`JpegError::Unsupported`] |
//! |---|---|
//! | SOF0 baseline and SOF1 extended sequential, Huffman coded | SOF2 progressive, SOF3 lossless |
//! | 8-bit samples | 12-bit and 16-bit samples |
//! | 1 component (greyscale) or 3 (JFIF YCbCr) | 2 or 4 components (CMYK), Adobe RGB transform |
//! | sampling factors 1 or 2 on each axis | factors 3 and 4 |
//! | DQT 8-bit and 16-bit, DHT, DRI + RST0–7 | arithmetic coding (SOF9–SOF15, DAC) |
//! | interleaved and non-interleaved scans | hierarchical (SOF5–SOF7, DHP, EXP), height from DNL |
//!
//! Output is one `u32` per pixel, `0xFFRRGGBB`.
//!
//! # Choices a decoder has to make, stated
//!
//! * **IDCT**: the separable form of the inverse DCT defined in T.81 §A.3.3,
//!   evaluated in `f32` with a precomputed cosine table. Rows of coefficients
//!   that are entirely zero are skipped.
//! * **Chroma upsampling**: box (sample replication). T.81 leaves upsampling to
//!   the decoder; this matches `djpeg -nosmooth`.
//! * **Colour conversion**: the JFIF / ITU-T T.871 YCbCr → RGB equations,
//!   rounded to nearest and clamped.
//! * **Strictness**: truncated or corrupt data is an error, never a partial
//!   image. Leftover bytes before a restart marker or before the marker that
//!   ends a scan are an error; so is a restart marker with the wrong number.
//!
//! # Limits
//!
//! Width and height 1..=[`MAX_DIMENSION`], at most [`MAX_PIXELS`] pixels.
//! Component planes are allocated at the first scan that uses them, so a
//! header claiming a large image costs nothing until entropy data arrives.

use core::fmt;

/// The largest width or height accepted.
pub const MAX_DIMENSION: u32 = 8192;

/// The largest pixel count accepted.
pub const MAX_PIXELS: u32 = 16_777_216;

/// A decoded image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height` pixels, row by row, each `0xFFRRGGBB`.
    pub pixels: Vec<u32>,
}

/// A coding feature this decoder refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Feature {
    /// Progressive DCT (SOF2, SOF6, SOF10, SOF14).
    Progressive,
    /// Lossless (SOF3, SOF7, SOF11, SOF15).
    Lossless,
    /// Hierarchical / differential (SOF5–SOF7, SOF13–SOF15, DHP, EXP).
    Hierarchical,
    /// Arithmetic entropy coding (SOF9–SOF15, DAC).
    Arithmetic,
    /// A sample precision other than 8 bits.
    Precision(u8),
    /// Four components: CMYK or YCCK.
    Cmyk,
    /// A component count other than 1, 3 or 4.
    Components(u8),
    /// A horizontal or vertical sampling factor above 2.
    SamplingFactor,
    /// A height of zero, to be defined later by a DNL marker.
    HeightFromDnl,
    /// Three components that are RGB rather than YCbCr (Adobe transform 0,
    /// or component identifiers `R`, `G`, `B`).
    RgbTransform,
}

/// Why an image could not be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JpegError {
    /// The data does not start with SOI.
    NotJpeg,
    /// The data ended early.
    Truncated,
    /// A valid coding feature this decoder does not implement.
    Unsupported(Feature),
    /// A marker where none is allowed, or a reserved marker.
    BadMarker {
        /// The marker's second byte.
        marker: u8,
    },
    /// A marker segment whose contents are malformed.
    BadSegment {
        /// The marker's second byte.
        marker: u8,
    },
    /// Zero or oversized dimensions.
    BadDimensions,
    /// A scan refers to a quantisation or Huffman table never defined.
    MissingTable,
    /// A scan appears before any frame header.
    MissingFrame,
    /// Entropy-coded data that does not decode under its tables.
    BadEntropyData,
    /// A restart marker is missing, misplaced or misnumbered.
    BadRestart,
    /// The image ended (EOI) before every component was decoded.
    Incomplete,
}

impl fmt::Display for JpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotJpeg => f.write_str("not a JPEG image"),
            Self::Truncated => f.write_str("JPEG data ends early"),
            Self::Unsupported(feature) => write!(f, "unsupported JPEG feature: {feature:?}"),
            Self::BadMarker { marker } => write!(f, "unexpected JPEG marker 0xFF{marker:02X}"),
            Self::BadSegment { marker } => write!(f, "malformed JPEG segment 0xFF{marker:02X}"),
            Self::BadDimensions => f.write_str("JPEG dimensions are zero or too large"),
            Self::MissingTable => f.write_str("JPEG scan uses an undefined table"),
            Self::MissingFrame => f.write_str("JPEG scan before the frame header"),
            Self::BadEntropyData => f.write_str("corrupt JPEG entropy-coded data"),
            Self::BadRestart => f.write_str("missing or misnumbered JPEG restart marker"),
            Self::Incomplete => f.write_str("JPEG image ended before all components"),
        }
    }
}

impl std::error::Error for JpegError {}

/// Zigzag index → natural (row * 8 + column) index, T.81 Figure A.6.
const ZIGZAG: [u8; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

const SOI: u8 = 0xD8;
const EOI: u8 = 0xD9;
const SOS: u8 = 0xDA;
const DQT: u8 = 0xDB;
const DNL: u8 = 0xDC;
const DRI: u8 = 0xDD;
const DHT: u8 = 0xC4;
const DAC: u8 = 0xCC;
const APP14: u8 = 0xEE;

/// Decode a JPEG image.
///
/// # Errors
/// [`JpegError`] for anything outside the supported subset, and for any
/// truncated or corrupt input.
pub fn decode(data: &[u8]) -> Result<Image, JpegError> {
    if data.get(..2) != Some(&[0xFF, SOI]) {
        return Err(JpegError::NotJpeg);
    }
    let mut d = Decoder::default();
    let mut pos = 2usize;
    loop {
        let marker = next_marker(data, &mut pos)?;
        match marker {
            EOI => return d.finish(),
            SOI | 0xD0..=0xD7 | 0x01 | DNL | 0x02..=0xBF | 0xC8 => {
                return Err(JpegError::BadMarker { marker })
            }
            _ => {}
        }
        let len_bytes = data
            .get(pos..pos.saturating_add(2))
            .ok_or(JpegError::Truncated)?;
        let len = match *len_bytes {
            [hi, lo] => usize::from(u16::from_be_bytes([hi, lo])),
            _ => return Err(JpegError::Truncated),
        };
        if len < 2 {
            return Err(JpegError::BadSegment { marker });
        }
        let body = data
            .get(pos.saturating_add(2)..pos.saturating_add(len))
            .ok_or(JpegError::Truncated)?;
        pos = pos.saturating_add(len);
        match marker {
            0xC0 | 0xC1 => d.frame_header(marker, body)?,
            0xC2 | 0xC6 => return Err(JpegError::Unsupported(Feature::Progressive)),
            0xC3 | 0xC7 => return Err(JpegError::Unsupported(Feature::Lossless)),
            0xC5 | 0xDE | 0xDF => return Err(JpegError::Unsupported(Feature::Hierarchical)),
            0xC9..=0xCB | DAC | 0xCD..=0xCF => {
                return Err(JpegError::Unsupported(Feature::Arithmetic))
            }
            DHT => d.huffman_tables(body)?,
            DQT => d.quantization_tables(body)?,
            DRI => d.restart_interval(body)?,
            SOS => pos = d.scan(data, body, pos)?,
            APP14 => d.adobe(body),
            // APP0-APP15 other than APP14, COM, and the JPGn extension range
            // carry nothing this decoder uses.
            _ => {}
        }
    }
}

/// Read the marker at `pos` (skipping fill bytes) and advance past it.
fn next_marker(data: &[u8], pos: &mut usize) -> Result<u8, JpegError> {
    match data.get(*pos) {
        Some(0xFF) => {}
        Some(&b) => return Err(JpegError::BadMarker { marker: b }),
        None => return Err(JpegError::Truncated),
    }
    let mut i = pos.saturating_add(1);
    while data.get(i) == Some(&0xFF) {
        i = i.saturating_add(1);
    }
    match data.get(i) {
        None => Err(JpegError::Truncated),
        Some(0x00) => Err(JpegError::BadMarker { marker: 0 }),
        Some(&m) => {
            *pos = i.saturating_add(1);
            Ok(m)
        }
    }
}

fn be16(bytes: &[u8], at: usize) -> Option<u16> {
    match *bytes.get(at..at.checked_add(2)?)? {
        [hi, lo] => Some(u16::from_be_bytes([hi, lo])),
        _ => None,
    }
}

#[derive(Default)]
struct Decoder {
    quant: [Option<[u16; 64]>; 4],
    dc: [Option<Huffman>; 4],
    ac: [Option<Huffman>; 4],
    restart_interval: u16,
    frame: Option<Frame>,
    adobe_transform: Option<u8>,
}

struct Frame {
    width: usize,
    height: usize,
    hmax: usize,
    vmax: usize,
    mcus_x: usize,
    mcus_y: usize,
    components: Vec<Component>,
}

struct Component {
    id: u8,
    h: usize,
    v: usize,
    quant_table: usize,
    /// Blocks per line and per column in the interleaved MCU grid; the plane
    /// is `blocks_x * 8` by `blocks_y * 8` samples.
    blocks_x: usize,
    blocks_y: usize,
    plane: Vec<u8>,
    decoded: bool,
}

impl Component {
    fn stride(&self) -> usize {
        self.blocks_x.saturating_mul(8)
    }
}

impl Decoder {
    fn frame_header(&mut self, marker: u8, body: &[u8]) -> Result<(), JpegError> {
        let bad = JpegError::BadSegment { marker };
        if self.frame.is_some() {
            return Err(JpegError::BadMarker { marker });
        }
        let precision = *body.first().ok_or(bad)?;
        if precision != 8 {
            return Err(JpegError::Unsupported(Feature::Precision(precision)));
        }
        let height = be16(body, 1).ok_or(bad)?;
        let width = be16(body, 3).ok_or(bad)?;
        let count = *body.get(5).ok_or(bad)?;
        match count {
            1 | 3 => {}
            4 => return Err(JpegError::Unsupported(Feature::Cmyk)),
            n => return Err(JpegError::Unsupported(Feature::Components(n))),
        }
        if height == 0 {
            return Err(JpegError::Unsupported(Feature::HeightFromDnl));
        }
        if width == 0
            || u32::from(width) > MAX_DIMENSION
            || u32::from(height) > MAX_DIMENSION
            || u32::from(width).saturating_mul(u32::from(height)) > MAX_PIXELS
        {
            return Err(JpegError::BadDimensions);
        }
        if body.len() != 6usize.saturating_add(usize::from(count).saturating_mul(3)) {
            return Err(bad);
        }
        let mut components: Vec<Component> = Vec::with_capacity(usize::from(count));
        for spec in body.get(6..).unwrap_or(&[]).chunks_exact(3) {
            let [id, hv, tq] = *spec else { return Err(bad) };
            let h = usize::from(hv >> 4);
            let v = usize::from(hv & 0x0F);
            if h == 0 || v == 0 || tq > 3 || components.iter().any(|c| c.id == id) {
                return Err(bad);
            }
            if h > 2 || v > 2 {
                return Err(JpegError::Unsupported(Feature::SamplingFactor));
            }
            components.push(Component {
                id,
                h,
                v,
                quant_table: usize::from(tq),
                blocks_x: 0,
                blocks_y: 0,
                plane: Vec::new(),
                decoded: false,
            });
        }
        let hmax = components.iter().map(|c| c.h).max().unwrap_or(1);
        let vmax = components.iter().map(|c| c.v).max().unwrap_or(1);
        let width = usize::from(width);
        let height = usize::from(height);
        let mcus_x = width.div_ceil(hmax.saturating_mul(8));
        let mcus_y = height.div_ceil(vmax.saturating_mul(8));
        for c in &mut components {
            c.blocks_x = mcus_x.saturating_mul(c.h);
            c.blocks_y = mcus_y.saturating_mul(c.v);
        }
        self.frame = Some(Frame {
            width,
            height,
            hmax,
            vmax,
            mcus_x,
            mcus_y,
            components,
        });
        Ok(())
    }

    fn huffman_tables(&mut self, body: &[u8]) -> Result<(), JpegError> {
        let bad = JpegError::BadSegment { marker: DHT };
        let mut rest = body;
        while let Some((&class_id, after)) = rest.split_first() {
            let class = class_id >> 4;
            let id = usize::from(class_id & 0x0F);
            let counts = after.get(..16).ok_or(bad)?;
            let total: usize = counts.iter().map(|&n| usize::from(n)).sum();
            if total > 256 {
                return Err(bad);
            }
            let values = after.get(16..16usize.saturating_add(total)).ok_or(bad)?;
            let table = Huffman::new(counts, values).ok_or(bad)?;
            let slot = match class {
                0 => self.dc.get_mut(id),
                1 => self.ac.get_mut(id),
                _ => None,
            }
            .ok_or(bad)?;
            *slot = Some(table);
            rest = after.get(16usize.saturating_add(total)..).ok_or(bad)?;
        }
        Ok(())
    }

    fn quantization_tables(&mut self, body: &[u8]) -> Result<(), JpegError> {
        let bad = JpegError::BadSegment { marker: DQT };
        let mut rest = body;
        while let Some((&pq_tq, after)) = rest.split_first() {
            let id = usize::from(pq_tq & 0x0F);
            let mut table = [0u16; 64];
            let used = match pq_tq >> 4 {
                0 => {
                    let raw = after.get(..64).ok_or(bad)?;
                    for (t, &b) in table.iter_mut().zip(raw) {
                        *t = u16::from(b);
                    }
                    64usize
                }
                1 => {
                    let raw = after.get(..128).ok_or(bad)?;
                    for (t, pair) in table.iter_mut().zip(raw.chunks_exact(2)) {
                        if let [hi, lo] = *pair {
                            *t = u16::from_be_bytes([hi, lo]);
                        }
                    }
                    128usize
                }
                _ => return Err(bad),
            };
            if table.contains(&0) {
                return Err(bad);
            }
            *self.quant.get_mut(id).ok_or(bad)? = Some(table);
            rest = after.get(used..).ok_or(bad)?;
        }
        Ok(())
    }

    fn restart_interval(&mut self, body: &[u8]) -> Result<(), JpegError> {
        if body.len() != 2 {
            return Err(JpegError::BadSegment { marker: DRI });
        }
        self.restart_interval = be16(body, 0).ok_or(JpegError::BadSegment { marker: DRI })?;
        Ok(())
    }

    fn adobe(&mut self, body: &[u8]) {
        if body.len() >= 12 && body.starts_with(b"Adobe") {
            self.adobe_transform = body.get(11).copied();
        }
    }

    /// Parse a scan header and decode the entropy-coded segment that follows
    /// it, returning the position of the marker that ends the scan.
    fn scan(&mut self, data: &[u8], header: &[u8], start: usize) -> Result<usize, JpegError> {
        let bad = JpegError::BadSegment { marker: SOS };
        let Self {
            quant,
            dc,
            ac,
            restart_interval,
            frame,
            ..
        } = self;
        let frame = frame.as_mut().ok_or(JpegError::MissingFrame)?;
        let count = usize::from(*header.first().ok_or(bad)?);
        if count == 0
            || count > frame.components.len()
            || header.len() != 4usize.saturating_add(count.saturating_mul(2))
        {
            return Err(bad);
        }
        let tail = header
            .get(1usize.saturating_add(count.saturating_mul(2))..)
            .ok_or(bad)?;
        if tail != [0, 63, 0] {
            // Ss=0, Se=63, Ah=Al=0 is the only sequential spectral selection.
            return Err(bad);
        }
        let mut scan: Vec<ScanComponent<'_>> = Vec::with_capacity(count);
        let mut last_index: Option<usize> = None;
        for spec in header.get(1..).unwrap_or(&[]).chunks_exact(2).take(count) {
            let [id, tables] = *spec else { return Err(bad) };
            let index = frame
                .components
                .iter()
                .position(|c| c.id == id)
                .ok_or(bad)?;
            // Components appear in frame order, each at most once (T.81 A.2.3).
            if last_index.is_some_and(|l| index <= l) {
                return Err(bad);
            }
            last_index = Some(index);
            let component = frame.components.get(index).ok_or(bad)?;
            let q = quant
                .get(component.quant_table)
                .copied()
                .flatten()
                .ok_or(JpegError::MissingTable)?;
            let dc_table = dc
                .get(usize::from(tables >> 4))
                .and_then(Option::as_ref)
                .ok_or(JpegError::MissingTable)?;
            let ac_table = ac
                .get(usize::from(tables & 0x0F))
                .and_then(Option::as_ref)
                .ok_or(JpegError::MissingTable)?;
            scan.push(ScanComponent {
                index,
                dc: dc_table,
                ac: ac_table,
                quant: q,
                prediction: 0,
            });
        }

        let interleaved = scan.len() > 1;
        let (mcus_x, mcus_y) = if interleaved {
            let blocks: usize = scan
                .iter()
                .filter_map(|s| frame.components.get(s.index))
                .map(|c| c.h.saturating_mul(c.v))
                .sum();
            if blocks > 10 {
                return Err(bad);
            }
            (frame.mcus_x, frame.mcus_y)
        } else {
            let c = scan
                .first()
                .and_then(|s| frame.components.get(s.index))
                .ok_or(bad)?;
            let w = frame
                .width
                .saturating_mul(c.h)
                .div_ceil(frame.hmax)
                .div_ceil(8);
            let h = frame
                .height
                .saturating_mul(c.v)
                .div_ceil(frame.vmax)
                .div_ceil(8);
            (w, h)
        };

        for s in &scan {
            let c = frame.components.get_mut(s.index).ok_or(bad)?;
            if c.plane.is_empty() {
                let len = c
                    .stride()
                    .checked_mul(c.blocks_y.saturating_mul(8))
                    .ok_or(JpegError::BadDimensions)?;
                c.plane = vec![0; len];
            }
        }

        let idct = Idct::new();
        let mut bits = Bits::new(data, start);
        let mut coefficients = [0f32; 64];
        let mut samples = [[0f32; 8]; 8];
        let interval = usize::from(*restart_interval);
        let mut until_restart = interval;
        let mut next_restart = 0u8;
        for mcu_y in 0..mcus_y {
            for mcu_x in 0..mcus_x {
                if interval > 0 {
                    if until_restart == 0 {
                        bits.restart(0xD0 | next_restart)?;
                        next_restart = next_restart.wrapping_add(1) & 7;
                        for s in &mut scan {
                            s.prediction = 0;
                        }
                        until_restart = interval;
                    }
                    until_restart = until_restart.saturating_sub(1);
                }
                for s in &mut scan {
                    let c = frame.components.get_mut(s.index).ok_or(bad)?;
                    let (per_x, per_y) = if interleaved { (c.h, c.v) } else { (1, 1) };
                    for v in 0..per_y {
                        for h in 0..per_x {
                            let mask = bits.block(s, &mut coefficients)?;
                            idct.transform(&coefficients, mask, &mut samples);
                            let bx = mcu_x.saturating_mul(per_x).saturating_add(h);
                            let by = mcu_y.saturating_mul(per_y).saturating_add(v);
                            // Hoist the stride: reading it inside the call
                            // would borrow `c` mutably (plane) and
                            // immutably (stride) in the same expression.
                            let stride = c.stride();
                            put_block(&mut c.plane, stride, bx, by, &samples)?;
                        }
                    }
                }
            }
        }
        for s in &scan {
            if let Some(c) = frame.components.get_mut(s.index) {
                c.decoded = true;
            }
        }
        bits.end_of_segment()
    }

    fn finish(self) -> Result<Image, JpegError> {
        let frame = self.frame.ok_or(JpegError::MissingFrame)?;
        if frame.components.iter().any(|c| !c.decoded) {
            return Err(JpegError::Incomplete);
        }
        let width = frame.width;
        let mut pixels = vec![0u32; width.saturating_mul(frame.height)];
        match frame.components.as_slice() {
            [luma] => {
                let stride = luma.stride();
                for (row, out) in pixels.chunks_exact_mut(width).enumerate() {
                    let start = row.saturating_mul(stride);
                    let src = luma
                        .plane
                        .get(start..start.saturating_add(width))
                        .ok_or(JpegError::Incomplete)?;
                    for (o, &l) in out.iter_mut().zip(src) {
                        let l = u32::from(l);
                        *o = 0xFF00_0000 | (l << 16) | (l << 8) | l;
                    }
                }
            }
            [y, cb, cr] => {
                let rgb_ids = [y.id, cb.id, cr.id] == *b"RGB";
                if self.adobe_transform == Some(0) || rgb_ids {
                    return Err(JpegError::Unsupported(Feature::RgbTransform));
                }
                let tables = ColorTables::new();
                let mut rows = [vec![0u8; width], vec![0u8; width], vec![0u8; width]];
                for (row, out) in pixels.chunks_exact_mut(width).enumerate() {
                    for (buf, c) in rows.iter_mut().zip([y, cb, cr]) {
                        upsampled_row(c, &frame, row, buf)?;
                    }
                    let [ys, cbs, crs] = &rows;
                    for (((o, &l), &b), &r) in out.iter_mut().zip(ys).zip(cbs).zip(crs) {
                        *o = tables.rgb(l, b, r);
                    }
                }
            }
            _ => return Err(JpegError::Incomplete),
        }
        Ok(Image {
            width: u32::try_from(width).map_err(|_| JpegError::BadDimensions)?,
            height: u32::try_from(frame.height).map_err(|_| JpegError::BadDimensions)?,
            pixels,
        })
    }
}

/// Fill `out` (one output row) with component `c`'s samples for output row
/// `row`, replicating samples where the component is subsampled.
fn upsampled_row(
    c: &Component,
    frame: &Frame,
    row: usize,
    out: &mut [u8],
) -> Result<(), JpegError> {
    let source_row = if c.v < frame.vmax { row >> 1 } else { row };
    let start = source_row.saturating_mul(c.stride());
    let src = c
        .plane
        .get(start..start.saturating_add(c.stride()))
        .ok_or(JpegError::Incomplete)?;
    if c.h < frame.hmax {
        for (pair, &s) in out.chunks_mut(2).zip(src) {
            pair.fill(s);
        }
    } else {
        let src = src.get(..out.len()).ok_or(JpegError::Incomplete)?;
        out.copy_from_slice(src);
    }
    Ok(())
}

fn put_block(
    plane: &mut [u8],
    stride: usize,
    bx: usize,
    by: usize,
    samples: &[[f32; 8]; 8],
) -> Result<(), JpegError> {
    let overflow = JpegError::BadDimensions;
    let top = by
        .checked_mul(8)
        .and_then(|r| r.checked_mul(stride))
        .and_then(|r| r.checked_add(bx.checked_mul(8)?))
        .ok_or(overflow)?;
    let mut start = top;
    for row in samples {
        let dst = plane
            .get_mut(start..start.saturating_add(8))
            .ok_or(overflow)?;
        for (d, &s) in dst.iter_mut().zip(row) {
            // Level shift (T.81 A.3.1), round to nearest; `as` saturates, so
            // this also clamps to 0..=255.
            *d = (s + 128.5) as u8;
        }
        start = start.saturating_add(stride);
    }
    Ok(())
}

struct ScanComponent<'t> {
    index: usize,
    dc: &'t Huffman,
    ac: &'t Huffman,
    quant: [u16; 64],
    prediction: i32,
}

const FAST_BITS: u32 = 9;

/// A Huffman decoding table, T.81 Annex C and §F.2.2.3.
struct Huffman {
    /// Indexed by the next [`FAST_BITS`] bits: `(length << 8) | symbol` for a
    /// code of at most `FAST_BITS` bits, 0 otherwise.
    fast: Vec<u16>,
    /// Per code length 1..=16 (index = length): the largest code, or -1.
    max_code: [i32; 17],
    /// Per code length: the smallest code.
    min_code: [u32; 17],
    /// Per code length: index in `values` of the smallest code's symbol.
    first_value: [u32; 17],
    values: Vec<u8>,
}

impl Huffman {
    /// Build from the 16 code-length counts and the symbol values. `None` when
    /// the counts describe more codes than fit their lengths.
    fn new(counts: &[u8], values: &[u8]) -> Option<Self> {
        let mut table = Self {
            fast: vec![0; 1 << FAST_BITS],
            max_code: [-1; 17],
            min_code: [0; 17],
            first_value: [0; 17],
            values: values.to_vec(),
        };
        let mut code = 0u32;
        let mut k = 0u32;
        for (len, &n) in (1u32..=16).zip(counts) {
            let n = u32::from(n);
            let slot = usize::try_from(len).ok()?;
            if n > 0 {
                *table.first_value.get_mut(slot)? = k;
                *table.min_code.get_mut(slot)? = code;
                let end = code.checked_add(n)?;
                if end > (1u32 << len) {
                    return None;
                }
                *table.max_code.get_mut(slot)? = i32::try_from(end.checked_sub(1)?).ok()?;
                if len <= FAST_BITS {
                    let shift = FAST_BITS.checked_sub(len)?;
                    for (i, c) in (code..end).enumerate() {
                        let index = usize::try_from(k).ok()?.checked_add(i)?;
                        let symbol = u16::from(*values.get(index)?);
                        let first = usize::try_from(c.checked_shl(shift)?).ok()?;
                        let span = 1usize.checked_shl(shift)?;
                        let entry = (u16::try_from(len).ok()? << 8) | symbol;
                        table
                            .fast
                            .get_mut(first..first.checked_add(span)?)?
                            .fill(entry);
                    }
                }
                code = end;
                k = k.checked_add(n)?;
            }
            code = code.checked_shl(1)?;
        }
        (usize::try_from(k).ok()? == values.len()).then_some(table)
    }
}

/// Entropy-coded segment reader: removes byte stuffing, stops at markers.
struct Bits<'a> {
    data: &'a [u8],
    /// Next unread byte. When `stopped`, the 0xFF of the marker (or the end).
    pos: usize,
    /// Buffered bits, most significant first.
    buffer: u64,
    count: u32,
    /// How many of the lowest buffered bits are 1-bits invented past the end
    /// of the segment. Consuming any of them means the data was short.
    invented: u32,
    stopped: bool,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8], pos: usize) -> Self {
        Self {
            data,
            pos,
            buffer: 0,
            count: 0,
            invented: 0,
            stopped: false,
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        if self.stopped {
            return None;
        }
        let b = *self.data.get(self.pos)?;
        if b != 0xFF {
            self.pos = self.pos.saturating_add(1);
            return Some(b);
        }
        let mut i = self.pos.saturating_add(1);
        while self.data.get(i) == Some(&0xFF) {
            i = i.saturating_add(1);
        }
        if self.data.get(i) == Some(&0x00) {
            // A stuffed 0xFF data byte (F.1.2.3).
            self.pos = i.saturating_add(1);
            Some(0xFF)
        } else {
            None
        }
    }

    fn refill(&mut self) {
        while self.count <= 56 {
            let shift = 56u32.saturating_sub(self.count);
            let byte = match self.next_byte() {
                Some(b) => b,
                None => {
                    self.stopped = true;
                    self.invented = self.invented.saturating_add(8);
                    0xFF
                }
            };
            self.buffer |= u64::from(byte).wrapping_shl(shift);
            self.count = self.count.saturating_add(8);
        }
    }

    fn peek16(&self) -> u32 {
        (self.buffer >> 48) as u32
    }

    fn consume(&mut self, n: u32) -> Result<(), JpegError> {
        if n > self.count.saturating_sub(self.invented) {
            return Err(JpegError::Truncated);
        }
        self.buffer = self.buffer.checked_shl(n).unwrap_or(0);
        self.count = self.count.saturating_sub(n);
        Ok(())
    }

    fn symbol(&mut self, table: &Huffman) -> Result<u8, JpegError> {
        if self.count < 32 {
            self.refill();
        }
        let peek = self.peek16();
        let fast = table
            .fast
            .get(usize::try_from(peek >> (16 - FAST_BITS)).unwrap_or(0))
            .copied()
            .unwrap_or(0);
        if fast != 0 {
            self.consume(u32::from(fast >> 8))?;
            return Ok((fast & 0xFF) as u8);
        }
        for len in (FAST_BITS + 1)..=16 {
            let slot = usize::try_from(len).unwrap_or(0);
            let max = table.max_code.get(slot).copied().unwrap_or(-1);
            // len ranges over an inclusive literal bound, so the shift amount
            // stays in 0..16 and the subtraction cannot underflow.
            #[allow(clippy::arithmetic_side_effects)]
            let code = peek.checked_shr(16 - len).unwrap_or(0);
            if u32::try_from(max).is_ok_and(|m| code <= m) {
                let min = table.min_code.get(slot).copied().unwrap_or(0);
                let offset = code.checked_sub(min).ok_or(JpegError::BadEntropyData)?;
                let index = table
                    .first_value
                    .get(slot)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(offset);
                let symbol = table
                    .values
                    .get(usize::try_from(index).unwrap_or(usize::MAX))
                    .copied()
                    .ok_or(JpegError::BadEntropyData)?;
                self.consume(len)?;
                return Ok(symbol);
            }
        }
        Err(JpegError::BadEntropyData)
    }

    /// RECEIVE and EXTEND (F.2.2.1) for a magnitude category of 1..=16 bits.
    fn signed(&mut self, size: u32) -> Result<i32, JpegError> {
        if self.count < 16 {
            self.refill();
        }
        let raw = self
            .peek16()
            .checked_shr(16u32.saturating_sub(size))
            .unwrap_or(0);
        self.consume(size)?;
        let raw = i32::try_from(raw).map_err(|_| JpegError::BadEntropyData)?;
        let half = 1i32.checked_shl(size.saturating_sub(1)).unwrap_or(0);
        if raw < half {
            let full = 1i32.checked_shl(size).unwrap_or(0).wrapping_sub(1);
            Ok(raw.wrapping_sub(full))
        } else {
            Ok(raw)
        }
    }

    /// Decode one data unit into dequantised coefficients in natural order.
    /// Returns a mask with bit `v` set when coefficient row `v` is non-zero.
    fn block(&mut self, s: &mut ScanComponent<'_>, out: &mut [f32; 64]) -> Result<u8, JpegError> {
        *out = [0.0; 64];
        let category = self.symbol(s.dc)?;
        if category > 11 {
            return Err(JpegError::BadEntropyData);
        }
        let diff = if category == 0 {
            0
        } else {
            self.signed(u32::from(category))?
        };
        s.prediction = s.prediction.wrapping_add(diff);
        let mut mask = 0u8;
        if s.prediction != 0 {
            let q = s.quant.first().copied().unwrap_or(1);
            if let Some(slot) = out.first_mut() {
                *slot = s.prediction as f32 * f32::from(q);
            }
            mask = 1;
        }
        let mut k = 1usize;
        while k < 64 {
            let rs = self.symbol(s.ac)?;
            let run = usize::from(rs >> 4);
            let size = u32::from(rs & 0x0F);
            if size == 0 {
                match run {
                    0 => break, // EOB
                    15 => {
                        k = k.saturating_add(16);
                        if k > 64 {
                            return Err(JpegError::BadEntropyData);
                        }
                        continue;
                    }
                    _ => return Err(JpegError::BadEntropyData),
                }
            }
            if size > 10 {
                return Err(JpegError::BadEntropyData);
            }
            k = k.saturating_add(run);
            let natural = usize::from(*ZIGZAG.get(k).ok_or(JpegError::BadEntropyData)?);
            let value = self.signed(size)?;
            let q = s.quant.get(k).copied().unwrap_or(1);
            if let Some(slot) = out.get_mut(natural) {
                *slot = value as f32 * f32::from(q);
            }
            mask |= 1u8.wrapping_shl(u32::try_from(natural >> 3).unwrap_or(0));
            k = k.saturating_add(1);
        }
        Ok(mask)
    }

    /// Check that only final-byte padding is left, then return the position
    /// of the marker that follows.
    fn marker_position(&self) -> Result<usize, JpegError> {
        if self.count.saturating_sub(self.invented) >= 8 {
            return Err(JpegError::BadEntropyData);
        }
        match self.data.get(self.pos) {
            Some(0xFF) => Ok(self.pos),
            Some(_) => Err(JpegError::BadEntropyData),
            None => Err(JpegError::Truncated),
        }
    }

    /// Consume the restart marker `expected` and reset the reader.
    fn restart(&mut self, expected: u8) -> Result<(), JpegError> {
        let mut i = self.marker_position().map_err(|e| match e {
            JpegError::Truncated => e,
            _ => JpegError::BadRestart,
        })?;
        while self.data.get(i) == Some(&0xFF) {
            i = i.saturating_add(1);
        }
        match self.data.get(i) {
            Some(&m) if m == expected => {}
            None => return Err(JpegError::Truncated),
            Some(_) => return Err(JpegError::BadRestart),
        }
        self.pos = i.saturating_add(1);
        self.buffer = 0;
        self.count = 0;
        self.invented = 0;
        self.stopped = false;
        Ok(())
    }

    fn end_of_segment(&self) -> Result<usize, JpegError> {
        self.marker_position()
    }
}

/// The inverse DCT of T.81 §A.3.3 in separable form.
struct Idct {
    /// `basis[x][u] = C(u)/2 · cos((2x+1)uπ/16)`, C(0) = 1/√2, else 1.
    basis: [[f32; 8]; 8],
}

impl Idct {
    fn new() -> Self {
        let mut basis = [[0f32; 8]; 8];
        for (x, row) in basis.iter_mut().enumerate() {
            for (u, cell) in row.iter_mut().enumerate() {
                let cu = if u == 0 {
                    core::f64::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
                let angle = (2.0 * x as f64 + 1.0) * u as f64 * core::f64::consts::PI / 16.0;
                *cell = (cu / 2.0 * angle.cos()) as f32;
            }
        }
        Self { basis }
    }

    /// `coefficients` in natural order (row = vertical frequency); `rows` has
    /// bit `v` set when row `v` holds a non-zero coefficient. Output samples
    /// are not level-shifted.
    fn transform(&self, coefficients: &[f32; 64], rows: u8, out: &mut [[f32; 8]; 8]) {
        let mut horizontal = [[0f32; 8]; 8];
        for (v, (dst, coef_row)) in horizontal
            .iter_mut()
            .zip(coefficients.chunks_exact(8))
            .enumerate()
        {
            if !bit(rows, v) {
                continue;
            }
            for (d, basis_x) in dst.iter_mut().zip(&self.basis) {
                *d = basis_x.iter().zip(coef_row).map(|(b, c)| b * c).sum();
            }
        }
        *out = [[0f32; 8]; 8];
        for (v, src) in horizontal.iter().enumerate() {
            if !bit(rows, v) {
                continue;
            }
            for (out_row, basis_y) in out.iter_mut().zip(&self.basis) {
                let weight = basis_y.get(v).copied().unwrap_or(0.0);
                for (o, s) in out_row.iter_mut().zip(src) {
                    *o += weight * s;
                }
            }
        }
    }
}

fn bit(mask: u8, index: usize) -> bool {
    u32::try_from(index)
        .ok()
        .and_then(|i| mask.checked_shr(i))
        .is_some_and(|m| m & 1 != 0)
}

/// YCbCr → RGB per JFIF (ITU-T T.871 §7).
struct ColorTables {
    cr_r: [f32; 256],
    cb_b: [f32; 256],
    cb_g: [f32; 256],
    cr_g: [f32; 256],
}

impl ColorTables {
    fn new() -> Self {
        let mut t = Self {
            cr_r: [0.0; 256],
            cb_b: [0.0; 256],
            cb_g: [0.0; 256],
            cr_g: [0.0; 256],
        };
        for (i, (((r, b), gb), gr)) in t
            .cr_r
            .iter_mut()
            .zip(t.cb_b.iter_mut())
            .zip(t.cb_g.iter_mut())
            .zip(t.cr_g.iter_mut())
            .enumerate()
        {
            let c = i as f32 - 128.0;
            *r = 1.402 * c;
            *b = 1.772 * c;
            *gb = -0.344_136 * c;
            *gr = -0.714_136 * c;
        }
        t
    }

    fn rgb(&self, y: u8, cb: u8, cr: u8) -> u32 {
        let y = f32::from(y) + 0.5;
        let cb = usize::from(cb);
        let cr = usize::from(cr);
        let r = y + self.cr_r.get(cr).copied().unwrap_or(0.0);
        let g = y
            + self.cb_g.get(cb).copied().unwrap_or(0.0)
            + self.cr_g.get(cr).copied().unwrap_or(0.0);
        let b = y + self.cb_b.get(cb).copied().unwrap_or(0.0);
        // `as u8` truncates and saturates: with the 0.5 added above that is
        // round-to-nearest clamped to 0..=255.
        0xFF00_0000 | (u32::from(r as u8) << 16) | (u32::from(g as u8) << 8) | u32::from(b as u8)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    macro_rules! fixture {
        ($name:literal) => {
            (
                $name,
                include_bytes!(concat!("../testdata/jpeg/", $name, ".jpg")).as_slice(),
                include_bytes!(concat!("../testdata/jpeg/", $name, ".ref.pnm")).as_slice(),
            )
        };
    }

    /// Every fixture with a djpeg reference.
    fn fixtures() -> Vec<(&'static str, &'static [u8], &'static [u8])> {
        vec![
            fixture!("s420-q75"),
            fixture!("s422-q75"),
            fixture!("s440-q75"),
            fixture!("s444-q75"),
            fixture!("gray-q75"),
            fixture!("s420-restart1"),
            fixture!("s420-restart3b"),
            fixture!("s420-optimize"),
            fixture!("s420-q10"),
            fixture!("s420-q95"),
            fixture!("s420-q2-sof1"),
            fixture!("s444-3scans"),
            fixture!("s420-3scans"),
            fixture!("big-s420-q85"),
            fixture!("big-s444-q85"),
            fixture!("big-gray-q85"),
        ]
    }

    /// (width, height, channels, samples) from a binary PPM (P6) or PGM (P5).
    fn pnm(bytes: &[u8]) -> (u32, u32, usize, &[u8]) {
        let mut fields = Vec::new();
        let mut i = 0;
        while fields.len() < 4 {
            while bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            let start = i;
            while !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            fields.push(std::str::from_utf8(&bytes[start..i]).expect("ascii"));
        }
        let channels = match fields[0] {
            "P6" => 3,
            "P5" => 1,
            other => panic!("not a binary PNM: {other}"),
        };
        assert_eq!(fields[3], "255");
        let w: u32 = fields[1].parse().expect("width");
        let h: u32 = fields[2].parse().expect("height");
        (w, h, channels, &bytes[i + 1..])
    }

    /// Largest per-channel difference against djpeg -nosmooth -dct int.
    ///
    /// djpeg's `int` DCT is the islow integer approximation; our IDCT is f32
    /// arithmetic close to the exact transform, so per-sample disagreement is
    /// inherent, not a defect. Measured envelope over every fixture: max 3,
    /// mean < 0.04 — an exact match would require cloning islow bit-for-bit.
    const MAX_CHANNEL_DIFF: u32 = 3;
    /// Mean per-channel absolute difference against the same reference.
    const MAX_MEAN_DIFF: f64 = 0.25;

    #[test]
    fn output_matches_djpeg_within_the_stated_tolerance() {
        for (name, jpg, reference) in fixtures() {
            let image = decode(jpg).unwrap_or_else(|e| panic!("{name}: {e}"));
            let (w, h, channels, samples) = pnm(reference);
            assert_eq!((image.width, image.height), (w, h), "{name}");
            assert_eq!(samples.len(), (w * h) as usize * channels, "{name}");
            let mut max = 0u32;
            let mut total = 0u64;
            for (i, &p) in image.pixels.iter().enumerate() {
                assert_eq!(p >> 24, 0xFF, "{name}: alpha");
                let ours = [(p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF];
                let theirs: [u32; 3] = if channels == 3 {
                    [0, 1, 2].map(|c| u32::from(samples[i * 3 + c]))
                } else {
                    [u32::from(samples[i]); 3]
                };
                for c in 0..3 {
                    let d = ours[c].abs_diff(theirs[c]);
                    max = max.max(d);
                    total += u64::from(d);
                }
            }
            let mean = total as f64 / (image.pixels.len() * 3) as f64;
            eprintln!("{name}: {w}x{h} max diff {max}, mean diff {mean:.4}");
            assert!(max <= MAX_CHANNEL_DIFF, "{name}: max diff {max}");
            assert!(mean <= MAX_MEAN_DIFF, "{name}: mean diff {mean}");
        }
    }

    #[test]
    fn refused_encodings_are_named() {
        let cases: [(&str, &[u8], Feature); 4] = [
            (
                "progressive",
                include_bytes!("../testdata/jpeg/reject-progressive.jpg"),
                Feature::Progressive,
            ),
            (
                "arithmetic",
                include_bytes!("../testdata/jpeg/reject-arithmetic.jpg"),
                Feature::Arithmetic,
            ),
            (
                "12-bit",
                include_bytes!("../testdata/jpeg/reject-12bit.jpg"),
                Feature::Precision(12),
            ),
            (
                "lossless",
                include_bytes!("../testdata/jpeg/reject-lossless.jpg"),
                Feature::Lossless,
            ),
        ];
        for (name, data, feature) in cases {
            assert_eq!(decode(data), Err(JpegError::Unsupported(feature)), "{name}");
        }
    }

    /// A minimal stream: SOI, one frame header with the given fields, EOI.
    fn with_frame(
        marker: u8,
        precision: u8,
        width: u16,
        height: u16,
        comps: &[(u8, u8)],
    ) -> Vec<u8> {
        let mut v = vec![0xFF, SOI, 0xFF, marker];
        let len = 8 + 3 * comps.len() as u16;
        v.extend_from_slice(&len.to_be_bytes());
        v.push(precision);
        v.extend_from_slice(&height.to_be_bytes());
        v.extend_from_slice(&width.to_be_bytes());
        v.push(comps.len() as u8);
        for &(id, hv) in comps {
            v.extend_from_slice(&[id, hv, 0]);
        }
        v.extend_from_slice(&[0xFF, EOI]);
        v
    }

    #[test]
    fn header_level_refusals_are_specific() {
        let ycc = [(1, 0x22), (2, 0x11), (3, 0x11)];
        let cmyk = [(1, 0x11), (2, 0x11), (3, 0x11), (4, 0x11)];
        assert_eq!(
            decode(&with_frame(0xC0, 8, 16, 16, &cmyk)),
            Err(JpegError::Unsupported(Feature::Cmyk))
        );
        assert_eq!(
            decode(&with_frame(0xC0, 8, 16, 16, &[(1, 0x11), (2, 0x11)])),
            Err(JpegError::Unsupported(Feature::Components(2)))
        );
        assert_eq!(
            decode(&with_frame(0xC1, 12, 16, 16, &ycc)),
            Err(JpegError::Unsupported(Feature::Precision(12)))
        );
        assert_eq!(
            decode(&with_frame(0xC0, 8, 16, 0, &ycc)),
            Err(JpegError::Unsupported(Feature::HeightFromDnl))
        );
        assert_eq!(
            decode(&with_frame(
                0xC0,
                8,
                16,
                16,
                &[(1, 0x31), (2, 0x11), (3, 0x11)]
            )),
            Err(JpegError::Unsupported(Feature::SamplingFactor))
        );
        assert_eq!(
            decode(&with_frame(0xC5, 8, 16, 16, &ycc)),
            Err(JpegError::Unsupported(Feature::Hierarchical))
        );
        assert_eq!(
            decode(&with_frame(0xCA, 8, 16, 16, &ycc)),
            Err(JpegError::Unsupported(Feature::Arithmetic))
        );
        assert_eq!(
            decode(&with_frame(0xC0, 8, 0, 16, &ycc)),
            Err(JpegError::BadDimensions)
        );
        assert_eq!(
            decode(&with_frame(0xC0, 8, 8193, 1, &ycc)),
            Err(JpegError::BadDimensions)
        );
        assert_eq!(
            decode(&with_frame(0xC0, 8, 8192, 2049, &ycc)),
            Err(JpegError::BadDimensions),
            "more than 16 777 216 pixels"
        );
        assert_eq!(
            decode(&with_frame(0xC0, 8, 4096, 4096, &ycc)),
            Err(JpegError::Incomplete),
            "exactly the pixel limit is accepted, then EOI arrives with no scan"
        );
        assert_eq!(
            decode(&with_frame(
                0xC0,
                8,
                16,
                16,
                &[(1, 0x11), (1, 0x11), (3, 0x11)]
            )),
            Err(JpegError::BadSegment { marker: 0xC0 }),
            "duplicate component identifiers"
        );
        assert_eq!(decode(b"\x89PNG\r\n"), Err(JpegError::NotJpeg));
        assert_eq!(
            decode(&[0xFF, SOI, 0xFF, EOI]),
            Err(JpegError::MissingFrame)
        );
        assert_eq!(
            decode(&[0xFF, SOI, 0xFF, SOS, 0, 8, 1, 1, 0, 0, 63, 0]),
            Err(JpegError::MissingFrame)
        );
        assert_eq!(
            decode(&[0xFF, SOI, 0xFF, 0xD0]),
            Err(JpegError::BadMarker { marker: 0xD0 })
        );
    }

    #[test]
    fn a_scan_without_its_tables_is_refused() {
        let (_, jpg, _) = fixture!("gray-q75");
        // Remove every DHT segment.
        let mut out = jpg[..2].to_vec();
        let mut i = 2;
        loop {
            let marker = jpg[i + 1];
            let len = usize::from(u16::from_be_bytes([jpg[i + 2], jpg[i + 3]]));
            if marker != DHT {
                out.extend_from_slice(&jpg[i..i + 2 + len]);
            }
            i += 2 + len;
            if marker == SOS {
                out.extend_from_slice(&jpg[i..]);
                break;
            }
        }
        assert_eq!(decode(&out), Err(JpegError::MissingTable));
    }

    #[test]
    fn rgb_encoded_jpegs_are_refused_rather_than_miscoloured() {
        let (_, jpg, _) = fixture!("s444-q75");
        let mut adobe = vec![0xFF, SOI, 0xFF, APP14, 0, 14];
        adobe.extend_from_slice(b"Adobe\x00\x64\x00\x00\x00\x00\x00");
        adobe.extend_from_slice(&jpg[2..]);
        assert_eq!(
            decode(&adobe),
            Err(JpegError::Unsupported(Feature::RgbTransform))
        );
    }

    #[test]
    fn every_truncation_is_an_error_and_none_panics() {
        for (name, jpg, _) in [fixture!("s420-restart1"), fixture!("s422-q75")] {
            for n in 0..jpg.len() {
                assert!(
                    decode(&jpg[..n]).is_err(),
                    "{name} truncated to {n} decoded"
                );
            }
            assert!(decode(jpg).is_ok());
        }
    }

    #[test]
    fn a_wrong_restart_marker_is_refused() {
        let (_, jpg, _) = fixture!("s420-restart3b");
        let rst = jpg
            .windows(2)
            .position(|w| w == [0xFF, 0xD0])
            .expect("the fixture has restart markers");
        let mut bad = jpg.to_vec();
        bad[rst + 1] = 0xD1;
        assert_eq!(decode(&bad), Err(JpegError::BadRestart));
    }

    struct XorShift(u64);

    impl XorShift {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    #[test]
    fn random_bit_flips_never_panic() {
        let sources = [
            fixture!("s420-q75"),
            fixture!("s420-restart1"),
            fixture!("big-gray-q85"),
        ];
        let mut rng = XorShift(0x2545_F491_4F6C_DD1D);
        let mut outcomes = [0usize; 2];
        for round in 0..6_000 {
            let (name, jpg, _) = sources[round % sources.len()];
            let mut data = jpg.to_vec();
            for _ in 0..1 + rng.below(3) {
                let i = rng.below(data.len());
                data[i] ^= 1 << rng.below(8);
            }
            match decode(&data) {
                Ok(image) => {
                    outcomes[0] += 1;
                    assert_eq!(
                        image.pixels.len(),
                        (image.width * image.height) as usize,
                        "{name} round {round}"
                    );
                }
                Err(_) => outcomes[1] += 1,
            }
        }
        // Both outcomes must occur for the loop to have exercised anything.
        assert!(outcomes[0] > 0 && outcomes[1] > 0, "{outcomes:?}");
    }

    #[test]
    fn the_zigzag_table_walks_the_antidiagonals() {
        let mut expected = Vec::new();
        for d in 0..15usize {
            let cells: Vec<usize> = (0..=d)
                .filter(|&r| r < 8 && d - r < 8)
                .map(|r| r * 8 + (d - r))
                .collect();
            // Even diagonals run bottom-left to top-right (row decreasing).
            if d % 2 == 0 {
                expected.extend(cells.iter().rev());
            } else {
                expected.extend(cells.iter());
            }
        }
        let table: Vec<usize> = ZIGZAG.iter().map(|&z| usize::from(z)).collect();
        assert_eq!(table, expected);
    }

    #[test]
    fn a_dc_only_block_is_flat() {
        let idct = Idct::new();
        let mut coefficients = [0f32; 64];
        coefficients[0] = 80.0; // DC of 80 → each sample 80/8 = 10
        let mut out = [[0f32; 8]; 8];
        idct.transform(&coefficients, 1, &mut out);
        for row in out {
            for s in row {
                assert!((s - 10.0).abs() < 1e-4, "{s}");
            }
        }
    }

    #[test]
    fn huffman_counts_that_overflow_their_lengths_are_refused() {
        // Three codes of length 1 cannot exist.
        let mut counts = [0u8; 16];
        counts[0] = 3;
        assert!(Huffman::new(&counts, &[1, 2, 3]).is_none());
        // Two codes of length 1 are the whole code space and are accepted.
        counts[0] = 2;
        assert!(Huffman::new(&counts, &[1, 2]).is_some());
        // Counts and values must agree.
        assert!(Huffman::new(&counts, &[1]).is_none());
    }
}
