//! A small, real HTML reader.
//!
//! This is not a browser engine and does not pretend to be one. It parses the
//! part of HTML that a page of writing and pictures actually uses — headings,
//! paragraphs, line breaks, images and horizontal rules — into blocks a window
//! can lay out and draw. A tag it does not know is skipped; the text inside it
//! is kept, because losing a sentence is worse than losing its styling.
//!
//! The parse is bounded in every direction: [`MAX_DOCUMENT_BYTES`] of input,
//! [`MAX_BLOCKS`] blocks and [`MAX_BLOCK_CHARS`] characters per block. A page
//! cannot turn opening a document into an allocation the size of the document's
//! ambitions.

/// The largest document this reader accepts.
pub const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;

/// The most blocks one document may produce.
pub const MAX_BLOCKS: usize = 600;

/// The most characters one text block may keep.
pub const MAX_BLOCK_CHARS: usize = 8_000;

/// The most table rows one document may keep.
pub const MAX_TABLE_ROWS: usize = 60;

/// The most table columns one document may keep.
pub const MAX_TABLE_COLS: usize = 6;

/// The most characters one table cell may keep.
pub const MAX_CELL_CHARS: usize = 120;

/// One block of a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A heading, with its HTML level (1 is the largest).
    Heading {
        /// 1, 2 or 3.
        level: u8,
        /// The heading's text.
        text: String,
    },
    /// A paragraph of text.
    Paragraph(String),
    /// An image, with the source exactly as the page wrote it.
    Image {
        /// The `src` attribute.
        source: String,
        /// The `alt` attribute, when there is one.
        alt: String,
        /// The declared width, when the page gave one.
        width: Option<u32>,
        /// The declared height, when the page gave one.
        height: Option<u32>,
    },
    /// A table of values: a header row and the rows under it.
    ///
    /// A table is kept as a table, not flattened into prose, because a page of
    /// metrics is read by its columns: "CPU 42%" run together with the next
    /// value is exactly the text a person cannot use.
    Table {
        /// The header cells, empty when the page had no header row.
        headers: Vec<String>,
        /// The body rows, each padded to the same number of columns.
        rows: Vec<Vec<String>>,
    },
    /// A horizontal rule.
    Rule,
}

/// A parsed document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    /// The `<title>`, when the page has one.
    pub title: Option<String>,
    /// The document's blocks, in reading order.
    pub blocks: Vec<Block>,
}

/// Parse a page. Input that is not UTF-8 is read lossily: a readable page with
/// one wrong byte is better than no page.
#[must_use]
pub fn parse(input: &[u8]) -> Document {
    let bytes = input
        .get(..input.len().min(MAX_DOCUMENT_BYTES))
        .unwrap_or(input);
    let text = String::from_utf8_lossy(bytes);
    let mut doc = Document {
        title: title_of(&text),
        blocks: Vec::new(),
    };
    let stripped = strip_hidden(&text);
    for segment in segments(&stripped) {
        if doc.blocks.len() >= MAX_BLOCKS {
            break;
        }
        match segment {
            Segment::Text(text) => parse_text(text, &mut doc),
            Segment::Table(raw) => {
                if let Some(block) = table(raw) {
                    doc.blocks.push(block);
                }
            }
        }
    }
    doc
}

/// One run of the document: prose, or a table kept out of the prose pass.
enum Segment<'a> {
    Text(&'a str),
    Table(&'a str),
}

/// Split a document into prose and `<table>` runs, so a table body's tags are
/// never read as paragraphs.
fn segments(text: &str) -> Vec<Segment<'_>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = open_tag(rest, "table") {
        let Some(after) = rest.get(at..) else {
            break;
        };
        let end = close_tag(after, "table").unwrap_or(after.len());
        out.push(Segment::Text(rest.get(..at).unwrap_or_default()));
        out.push(Segment::Table(after.get(..end).unwrap_or_default()));
        let next = after.get(end..).unwrap_or_default();
        if next.is_empty() {
            return out;
        }
        rest = next;
    }
    if !rest.is_empty() {
        out.push(Segment::Text(rest));
    }
    out
}

