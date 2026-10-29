//! A real reader for a page on this machine.
//!
//! Litric Browser's web engine is Chromium over the broker, and this image has
//! no Chromium. What the window can do by itself, with no engine and no network,
//! is open a page stored on the machine: read its bytes, parse its headings,
//! paragraphs and pictures, lay them out, decode the pictures with
//! [`litric_browser::jpeg`] and draw them. That is a real path — a real file,
//! real bytes, a real parse, real pixels — and it is what this module wires.
//!
//! # Bounds
//!
//! * The document is at most [`litric_browser::html::MAX_DOCUMENT_BYTES`].
//! * At most [`MAX_CACHED_IMAGES`] decoded pictures are kept, and at most
//!   [`MAX_CACHED_PIXELS`] decoded pixels in total (32 MiB at four bytes each).
//! * A picture is decoded only when its block is inside the viewport, and at
//!   most [`MAX_DECODES_PER_FRAME`] new pictures are decoded in one frame, so a
//!   page full of panels cannot stall a frame.
//! * One picture is at most [`MAX_IMAGE_BYTES`] on disk.
//!
//! Nothing here polls, spawns a thread or allocates per frame beyond the
//! strings the visible text needs to be drawn.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

use litric_browser::html::{self, Block};
use litric_design::TypeStep;

use crate::font::Style;

use crate::canvas::{Canvas, Rect};
use crate::font;
use crate::ui::Tokens;
use litric_design::Radius;

/// The most decoded pictures kept at once.
pub const MAX_CACHED_IMAGES: usize = 4;

/// The most decoded pixels kept at once: 8 million, 32 MiB at four bytes each.
pub const MAX_CACHED_PIXELS: usize = 8_000_000;

/// The largest picture file that will be decoded.
pub const MAX_IMAGE_BYTES: u64 = 16 * 1024 * 1024;

/// How many new pictures one frame may decode.
pub const MAX_DECODES_PER_FRAME: usize = 2;

/// The space between the page's edge and its text, in pixels.
const MARGIN: f32 = 24.0;

/// One laid-out block.
#[derive(Debug, Clone)]
struct Laid {
    y: f32,
    h: f32,
    kind: LaidKind,
}

/// What a laid-out block draws.
#[derive(Debug, Clone)]
enum LaidKind {
    Text {
        lines: Vec<String>,
        line_h: f32,
        heading: bool,
    },
    Image {
        path: PathBuf,
        alt: String,
        w: f32,
        h: f32,
    },
    Rule,
}

/// A page open in the window.
#[derive(Debug)]
pub struct Reader {
    url: String,
    title: String,
    directory: PathBuf,
    source: Vec<Block>,
    laid: Vec<Laid>,
    layout_width: f32,
    scroll: f32,
    content_h: f32,
    images: HashMap<usize, litric_browser::jpeg::Image>,
    order: VecDeque<usize>,
    pixels_cached: usize,
    decoded: usize,
}

