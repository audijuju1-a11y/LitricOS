//! A real reader for a page on this machine.
//!
//! Litric Browser's web engine is Chromium over the broker, and this image has
//! no Chromium. What the window can do by itself, with no engine and no network,
//! is open a page stored on the machine: read its bytes, parse its headings,
//! paragraphs, tables and pictures, lay them out, decode the pictures with
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
use litric_browser::url::{Scheme, Url};
use litric_design::TypeStep;

use crate::font::Style;

use crate::canvas::{rgb, with_alpha, Canvas, Rect};
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
        /// 0 for body text, 1–3 for a heading of that level.
        level: u8,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        col_w: f32,
        row_h: f32,
    },
    Image {
        source: ImageSource,
        alt: String,
        w: f32,
        h: f32,
    },
    Rule,
}

/// Where a page's pictures live.
#[derive(Debug, Clone)]
enum Base {
    /// A page read from a file: pictures are files beside it.
    Directory(PathBuf),
    /// A page served over HTTP: pictures are fetched from the same server.
    Page(Url),
}

impl Base {
    /// Resolve one `<img src>` from the page into something decodable.
    fn image(&self, source: &str) -> ImageSource {
        let source = source.trim();
        if source.is_empty() {
            return ImageSource::Unavailable;
        }
        match self {
            Self::Directory(dir) => {
                ImageSource::File(dir.join(source.trim_start_matches('/').replace("//", "/")))
            }
            Self::Page(page) => match page.join(source) {
                Some(url)
                    if url.scheme == Scheme::Http
                        && litric_browser::http::is_loopback_host(&url.host) =>
                {
                    ImageSource::Url(url)
                }
                // A picture on another machine, or over TLS, needs the engine
                // this image does not have: the placeholder says so, and no
                // request leaves this machine.
                _ => ImageSource::Unavailable,
            },
        }
    }
}

/// Where one picture in a page comes from.
#[derive(Debug, Clone)]
enum ImageSource {
    File(PathBuf),
    Url(Url),
    Unavailable,
}

/// Whether the reader, not the engine, opens `url`.
///
/// Two kinds of address are its own: a file on this machine, and a page served
/// by a web server on this machine. Everything else is the engine's business,
/// and the engine says honestly when it is not there.
#[must_use]
pub fn serves(url: &str) -> bool {
    if url.starts_with("file://") {
        return true;
    }
    match Url::parse(url) {
        Ok(parsed) => {
            parsed.scheme == Scheme::Http && litric_browser::http::is_loopback_host(&parsed.host)
        }
        Err(_) => false,
    }
}

/// A page open in the window.
#[derive(Debug)]
pub struct Reader {
    url: String,
    title: String,
    base: Base,
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
    /// Whether the reader, not the engine, opens `url`.
    #[must_use]
    pub fn serves(url: &str) -> bool {
        serves(url)
    }