/// The byte offset of `<name` (opening tag, any case), when it is a tag and not
/// a longer name.
fn open_tag(text: &str, name: &str) -> Option<usize> {
    let lower = text.to_ascii_lowercase();
    let needle = format!("<{name}");
    let mut from = 0usize;
    while let Some(found) = lower.get(from..)?.find(&needle) {
        let at = from.saturating_add(found);
        let after = at.saturating_add(needle.len());
        let next = lower.get(after..).and_then(|s| s.chars().next());
        if next.is_some_and(|c| c == '>' || c == '/' || c.is_ascii_whitespace()) {
            return Some(at);
        }
        from = after;
    }
    None
}

/// The byte offset just past `</name>`, when the closing tag exists.
fn close_tag(text: &str, name: &str) -> Option<usize> {
    let lower = text.to_ascii_lowercase();
    let needle = format!("</{name}");
    let at = lower.find(&needle)?;
    let after = lower.get(at..)?.find('>')?;
    Some(at.saturating_add(after).saturating_add(1))
}

/// Parse one run of prose into blocks. This is the original pass: a tag it does
/// not know is skipped and its text kept.
fn parse_text(stripped: &str, doc: &mut Document) {
    let mut para = String::new();
    let mut heading: Option<u8> = None;
    let mut rest = stripped;

    while let Some(open) = rest.find('<') {
        push_text(&mut para, rest.get(..open).unwrap_or_default());
        let after = rest.get(open..).unwrap_or_default();
        let Some(close) = after.find('>') else {
            // An unterminated tag at the end is markup, not prose: drop it.
            rest = "";
            break;
        };
        let raw = after.get(1..close).unwrap_or_default();
        rest = after.get(close + 1..).unwrap_or_default();
        let name = tag_name(raw);
        let closing = raw.trim_start().starts_with('/');
        match name.as_str() {
            "title" | "head" | "html" | "body" | "meta" | "link" | "!doctype" => {}
            "img" => {
                flush(doc, &mut para, heading);
                if let Some(block) = image(raw) {
                    if doc.blocks.len() < MAX_BLOCKS {
                        doc.blocks.push(block);
                    }
                }
            }
            "hr" => {
                flush(doc, &mut para, heading);
                if doc.blocks.len() < MAX_BLOCKS {
                    doc.blocks.push(Block::Rule);
                }
            }
            "br" => flush(doc, &mut para, heading),
            "h1" | "h2" | "h3" => {
                flush(doc, &mut para, heading);
                if closing {
                    heading = None;
                } else if name.len() == 2 {
                    heading = name.as_bytes().last().map(|b| b.saturating_sub(b'0'));
                }
            }
            "p" | "div" | "li" | "tr" | "ul" | "ol" | "table" | "section" | "article"
            | "blockquote" => flush(doc, &mut para, heading),
            _ => {}
        }
    }
    push_text(&mut para, rest);
    flush(doc, &mut para, heading);
}