impl Reader {
    /// Open the page at `url`. Only `file://` addresses are read here; anything
    /// else is the engine's business and is refused with a plain sentence.
    ///
    /// # Errors
    /// A sentence for the window when the file is missing, unreadable or over
    /// the document limit.
    pub fn open(url: &str) -> Result<Self, String> {
        let Some(rest) = url.strip_prefix("file://") else {
            return Err("this page is not a local file".to_owned());
        };
        let rest = rest.split(['?', '#']).next().unwrap_or(rest);
        let decoded = percent_decode(rest);
        let path = PathBuf::from(decoded);
        let meta = std::fs::metadata(&path).map_err(|_| "the file could not be read".to_owned())?;
        if !meta.is_file() || meta.len() > html::MAX_DOCUMENT_BYTES as u64 {
            return Err("the page is too large to open".to_owned());
        }
        let bytes = std::fs::read(&path).map_err(|_| "the file could not be read".to_owned())?;
        let document = html::parse(&bytes);
        let directory = path
            .parent()
            .map_or_else(|| PathBuf::from("/"), Path::to_path_buf);
        let title = document
            .title
            .or_else(|| {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
            })
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| "Page".to_owned());
        Ok(Self {
            url: url.to_owned(),
            title,
            directory,
            source: document.blocks,
            laid: Vec::new(),
            layout_width: 0.0,
            scroll: 0.0,
            content_h: 0.0,
            images: HashMap::new(),
            order: VecDeque::new(),
            pixels_cached: 0,
            decoded: 0,
        })
    }

    /// The address this page was opened from.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The page's own title, or its file name when it has none.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// How tall the laid-out page is, in pixels.
    #[must_use]
    pub fn content_height(&self) -> f32 {
        self.content_h
    }

    /// How far down the page is scrolled, in pixels.
    #[must_use]
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    /// How many pictures are decoded and cached right now.
    #[must_use]
    pub fn cached_images(&self) -> usize {
        self.images.len()
    }

    /// How many decoded pixels are cached right now.
    #[must_use]
    pub fn cached_pixels(&self) -> usize {
        self.pixels_cached
    }

    /// How many pictures have been decoded since the page opened.
    #[must_use]
    pub fn decoded_images(&self) -> usize {
        self.decoded
    }

    /// Scroll by `delta` pixels, clamped to the page.
    pub fn scroll_by(&mut self, delta: f32, viewport_h: f32) {
        let max = (self.content_h - viewport_h).max(0.0);
        self.scroll = (self.scroll + delta).clamp(0.0, max);
    }

    /// The lines of text that are on screen at the current scroll position.
    #[must_use]
    pub fn visible_text(&self, viewport_h: f32) -> Vec<String> {
        let top = self.scroll;
        let bottom = self.scroll + viewport_h.max(0.0);
        let mut out = Vec::new();
        for laid in &self.laid {
            if laid.y + laid.h <= top || laid.y >= bottom {
                continue;
            }
            if let LaidKind::Text { lines, .. } = &laid.kind {
                out.extend(lines.iter().cloned());
            }
        }
        out
    }

    /// The rectangle of the `ordinal`th picture, in page coordinates (add the
    /// page's own origin to get window coordinates).
    #[must_use]
    pub fn image_area(&self, ordinal: usize) -> Option<Rect> {
        let mut seen = 0usize;
        for laid in &self.laid {
            if let LaidKind::Image { w, h, .. } = &laid.kind {
                if seen == ordinal {
                    return Some(Rect::new(MARGIN, laid.y - self.scroll, *w, *h));
                }
                seen = seen.saturating_add(1);
            }
        }
        None
    }

    /// Lay the page out for `width`, when the width changed.
    fn layout(&mut self, width: f32, tokens: &Tokens) {
        let body = Style::new(tokens.text_px(TypeStep::Footnote), tokens.text());
        let heading = Style::bold(tokens.text_px(TypeStep::Title3), tokens.text());
        let mut y = MARGIN;
        let mut laid = Vec::with_capacity(self.source.len());
        for block in &self.source {
            match block {
                Block::Heading { text, level } => {
                    let scale = match level {
                        1 => 1.12,
                        2 => 1.0,
                        _ => 0.9,
                    };
                    let style = if (*level) == 1 {
                        heading
                    } else {
                        Style::bold(tokens.text_px(TypeStep::Body), tokens.text())
                    };
                    let size = style.size * scale;
                    let lines = wrap_at(text, &style, width, 40);
                    let line_h = size * 1.35;
                    let h = (lines.len() as f32).max(1.0) * line_h + 14.0;
                    laid.push(Laid {
                        y,
                        h,
                        kind: LaidKind::Text {
                            lines,
                            line_h,
                            heading: true,
                        },
                    });
                    y += h;
                }
                Block::Paragraph(text) => {
                    let lines = wrap_at(text, &body, width, 400);
                    let line_h = body.size * 1.5;
                    let h = (lines.len() as f32).max(1.0) * line_h + 10.0;
                    laid.push(Laid {
                        y,
                        h,
                        kind: LaidKind::Text {
                            lines,
                            line_h,
                            heading: false,
                        },
                    });
                    y += h;
                }
                Block::Image {
                    source,
                    alt,
                    width: declared_w,
                    height: declared_h,
                } => {
                    let aspect = match (declared_w, declared_h) {
                        (Some(w), Some(h)) if *w > 0 && *h > 0 => *h as f32 / *w as f32,
                        _ => 0.66,
                    };
                    let h = (width * aspect).clamp(0.1, 2_000.0);
                    let path = self
                        .directory
                        .join(source.trim_start_matches('/').replace("//", "/"));
                    laid.push(Laid {
                        y,
                        h,
                        kind: LaidKind::Image {
                            path,
                            alt: alt.clone(),
                            w: width,
                            h,
                        },
                    });
                    y += h;
                }
                Block::Rule => {
                    laid.push(Laid {
                        y,
                        h: 17.0,
                        kind: LaidKind::Rule,
                    });
                    y += 17.0;
                }
            }
        }
        self.content_h = y + MARGIN;
        self.laid = laid;
        self.layout_width = width;
        let max = (self.content_h - 1.0).max(0.0);
        self.scroll = self.scroll.clamp(0.0, max);
    }

    /// Draw the page into `page`, decoding only the pictures the viewport needs.
    pub fn draw(&mut self, canvas: &mut Canvas, tokens: &Tokens, page: Rect) {
        if page.w <= 1.0 || page.h <= 1.0 {
            return;
        }
        let width = (page.w - MARGIN * 2.0).max(80.0);
        if (self.layout_width - width).abs() > 0.5 {
            self.layout(width, tokens);
        }
        let max = (self.content_h - page.h).max(0.0);
        self.scroll = self.scroll.clamp(0.0, max);

        let body = Style::new(tokens.text_px(TypeStep::Footnote), tokens.text());
        let heading = Style::bold(tokens.text_px(TypeStep::Title3), tokens.text());
        let clip = canvas.push_clip(page);
        // Decode at most the frame's budget, and only pictures that are on
        // screen. This vector stays empty on every frame where nothing new
        // appears, so the steady path allocates nothing.
        let mut pending: Vec<(usize, PathBuf)> = Vec::new();
        for (index, laid) in self.laid.iter().enumerate() {
            let top = page.y + laid.y - self.scroll;
            if top + laid.h < page.y || top > page.bottom() {
                continue;
            }
            if let LaidKind::Image { path, .. } = &laid.kind {
                if !self.images.contains_key(&index) && pending.len() < MAX_DECODES_PER_FRAME {
                    pending.push((index, path.clone()));
                }
            }
        }
        for (index, path) in pending {
            self.decode(index, &path);
        }

        for (index, laid) in self.laid.iter().enumerate() {
            let top = page.y + laid.y - self.scroll;
            if top + laid.h < page.y || top > page.bottom() {
                continue;
            }
            match &laid.kind {
                LaidKind::Text {
                    lines,
                    line_h,
                    heading: is_heading,
                } => {
                    let style = if *is_heading && !lines.is_empty() {
                        &heading
                    } else {
                        &body
                    };
                    for (i, line) in lines.iter().enumerate() {
                        let baseline = top + style.size * 0.9 + i as f32 * line_h;
                        font::draw(canvas, line, page.x + MARGIN, baseline, style);
                    }
                }
                LaidKind::Image { alt, w, h, .. } => {
                    if let Some(image) = self.images.get(&index) {
                        canvas.blit_argb(
                            Rect::new(page.x + MARGIN, top, *w, *h),
                            &image.pixels,
                            image.width,
                            image.height,
                        );
                    } else {
                        // Not decoded this frame: a quiet placeholder, never a
                        // misleading blank.
                        canvas.fill_round_rect(
                            Rect::new(page.x + MARGIN, top, *w, *h),
                            tokens.radius(Radius::Sm),
                            tokens.border(),
                        );
                        let alt_style =
                            Style::new(tokens.text_px(TypeStep::Footnote), tokens.text_dim());
                        let label = if alt.is_empty() {
                            "Picture"
                        } else {
                            alt.as_str()
                        };
                        font::draw(
                            canvas,
                            &crate::ui::elide(label, &alt_style, *w),
                            page.x + MARGIN + 8.0,
                            top + 20.0,
                            &alt_style,
                        );
                    }
                }
                LaidKind::Rule => {
                    canvas.fill_round_rect(
                        Rect::new(page.x + MARGIN, top, width, 1.0),
                        0.0,
                        tokens.border(),
                    );
                }
            }
        }
        canvas.set_clip(clip);
    }

    /// Decode one picture into the bounded cache, evicting the oldest.
    fn decode(&mut self, index: usize, path: &Path) {
        let Ok(meta) = std::fs::metadata(path) else {
            return;
        };
        if !meta.is_file() || meta.len() > MAX_IMAGE_BYTES {
            return;
        }
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        let Ok(image) = litric_browser::jpeg::decode(&bytes) else {
            return;
        };
        let pixels = image.pixels.len();
        if pixels == 0 || pixels > MAX_CACHED_PIXELS {
            return;
        }
        self.decoded = self.decoded.saturating_add(1);
        self.pixels_cached = self.pixels_cached.saturating_add(pixels);
        self.images.insert(index, image);
        self.order.push_back(index);
        while self.order.len() > MAX_CACHED_IMAGES || self.pixels_cached > MAX_CACHED_PIXELS {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if oldest == index {
                break;
            }
            if let Some(image) = self.images.remove(&oldest) {
                self.pixels_cached = self.pixels_cached.saturating_sub(image.pixels.len());
            }
        }
    }
}