    /// Open the page at `url`.
    ///
    /// A `file://` page is read from disk. A page served by a web server on
    /// this machine is fetched over a real loopback socket, bounded by the HTTP
    /// reader. Anything else is the engine's business and is refused here.
    ///
    /// # Errors
    /// A sentence for the window when the page is missing, unreadable, over the
    /// document limit or not served correctly.
    pub fn open(url: &str) -> Result<Self, String> {
        let (bytes, base, fallback_title) = if let Some(rest) = url.strip_prefix("file://") {
            let rest = rest.split(['?', '#']).next().unwrap_or(rest);
            let decoded = percent_decode(rest);
            let path = PathBuf::from(decoded);
            let meta =
                std::fs::metadata(&path).map_err(|_| "the file could not be read".to_owned())?;
            if !meta.is_file() || meta.len() > html::MAX_DOCUMENT_BYTES as u64 {
                return Err("the page is too large to open".to_owned());
            }
            let bytes =
                std::fs::read(&path).map_err(|_| "the file could not be read".to_owned())?;
            let directory = path
                .parent()
                .map_or_else(|| PathBuf::from("/"), Path::to_path_buf);
            let fallback = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned());
            (bytes, Base::Directory(directory), fallback)
        } else {
            let parsed =
                Url::parse(url).map_err(|_| "this page is not on this machine".to_owned())?;
            if parsed.scheme != Scheme::Http
                || !litric_browser::http::is_loopback_host(&parsed.host)
            {
                return Err("this page is not on this machine".to_owned());
            }
            let response = litric_browser::http::get(&parsed)?;
            if !(200..300).contains(&response.status) {
                return Err(format!("the server answered {}", response.status));
            }
            let fallback = path_title(&parsed);
            (response.body, Base::Page(parsed), Some(fallback))
        };
        let document = html::parse(&bytes);
        let title = document
            .title
            .or(fallback_title)
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| "Page".to_owned());
        Ok(Self {
            url: url.to_owned(),
            title,
            base,
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
            match &laid.kind {
                LaidKind::Text { lines, .. } => out.extend(lines.iter().cloned()),
                LaidKind::Table { headers, rows, .. } => {
                    if !headers.is_empty() {
                        out.push(headers.join("  "));
                    }
                    for row in rows {
                        out.push(row.join("  "));
                    }
                }
                LaidKind::Image { alt, .. } => {
                    if !alt.is_empty() {
                        out.push(alt.clone());
                    }
                }
                LaidKind::Rule => {}
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
        let body = Style::new(tokens.text_px(TypeStep::Body), tokens.text());
        let heading1 = Style::bold(tokens.text_px(TypeStep::Title2), tokens.text());
        let heading2 = Style::bold(tokens.text_px(TypeStep::Title3), tokens.text());
        let heading3 = Style::bold(tokens.text_px(TypeStep::Headline), tokens.text());
        let mut y = MARGIN;
        let mut laid = Vec::with_capacity(self.source.len());
        for block in &self.source {
            match block {
                Block::Heading { text, level } => {
                    let style = match level {
                        1 => &heading1,
                        2 => &heading2,
                        _ => &heading3,
                    };
                    let lines = wrap_at(text, style, width, 40);
                    let line_h = style.size * 1.3;
                    let h = (lines.len() as f32).max(1.0) * line_h + 14.0;
                    laid.push(Laid {
                        y,
                        h,
                        kind: LaidKind::Text {
                            lines,
                            line_h,
                            level: (*level).clamp(1, 3),
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
                            level: 0,
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
                    let source = self.base.image(source);
                    laid.push(Laid {
                        y,
                        h,
                        kind: LaidKind::Image {
                            source,
                            alt: alt.clone(),
                            w: width,
                            h,
                        },
                    });
                    y += h;
                }
                Block::Table { headers, rows } => {
                    let cols = headers
                        .len()
                        .max(rows.iter().map(Vec::len).max().unwrap_or(0))
                        .max(1);
                    let col_w = (width / cols as f32).max(40.0);
                    let row_h = body.size * 1.9;
                    let lines = rows.len().saturating_add(usize::from(!headers.is_empty()));
                    let h = lines as f32 * row_h + 14.0;
                    laid.push(Laid {
                        y,
                        h,
                        kind: LaidKind::Table {
                            headers: headers.clone(),
                            rows: rows.clone(),
                            col_w,
                            row_h,
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

        let body = Style::new(tokens.text_px(TypeStep::Body), tokens.text());
        let heading1 = Style::bold(tokens.text_px(TypeStep::Title2), tokens.text());
        let heading2 = Style::bold(tokens.text_px(TypeStep::Title3), tokens.text());
        let heading3 = Style::bold(tokens.text_px(TypeStep::Headline), tokens.text());
        // A page reads on its own surface, not on the window's chrome: a sheet
        // a shade lighter than the panel, with its own edge.
        canvas.fill_round_rect(
            page,
            tokens.radius(Radius::Md),
            with_alpha(rgb(0xFFFFFF), 0.035),
        );
        canvas.stroke_round_rect(page, tokens.radius(Radius::Md), 1.0, tokens.border());
        let clip = canvas.push_clip(page);
        // Decode at most the frame's budget, and only pictures that are on
        // screen. This vector stays empty on every frame where nothing new
        // appears, so the steady path allocates nothing.
        let mut pending: Vec<(usize, ImageSource)> = Vec::new();
        for (index, laid) in self.laid.iter().enumerate() {
            let top = page.y + laid.y - self.scroll;
            if top + laid.h < page.y || top > page.bottom() {
                continue;
            }
            if let LaidKind::Image { source, .. } = &laid.kind {
                if !self.images.contains_key(&index) && pending.len() < MAX_DECODES_PER_FRAME {
                    pending.push((index, source.clone()));
                }
            }
        }
        for (index, source) in pending {
            self.decode(index, &source);
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
                    level,
                } => {
                    let style = match level {
                        0 => &body,
                        1 => &heading1,
                        2 => &heading2,
                        _ => &heading3,
                    };
                    for (i, line) in lines.iter().enumerate() {
                        let baseline = top + style.size * 0.95 + i as f32 * line_h;
                        font::draw(canvas, line, page.x + MARGIN, baseline, style);
                    }
                }
                LaidKind::Table {
                    headers,
                    rows,
                    col_w,
                    row_h,
                } => {
                    let head_style = Style::bold(body.size, tokens.text());
                    let mut row_top = top;
                    if !headers.is_empty() {
                        for (column, cell) in headers.iter().enumerate() {
                            let x = page.x + MARGIN + column as f32 * col_w;
                            font::draw(
                                canvas,
                                &crate::ui::elide(cell, &head_style, col_w - 12.0),
                                x,
                                row_top + body.size * 1.05,
                                &head_style,
                            );
                        }
                        row_top += row_h;
                        canvas.fill_round_rect(
                            Rect::new(page.x + MARGIN, row_top - 8.0, width, 1.0),
                            0.0,
                            tokens.border(),
                        );
                    }
                    for (index, row) in rows.iter().enumerate() {
                        // Zebra striping: a row of values is followed across,
                        // and a rule between every row is heavier than the
                        // data. Alternating tone is the cheapest way to keep
                        // the eye on one row.
                        if index % 2 == 1 {
                            canvas.fill_round_rect(
                                Rect::new(
                                    page.x + MARGIN,
                                    row_top - body.size * 0.95,
                                    width,
                                    *row_h,
                                ),
                                tokens.radius(Radius::Sm),
                                with_alpha(tokens.text(), 0.045),
                            );
                        }
                        for (column, cell) in row.iter().enumerate() {
                            let x = page.x + MARGIN + column as f32 * *col_w;
                            font::draw(
                                canvas,
                                &crate::ui::elide(cell, &body, *col_w - 12.0),
                                x,
                                row_top + body.size * 1.05,
                                &body,
                            );
                        }
                        row_top += *row_h;
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
    fn decode(&mut self, index: usize, source: &ImageSource) {
        let bytes = match source {
            ImageSource::File(path) => {
                let Ok(meta) = std::fs::metadata(path) else {
                    return;
                };
                if !meta.is_file() || meta.len() > MAX_IMAGE_BYTES {
                    return;
                }
                let Ok(bytes) = std::fs::read(path) else {
                    return;
                };
                bytes
            }
            ImageSource::Url(url) => {
                match litric_browser::http::get_with_limit(url, MAX_IMAGE_BYTES as usize) {
                    Ok(response)
                        if response.status == 200
                            && response
                                .content_type
                                .as_deref()
                                .is_some_and(|kind| kind.starts_with("image/jpeg")) =>
                    {
                        response.body
                    }
                    // Not a JPEG, not a success, or not reachable: the
                    // placeholder stays, and the page keeps its layout.
                    _ => return,
                }
            }
            ImageSource::Unavailable => return,
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

/// A page served without a `<title>` takes the name of its last path segment.
fn path_title(url: &Url) -> String {
    let path = url.path.trim_end_matches('/');
    let last = path
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(&url.host);
    percent_decode(last)
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
        assert_eq!(err, "this page is not on this machine");
        let err = Reader::open("http://example.com/").expect_err("remote http");
        assert_eq!(err, "this page is not on this machine");
    }

    #[test]
    fn the_reader_serves_this_machines_files_and_web_server_only() {
        assert!(Reader::serves("file:///tmp/page.html"));
        assert!(Reader::serves("http://127.0.0.1:8080/"));
        assert!(Reader::serves("http://localhost:3000/app"));
        assert!(Reader::serves("http://[::1]:9000/"));
        assert!(!Reader::serves("http://example.com/"));
        assert!(!Reader::serves("https://127.0.0.1/"));
        assert!(!Reader::serves("http://localhost.example.com/"));
        assert!(!Reader::serves("litric://about"));
    }

    #[test]
    fn a_page_reference_resolves_only_to_this_machines_server() {
        let page = Url::parse("http://127.0.0.1:8080/apps/reader/index.html").expect("a page");
        let base = Base::Page(page);
        match base.image("panel.jpg") {
            ImageSource::Url(url) => {
                assert_eq!(url.to_url(), "http://127.0.0.1:8080/apps/reader/panel.jpg");
            }
            other => panic!("expected a fetched picture, got {other:?}"),
        }
        match base.image("/art/two.jpg") {
            ImageSource::Url(url) => {
                assert_eq!(url.to_url(), "http://127.0.0.1:8080/art/two.jpg");
            }
            other => panic!("expected a root-relative picture, got {other:?}"),
        }
        assert!(matches!(
            base.image("https://example.com/x.jpg"),
            ImageSource::Unavailable
        ));
        let file = Base::Directory(PathBuf::from("/tmp/page"));
        assert!(matches!(file.image("a.jpg"), ImageSource::File(_)));
    }

    #[test]
    fn a_table_is_laid_out_as_columns_of_values() {
        let _ = crate::font::initialize();
        let dir = std::env::temp_dir().join(format!("litric-reader-table-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let page = dir.join("values.html");
        std::fs::write(
            &page,
            b"<h2>Values</h2><table><tr><th>Metric</th><th>Value</th></tr>\
              <tr><td>CPU</td><td>42%</td></tr></table>",
        )
        .expect("the page");
        let mut reader =
            Reader::open(&format!("file://{}", page.display())).expect("the page opens");
        let tokens = crate::ui::Tokens::new(
            litric_design::Theme::of(litric_design::ThemeKind::Dark),
            1.0,
        );
        let mut canvas = Canvas::new(420, 320, rgb(0));
        reader.draw(&mut canvas, &tokens, Rect::new(0.0, 0.0, 420.0, 320.0));
        let lines = reader.visible_text(320.0);
        assert!(
            lines.iter().any(|line| line == "Metric  Value"),
            "{lines:?}"
        );
        assert!(lines.iter().any(|line| line == "CPU  42%"), "{lines:?}");
        assert!(reader.content_height() > 40.0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_cache_bounds_are_pinned() {
        assert_eq!(MAX_CACHED_IMAGES, 4);
        assert_eq!(MAX_CACHED_PIXELS, 8_000_000);
        assert_eq!(MAX_IMAGE_BYTES, 16 * 1024 * 1024);
        assert_eq!(MAX_DECODES_PER_FRAME, 2);
    }
}