/// Parse one `<table>` run into a [`Block::Table`].
///
/// Only `<tr>`, `<td>` and `<th>` structure the result; any other tag inside a
/// cell is skipped and its text kept. A row of `<th>` becomes the header when
/// there is a body under it.
fn table(raw: &str) -> Option<Block> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut header_rows: Vec<bool> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut in_cell = false;
    let mut row_is_header = true;
    let mut saw_header_cell = false;
    let mut rest = raw;
    while let Some(open) = rest.find('<') {
        let text = rest.get(..open).unwrap_or_default();
        if in_cell {
            push_cell(&mut cell, text);
        }
        let after = rest.get(open..).unwrap_or_default();
        let Some(close) = after.find('>') else {
            break;
        };
        let tag_raw = after.get(1..close).unwrap_or_default();
        rest = after.get(close + 1..).unwrap_or_default();
        let name = tag_name(tag_raw);
        let closing = tag_raw.trim_start().starts_with('/');
        match name.as_str() {
            "td" | "th" => {
                if closing {
                    if in_cell {
                        row.push(collapse(&decode_entities(&cell)));
                        cell.clear();
                        in_cell = false;
                    }
                } else if !in_cell {
                    in_cell = true;
                    cell.clear();
                    if name == "th" {
                        saw_header_cell = true;
                    } else {
                        row_is_header = false;
                    }
                }
            }
            "tr" => {
                if closing && !row.is_empty() {
                    rows.push(std::mem::take(&mut row));
                    header_rows.push(row_is_header && saw_header_cell);
                    row_is_header = true;
                    saw_header_cell = false;
                }
            }
            "table" if closing => break,
            _ => {}
        }
    }
    if !row.is_empty() {
        rows.push(row);
        header_rows.push(row_is_header && saw_header_cell);
    }
    if rows.is_empty() {
        return None;
    }
    let mut headers = Vec::new();
    if header_rows.first().copied().unwrap_or(false) && rows.len() > 1 {
        headers = rows.remove(0);
    }
    rows.truncate(MAX_TABLE_ROWS);
    let cols = headers
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0))
        .min(MAX_TABLE_COLS);
    if cols == 0 {
        return None;
    }
    if !headers.is_empty() {
        headers.truncate(cols);
        while headers.len() < cols {
            headers.push(String::new());
        }
    }
    for row in &mut rows {
        row.truncate(cols);
        while row.len() < cols {
            row.push(String::new());
        }
    }
    Some(Block::Table { headers, rows })
}

/// Append cell text, bounded so one cell cannot grow without a limit.
fn push_cell(cell: &mut String, text: &str) {
    if cell.len() >= MAX_CELL_CHARS {
        return;
    }
    let room = MAX_CELL_CHARS.saturating_sub(cell.len());
    let mut cut = text.len().min(room);
    while cut > 0 && !text.is_char_boundary(cut) {
        cut = cut.saturating_sub(1);
    }
    cell.push_str(text.get(..cut).unwrap_or_default());
}

/// The `<title>`'s text, collapsed to one line.
fn title_of(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let open_end = lower
        .get(start..)?
        .find('>')?
        .saturating_add(start)
        .saturating_add(1);
    let end = lower
        .get(open_end..)?
        .find("</title")?
        .saturating_add(open_end);
    let raw = text.get(open_end..end)?;
    let title = collapse(&decode_entities(raw));
    (!title.is_empty()).then_some(title)
}

/// Remove script, style, comment and doctype contents, which are not prose.
fn strip_hidden(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        let lower = rest.to_ascii_lowercase();
        let next = ["<script", "<style", "<head", "<!--"]
            .into_iter()
            .filter_map(|tag| lower.find(tag).map(|at| (at, tag)))
            .min_by_key(|(at, _)| *at);
        let Some((at, tag)) = next else {
            out.push_str(rest);
            break;
        };
        out.push_str(rest.get(..at).unwrap_or_default());
        let closer = match tag {
            "<!--" => "-->",
            "<script" => "</script",
            "<head" => "</head",
            _ => "</style",
        };
        let after = rest.get(at..).unwrap_or_default();
        match after.to_ascii_lowercase().find(closer) {
            Some(end) => {
                let skip = after
                    .get(end..)
                    .and_then(|tail| tail.find('>'))
                    .map_or(after.len(), |close| {
                        end.saturating_add(close).saturating_add(1)
                    });
                rest = after.get(skip..).unwrap_or_default();
            }
            None => break,
        }
    }
    out
}

/// A tag's lowercased name, without attributes or the closing slash.
fn tag_name(raw: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('/').trim_start();
    let name: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '!')
        .collect();
    name.to_ascii_lowercase()
}

/// Build an image block from an `<img>` tag's attributes.
fn image(raw: &str) -> Option<Block> {
    let source = attribute(raw, "src")?;
    if source.is_empty() || source.starts_with("data:") {
        return None;
    }
    Some(Block::Image {
        source,
        alt: attribute(raw, "alt").unwrap_or_default(),
        width: attribute(raw, "width").and_then(|v| v.parse().ok()),
        height: attribute(raw, "height").and_then(|v| v.parse().ok()),
    })
}