/// Wrap `text` in `style` to `width`, returning at most `max_lines` lines.
fn wrap_at(text: &str, style: &Style, width: f32, max_lines: usize) -> Vec<String> {
    crate::ui::wrap(text, style, width, max_lines)
}

/// Decode `%20`-style escapes in a file URL's path.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let Some(&b) = bytes.get(i) else {
            break;
        };
        if b == b'%' {
            let hex = bytes.get(i.saturating_add(1)..i.saturating_add(3));
            let decoded = hex
                .and_then(|h| std::str::from_utf8(h).ok())
                .and_then(|h| u8::from_str_radix(h, 16).ok());
            if let Some(byte) = decoded {
                out.push(byte);
                i = i.saturating_add(3);
                continue;
            }
        }
        out.push(b);
        i = i.saturating_add(1);
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_in_a_file_address_are_decoded() {
        assert_eq!(percent_decode("/tmp/a%20b/page.html"), "/tmp/a b/page.html");
        assert_eq!(percent_decode("/tmp/100%"), "/tmp/100%");
        assert_eq!(percent_decode("no-escapes"), "no-escapes");
    }

    #[test]
    fn a_missing_file_is_refused_in_a_sentence() {
        let err = Reader::open("file:///nonexistent/litric-page.html").expect_err("missing");
        assert_eq!(err, "the file could not be read");
        let err = Reader::open("https://example.com/").expect_err("remote");
        assert_eq!(err, "this page is not a local file");
    }

    #[test]
    fn the_cache_bounds_are_pinned() {
        assert_eq!(MAX_CACHED_IMAGES, 4);
        assert_eq!(MAX_CACHED_PIXELS, 8_000_000);
        assert_eq!(MAX_IMAGE_BYTES, 16 * 1024 * 1024);
        assert_eq!(MAX_DECODES_PER_FRAME, 2);
    }
}