/// The value of `name="..."`, `name='...'` or `name=value`.
fn attribute(raw: &str, name: &str) -> Option<String> {
    let lower = raw.to_ascii_lowercase();
    let mut from = 0usize;
    while let Some(found) = lower.get(from..)?.find(name) {
        let at = from.saturating_add(found);
        let before_ok = at == 0
            || lower
                .get(..at)
                .and_then(|s| s.chars().last())
                .is_some_and(|c| c.is_ascii_whitespace());
        let after = at.saturating_add(name.len());
        let rest = lower.get(after..)?;
        let rest = rest.trim_start();
        let skipped = lower
            .get(after..)
            .map_or(0, |s| s.len().saturating_sub(rest.len()));
        if before_ok && rest.starts_with('=') {
            let value_at = after.saturating_add(skipped).saturating_add(1);
            let tail = raw.get(value_at..)?.trim_start();
            let quote = tail.chars().next();
            let value = match quote {
                Some('"' | '\'') => {
                    let q = quote?;
                    let body = tail.get(1..)?;
                    body.get(..body.find(q)?)?.to_owned()
                }
                _ => tail
                    .split(|c: char| c.is_ascii_whitespace() || c == '>')
                    .next()
                    .unwrap_or_default()
                    .to_owned(),
            };
            return Some(decode_entities(&value));
        }
        from = after;
    }
    None
}

/// Finish the paragraph or heading being collected.
fn flush(doc: &mut Document, para: &mut String, heading: Option<u8>) {
    let text = collapse(&decode_entities(para));
    para.clear();
    if text.is_empty() || doc.blocks.len() >= MAX_BLOCKS {
        return;
    }
    let mut text = text;
    if text.len() > MAX_BLOCK_CHARS {
        text.truncate(MAX_BLOCK_CHARS);
    }
    match heading {
        Some(level) => doc.blocks.push(Block::Heading {
            level: level.clamp(1, 3),
            text,
        }),
        None => doc.blocks.push(Block::Paragraph(text)),
    }
}

/// Append visible text, bounded so a tag-free document still cannot grow the
/// paragraph without a limit.
fn push_text(para: &mut String, text: &str) {
    if para.len() >= MAX_BLOCK_CHARS {
        return;
    }
    let room = MAX_BLOCK_CHARS.saturating_sub(para.len());
    let mut cut = text.len().min(room);
    while cut > 0 && !text.is_char_boundary(cut) {
        cut = cut.saturating_sub(1);
    }
    para.push_str(text.get(..cut).unwrap_or_default());
}

/// Collapse runs of whitespace to single spaces and trim.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Decode the five named entities and numeric character references.
fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(rest.get(..at).unwrap_or_default());
        let tail = rest.get(at..).unwrap_or_default();
        let Some(end) = tail.find(';') else {
            out.push_str(tail);
            return out;
        };
        let entity = tail.get(1..end).unwrap_or_default();
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some(' '),
            _ => entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .or_else(|| entity.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                .and_then(char::from_u32),
        };
        if let Some(c) = decoded {
            out.push(c);
        } else {
            out.push_str(tail.get(..=end).unwrap_or_default());
        }
        rest = tail.get(end.saturating_add(1)..).unwrap_or_default();
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_paragraphs_images_and_rules_keep_their_order() {
        let doc = parse(
            b"<html><head><title>A Page</title></head><body>\
              <h1>Chapter One</h1><p>Some text.</p>\
              <img src=\"panel.jpg\" alt=\"A panel\" width=\"900\" height=\"600\">\
              <hr><h2>Chapter Two</h2><p>More text.</p></body></html>",
        );
        assert_eq!(doc.title.as_deref(), Some("A Page"));
        assert_eq!(
            doc.blocks,
            vec![
                Block::Heading {
                    level: 1,
                    text: "Chapter One".to_owned()
                },
                Block::Paragraph("Some text.".to_owned()),
                Block::Image {
                    source: "panel.jpg".to_owned(),
                    alt: "A panel".to_owned(),
                    width: Some(900),
                    height: Some(600)
                },
                Block::Rule,
                Block::Heading {
                    level: 2,
                    text: "Chapter Two".to_owned()
                },
                Block::Paragraph("More text.".to_owned()),
            ]
        );
    }

    #[test]
    fn hidden_blocks_contribute_no_text_and_entities_are_decoded() {
        let doc = parse(
            b"<style>p{color:red}</style><script>alert('no')</script>\
              <p>Tom &amp; Jerry&nbsp;say &quot;hi&quot; &#65;&#x42;</p>",
        );
        assert_eq!(
            doc.blocks,
            vec![Block::Paragraph("Tom & Jerry say \"hi\" AB".to_owned())]
        );
    }

    #[test]
    fn unknown_tags_keep_their_text_and_unterminated_tags_do_not_panic() {
        let doc = parse(b"<custom>kept text</custom><p>and <b>this</b>");
        assert_eq!(
            doc.blocks,
            vec![
                Block::Paragraph("kept text".to_owned()),
                Block::Paragraph("and this".to_owned())
            ]
        );
        let broken = parse(b"<p>dangling <img src=\"x.jpg\"");
        assert_eq!(broken.blocks, vec![Block::Paragraph("dangling".to_owned())]);
    }

    #[test]
    fn the_limits_are_pinned_and_enforced() {
        assert_eq!(MAX_DOCUMENT_BYTES, 4 * 1024 * 1024);
        assert_eq!(MAX_BLOCKS, 600);
        assert_eq!(MAX_BLOCK_CHARS, 8_000);
        assert_eq!(MAX_TABLE_ROWS, 60);
        assert_eq!(MAX_TABLE_COLS, 6);
        assert_eq!(MAX_CELL_CHARS, 120);
        let mut page = String::from("<p>");
        page.push_str(&"a".repeat(MAX_BLOCK_CHARS * 2));
        page.push_str("</p>");
        let doc = parse(page.as_bytes());
        match doc.blocks.first() {
            Some(Block::Paragraph(text)) => assert_eq!(text.chars().count(), MAX_BLOCK_CHARS),
            other => panic!("expected one paragraph, got {other:?}"),
        }
        let many = "<p>one</p>".repeat(MAX_BLOCKS + 50);
        assert_eq!(parse(many.as_bytes()).blocks.len(), MAX_BLOCKS);
    }

    #[test]
    fn a_table_keeps_its_columns_and_values() {
        let doc = parse(
            b"<h2>Values</h2><table><tr><th>Metric</th><th>Value</th></tr>\
              <tr><td>CPU</td><td>42%</td></tr>\
              <tr><td>Memory</td><td>1.2 GB</td></tr></table><p>after</p>",
        );
        assert_eq!(
            doc.blocks,
            vec![
                Block::Heading {
                    level: 2,
                    text: "Values".to_owned()
                },
                Block::Table {
                    headers: vec!["Metric".to_owned(), "Value".to_owned()],
                    rows: vec![
                        vec!["CPU".to_owned(), "42%".to_owned()],
                        vec!["Memory".to_owned(), "1.2 GB".to_owned()],
                    ]
                },
                Block::Paragraph("after".to_owned()),
            ]
        );
    }

    #[test]
    fn a_table_without_a_header_row_is_all_body_and_is_bounded() {
        let doc = parse(b"<table><tr><td>a</td><td>b</td></tr></table>");
        match doc.blocks.first() {
            Some(Block::Table { headers, rows }) => {
                assert!(headers.is_empty(), "{headers:?}");
                assert_eq!(rows, &vec![vec!["a".to_owned(), "b".to_owned()]]);
            }
            other => panic!("expected a table, got {other:?}"),
        }
        let many = "<tr><td>x</td></tr>".repeat(MAX_TABLE_ROWS + 10);
        let doc = parse(format!("<table>{many}</table>").as_bytes());
        match doc.blocks.first() {
            Some(Block::Table { rows, .. }) => assert_eq!(rows.len(), MAX_TABLE_ROWS),
            other => panic!("expected a bounded table, got {other:?}"),
        }
    }
}
