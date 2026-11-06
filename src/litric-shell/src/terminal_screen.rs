//! A small cell grid for full-screen terminal programs.
//!
//! This is deliberately not a complete terminal emulator. It models the parts
//! an alternate-screen program needs in order to be usable and to return the
//! normal scrollback unchanged: a cursor, erased cells, cursor addressing,
//! line wrapping, SGR colours/attributes, wide characters, combining marks and
//! an explicit scroll region. Sixel remains unsupported and is ignored rather
//! than silently rendered as text. Mouse reporting and bracketed paste are
//! handled by `TerminalApp`, because they need the shell's pointer geometry
//! and PTY writer; this grid only owns cells.
//!
//! A cell owns a display cluster, its column width and its SGR style. A wide
//! character therefore occupies its base cell plus a zero-width continuation;
//! a combining mark is appended to the base cluster and never advances the
//! cursor. Styles are copied into each cell, so a later SGR change cannot
//! recolour text that was already written.

/// A terminal colour, as requested by SGR.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Color {
    /// The terminal's configured foreground or background.
    #[default]
    Default,
    /// One of the 256 xterm palette entries. 0–15 are the ANSI colours,
    /// 16–231 the 6×6×6 cube and 232–255 the grey ramp.
    Indexed(u8),
    /// A 24-bit colour from `38;2;r;g;b` or `48;2;r;g;b`.
    Rgb(u8, u8, u8),
}

impl Color {
    /// Resolve to an opaque `0xAARRGGBB` pixel, using `default` for
    /// [`Color::Default`].
    #[must_use]
    pub fn to_argb(self, default: u32) -> u32 {
        match self {
            Self::Default => default,
            Self::Indexed(index) => indexed_argb(index).unwrap_or(default),
            Self::Rgb(r, g, b) => {
                0xFF00_0000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
            }
        }
    }
}

/// SGR attributes that affect a cell's appearance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Attrs {
    /// Bold text (`SGR 1`).
    pub bold: bool,
    /// Dim/faint text (`SGR 2`).
    pub dim: bool,
    /// Italic text (`SGR 3`). The renderer synthesises an oblique because the
    /// bundled faces do not ship an italic cut.
    pub italic: bool,
    /// Underlined text (`SGR 4`).
    pub underline: bool,
    /// Reverse video (`SGR 7`): foreground and background swap.
    pub reverse: bool,
}

/// The complete SGR state copied into each written cell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CellStyle {
    /// Foreground request.
    pub fg: Color,
    /// Background request.
    pub bg: Color,
    /// Bold, dim, italic, underline and reverse.
    pub attrs: Attrs,
}

impl CellStyle {
    /// Return to the terminal defaults. This is what `SGR 0` and an
    /// alternate-screen transition apply.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// One cell: a display cluster, the columns it occupies and its style.
///
/// `width == 0` is a continuation cell owned by the wide cell immediately to
/// its left. It is never drawn and is cleared whenever its owner is cleared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The base character plus any combining marks.
    pub text: String,
    /// 1 for an ordinary cell, 2 for a wide base cell, 0 for a continuation.
    pub width: u8,
    /// The style fixed when this cell was written.
    pub style: CellStyle,
}

impl Cell {
    fn blank() -> Self {
        Self {
            text: " ".to_owned(),
            width: 1,
            style: CellStyle::default(),
        }
    }

    fn continuation() -> Self {
        Self {
            text: String::new(),
            width: 0,
            style: CellStyle::default(),
        }
    }
}

fn blank_row(cols: usize) -> Vec<Cell> {
    vec![Cell::blank(); cols]
}

/// Width of one Unicode scalar in terminal columns.
///
/// This covers the ranges the terminal needs: CJK/full-width characters,
/// emoji, Thai/Lao combining marks, the common combining-mark blocks and the
/// zero-width joiners used by emoji. It is deliberately a small, dependency-free
/// approximation of Unicode's East Asian Width table, not a replacement for a
/// full grapheme-cluster implementation.
#[must_use]
pub fn display_width(ch: char) -> usize {
    let cp = u32::from(ch);
    if cp == 0 || is_zero_width(cp) {
        0
    } else if is_wide(cp) {
        2
    } else {
        1
    }
}

fn is_zero_width(cp: u32) -> bool {
    matches!(
        cp,
        0x0300..=0x036F
            | 0x0483..=0x0489
            | 0x0591..=0x05BD
            | 0x05BF
            | 0x05C1..=0x05C2
            | 0x05C4..=0x05C5
            | 0x05C7
            | 0x0610..=0x061A
            | 0x064B..=0x065F
            | 0x0670
            | 0x06D6..=0x06DC
            | 0x06DF..=0x06E4
            | 0x06E7..=0x06E8
            | 0x06EA..=0x06ED
            | 0x0711
            | 0x0730..=0x074A
            | 0x07A6..=0x07B0
            | 0x07EB..=0x07F3
            | 0x0816..=0x0819
            | 0x081B..=0x0823
            | 0x0825..=0x0827
            | 0x0829..=0x082D
            | 0x0859..=0x085B
            | 0x08D3..=0x08E1
            | 0x08E3..=0x0902
            | 0x093A
            | 0x093C
            | 0x0941..=0x0948
            | 0x094D
            | 0x0951..=0x0957
            | 0x0962..=0x0963
            | 0x0981
            | 0x09BC
            | 0x09C1..=0x09C4
            | 0x09CD
            | 0x09E2..=0x09E3
            | 0x0A01..=0x0A02
            | 0x0A3C
            | 0x0A41..=0x0A42
            | 0x0A47..=0x0A48
            | 0x0A4B..=0x0A4D
            | 0x0A51
            | 0x0A70..=0x0A71
            | 0x0A75
            | 0x0A81..=0x0A82
            | 0x0ABC
            | 0x0AC1..=0x0AC5
            | 0x0AC7..=0x0AC8
            | 0x0ACD
            | 0x0AE2..=0x0AE3
            | 0x0B01
            | 0x0B3C
            | 0x0B3F
            | 0x0B41..=0x0B44
            | 0x0B4D
            | 0x0B56
            | 0x0B62..=0x0B63
            | 0x0B82
            | 0x0BC0
            | 0x0BCD
            | 0x0C00
            | 0x0C3E..=0x0C40
            | 0x0C46..=0x0C48
            | 0x0C4A..=0x0C4D
            | 0x0C55..=0x0C56
            | 0x0C62..=0x0C63
            | 0x0C81
            | 0x0CBC
            | 0x0CBF
            | 0x0CC6
            | 0x0CCC..=0x0CCD
            | 0x0CE2..=0x0CE3
            | 0x0D00..=0x0D01
            | 0x0D41..=0x0D44
            | 0x0D4D
            | 0x0D62..=0x0D63
            | 0x0DCA
            | 0x0DD2..=0x0DD4
            | 0x0DD6
            | 0x0E31
            | 0x0E34..=0x0E3A
            | 0x0E47..=0x0E4E
            | 0x0EB1
            | 0x0EB4..=0x0EBC
            | 0x0EC8..=0x0ECD
            | 0x0F18..=0x0F19
            | 0x0F35
            | 0x0F37
            | 0x0F39
            | 0x0F71..=0x0F7E
            | 0x0F80..=0x0F84
            | 0x0F86..=0x0F87
            | 0x0F8D..=0x0F97
            | 0x0F99..=0x0FBC
            | 0x0FC6
            | 0x102D..=0x1030
            | 0x1032..=0x1037
            | 0x1039..=0x103A
            | 0x103D..=0x103E
            | 0x1058..=0x1059
            | 0x105E..=0x1060
            | 0x1071..=0x1074
            | 0x1082
            | 0x1085..=0x1086
            | 0x108D
            | 0x109D
            | 0x135D..=0x135F
            | 0x1712..=0x1714
            | 0x1732..=0x1734
            | 0x1752..=0x1753
            | 0x1772..=0x1773
            | 0x17B4..=0x17B5
            | 0x17B7..=0x17BD
            | 0x17C6
            | 0x17C9..=0x17D3
            | 0x17DD
            | 0x180B..=0x180D
            | 0x1885..=0x1886
            | 0x18A9
            | 0x1920..=0x1922
            | 0x1927..=0x1928
            | 0x1932
            | 0x1939..=0x193B
            | 0x1A17..=0x1A18
            | 0x1A1B
            | 0x1A56
            | 0x1A58..=0x1A5E
            | 0x1A60
            | 0x1A62
            | 0x1A65..=0x1A6C
            | 0x1A73..=0x1A7C
            | 0x1A7F
            | 0x1AB0..=0x1AFF
            | 0x1B00..=0x1B03
            | 0x1B34
            | 0x1B36..=0x1B3A
            | 0x1B3C
            | 0x1B42
            | 0x1B6B..=0x1B73
            | 0x1B80..=0x1B81
            | 0x1BA2..=0x1BA5
            | 0x1BA8..=0x1BA9
            | 0x1BAB..=0x1BAD
            | 0x1BE6
            | 0x1BE8..=0x1BE9
            | 0x1BED
            | 0x1BEF..=0x1BF1
            | 0x1C2C..=0x1C33
            | 0x1C36..=0x1C37
            | 0x1CD0..=0x1CD2
            | 0x1CD4..=0x1CE0
            | 0x1CE2..=0x1CE8
            | 0x1CED
            | 0x1CF4
            | 0x1CF8..=0x1CF9
            | 0x1DC0..=0x1DFF
            | 0x200B..=0x200F
            | 0x202A..=0x202E
            | 0x2060..=0x206F
            | 0x20D0..=0x20FF
            | 0x2CEF..=0x2CF1
            | 0x2D7F
            | 0x2DE0..=0x2DFF
            | 0x302A..=0x302D
            | 0x3099..=0x309A
            | 0xA66F
            | 0xA674..=0xA67D
            | 0xA69E..=0xA69F
            | 0xA6F0..=0xA6F1
            | 0xA802
            | 0xA806
            | 0xA80B
            | 0xA825..=0xA826
            | 0xA8C4
            | 0xA8E0..=0xA8F1
            | 0xA926..=0xA92D
            | 0xA947..=0xA951
            | 0xA980..=0xA982
            | 0xA9B3
            | 0xA9B6..=0xA9B9
            | 0xA9BC
            | 0xA9E5
            | 0xAA29..=0xAA2E
            | 0xAA31..=0xAA32
            | 0xAA35..=0xAA36
            | 0xAA43
            | 0xAA4C
            | 0xAA7C
            | 0xAAB0
            | 0xAAB2..=0xAAB4
            | 0xAAB7..=0xAAB8
            | 0xAABE..=0xAABF
            | 0xAAC1
            | 0xAAEC..=0xAAED
            | 0xAAF6
            | 0xABE5
            | 0xABE8
            | 0xABED
            | 0xFB1E
            | 0xFE00..=0xFE0F
            | 0xFE20..=0xFE2F
            | 0xFEFF
            | 0x101FD
            | 0x102E0
            | 0x10376..=0x1037A
            | 0x10A01..=0x10A03
            | 0x10A05..=0x10A06
            | 0x10A0C..=0x10A0F
            | 0x10A38..=0x10A3A
            | 0x10A3F
            | 0x10AE5..=0x10AE6
            | 0x11001
            | 0x11038..=0x11046
            | 0x1107F..=0x11081
            | 0x110B3..=0x110B6
            | 0x110B9..=0x110BA
            | 0x11100..=0x11102
            | 0x11127..=0x1112B
            | 0x1112D..=0x11134
            | 0x11173
            | 0x11180..=0x11181
            | 0x111B6..=0x111BE
            | 0x111CA..=0x111CC
            | 0x1122F..=0x11231
            | 0x11234
            | 0x11236..=0x11237
            | 0x1123E
            | 0x112DF
            | 0x112E3..=0x112EA
            | 0x11300..=0x11301
            | 0x1133C
            | 0x11340
            | 0x11366..=0x1136C
            | 0x11370..=0x11374
            | 0x11438..=0x1143F
            | 0x11442..=0x11444
            | 0x11446
            | 0x1145E
            | 0x114B3..=0x114B8
            | 0x114BA
            | 0x114BF..=0x114C0
            | 0x114C2..=0x114C3
            | 0x115B2..=0x115B5
            | 0x115BC..=0x115BD
            | 0x115BF..=0x115C0
            | 0x115DC..=0x115DD
            | 0x11633..=0x1163A
            | 0x1163D
            | 0x1163F..=0x11640
            | 0x116AB
            | 0x116AD
            | 0x116B0..=0x116B5
            | 0x116B7
            | 0x1171D..=0x1171F
            | 0x11722..=0x11725
            | 0x11727..=0x1172B
            | 0x11A01..=0x11A06
            | 0x11A09..=0x11A0A
            | 0x11A33..=0x11A38
            | 0x11A3B..=0x11A3E
            | 0x11A47
            | 0x11A51..=0x11A56
            | 0x11A59..=0x11A5B
            | 0x11A8A..=0x11A96
            | 0x11A98..=0x11A99
            | 0x11C30..=0x11C36
            | 0x11C38..=0x11C3D
            | 0x11C3F
            | 0x11C92..=0x11CA7
            | 0x11CAA..=0x11CB0
            | 0x11CB2..=0x11CB3
            | 0x11CB5..=0x11CB6
            | 0x11D31..=0x11D36
            | 0x11D3A
            | 0x11D3C..=0x11D3D
            | 0x11D3F..=0x11D45
            | 0x11D47
            | 0x16AF0..=0x16AF4
            | 0x16B30..=0x16B36
            | 0x16F8F..=0x16F92
            | 0x1BC9D..=0x1BC9E
            | 0x1D167..=0x1D169
            | 0x1D17B..=0x1D182
            | 0x1D185..=0x1D18B
            | 0x1D1AA..=0x1D1AD
            | 0x1D242..=0x1D244
            | 0x1DA00..=0x1DA36
            | 0x1DA3B..=0x1DA6C
            | 0x1DA75
            | 0x1DA84
            | 0x1DA9B..=0x1DA9F
            | 0x1DAA1..=0x1DAAF
            | 0x1E000..=0x1E006
            | 0x1E008..=0x1E018
            | 0x1E01B..=0x1E021
            | 0x1E023..=0x1E024
            | 0x1E026..=0x1E02A
            | 0x1E8D0..=0x1E8D6
            | 0x1E944..=0x1E94A
            | 0xE0020..=0xE007F
            | 0xE0100..=0xE01EF
            | 0x1F3FB..=0x1F3FF
    )
}

fn is_wide(cp: u32) -> bool {
    matches!(
        cp,
        0x1100..=0x115F
            | 0x2329..=0x232A
            | 0x2E80..=0x303E
            | 0x3041..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xA960..=0xA97F
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE19
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F1E6..=0x1F1FF
            | 0x1F300..=0x1FAFF
            | 0x20000..=0x3FFFD
    )
}

fn indexed_argb(index: u8) -> Option<u32> {
    const ANSI: [u32; 16] = [
        0x000000, 0x800000, 0x008000, 0x808000, 0x000080, 0x800080, 0x008080, 0xC0C0C0, 0x808080,
        0xFF0000, 0x00FF00, 0xFFFF00, 0x0000FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF,
    ];
    if let Some(color) = ANSI.get(usize::from(index)) {
        return Some(0xFF00_0000 | color);
    }
    if index >= 232 {
        let level = 8u32.saturating_add(u32::from(index.saturating_sub(232)).saturating_mul(10));
        return Some(0xFF00_0000 | (level << 16) | (level << 8) | level);
    }
    let n = u32::from(index).saturating_sub(16);
    let r = n.div_euclid(36);
    let g = n.rem_euclid(36).div_euclid(6);
    let b = n.rem_euclid(6);
    let component = |value: u32| -> u32 {
        match value {
            0 => 0,
            1 => 95,
            2 => 135,
            3 => 175,
            4 => 215,
            _ => 255,
        }
    };
    Some(0xFF00_0000 | (component(r) << 16) | (component(g) << 8) | component(b))
}

/// A rectangular screen of styled cells with a cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    cols: usize,
    rows: usize,
    cells: Vec<Vec<Cell>>,
    /// Zero-based row of the cursor.
    pub cursor_row: usize,
    /// Zero-based column of the cursor.
    pub cursor_col: usize,
    /// Whether the program asked for the cursor to be shown.
    pub cursor_visible: bool,
    /// The preceding cell ended with a zero-width joiner, so the next printable
    /// scalar belongs to the same cluster.
    join_next: bool,
    /// First row of the current scroll region, zero-based and inclusive.
    scroll_top: usize,
    /// Last row of the current scroll region, zero-based and inclusive.
    scroll_bottom: usize,
}

impl Screen {
    /// A blank screen with the cursor at the top left.
    #[must_use]
    pub fn new(cols: usize, rows: usize) -> Self {
        let cols = cols.clamp(1, 500);
        let rows = rows.clamp(1, 300);
        Self {
            cols,
            rows,
            cells: vec![blank_row(cols); rows],
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            join_next: false,
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
        }
    }

    /// Width in cells.
    #[must_use]
    pub const fn cols(&self) -> usize {
        self.cols
    }

    /// Height in cells.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// Change the grid size, keeping the top-left content.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let cols = cols.clamp(1, 500);
        let rows = rows.clamp(1, 300);
        if cols == self.cols && rows == self.rows {
            return;
        }
        let mut cells = vec![blank_row(cols); rows];
        for (row, source) in self.cells.iter().take(rows).enumerate() {
            let Some(target) = cells.get_mut(row) else {
                continue;
            };
            for (col, cell) in source.iter().take(cols).enumerate() {
                if let Some(slot) = target.get_mut(col) {
                    *slot = cell.clone();
                }
            }
        }
        for row in &mut cells {
            repair_row(row);
        }
        self.cols = cols;
        self.rows = rows;
        self.cells = cells;
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.scroll_top = self.scroll_top.min(rows.saturating_sub(1));
        self.scroll_bottom = self.scroll_bottom.min(rows.saturating_sub(1));
        if self.scroll_top >= self.scroll_bottom {
            self.scroll_top = 0;
            self.scroll_bottom = rows.saturating_sub(1);
        }
        self.join_next = false;
    }

    /// Write one printable character at the cursor and advance.
    pub fn put(&mut self, ch: char) {
        self.put_styled(ch, CellStyle::default());
    }

    /// Write one printable character with a style at the cursor and advance.
    ///
    /// Combining marks attach to the preceding base cell and do not advance.
    /// A wide character occupies two columns; if only one column remains it
    /// wraps before writing rather than dropping half a glyph at the edge.
    pub fn put_styled(&mut self, ch: char, style: CellStyle) {
        let width = display_width(ch);
        if width == 0 {
            self.combine(ch);
            return;
        }
        if self.join_next {
            self.join_next = false;
            if self.append_to_previous(ch) {
                return;
            }
        }
        let width = width.min(self.cols).max(1);
        if self.cursor_col.saturating_add(width) > self.cols {
            self.cursor_col = 0;
            self.line_feed();
        }
        let start = self.cursor_col;
        if let Some(row) = self.cells.get_mut(self.cursor_row) {
            clear_block(row, start, start.saturating_add(width));
            if let Some(cell) = row.get_mut(start) {
                cell.text = ch.to_string();
                cell.width = u8::try_from(width).unwrap_or(1);
                cell.style = style;
            }
            if width == 2 {
                if let Some(cell) = row.get_mut(start.saturating_add(1)) {
                    *cell = Cell::continuation();
                }
            }
        }
        self.cursor_col = start.saturating_add(width);
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.line_feed();
        }
    }

    fn combine(&mut self, ch: char) {
        if ch == '\u{200D}' {
            if self.append_to_previous(ch) {
                self.join_next = true;
            }
            return;
        }
        let _ = self.append_to_previous(ch);
    }

    fn append_to_previous(&mut self, ch: char) -> bool {
        if self.cursor_col == 0 {
            return false;
        }
        let row = self.cells.get_mut(self.cursor_row);
        let Some(row) = row else {
            return false;
        };
        let mut index = self.cursor_col.saturating_sub(1);
        if row.get(index).is_some_and(|cell| cell.width == 0) {
            index = index.saturating_sub(1);
        }
        let Some(cell) = row.get_mut(index) else {
            return false;
        };
        if cell.width == 0 {
            return false;
        }
        cell.text.push(ch);
        true
    }

    /// Move down one row, scrolling the active region at its bottom.
    pub fn line_feed(&mut self) {
        if self.cursor_row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor_row.saturating_add(1) < self.rows {
            self.cursor_row = self.cursor_row.saturating_add(1);
        } else {
            // The cursor is below the region at the physical bottom. There is
            // no in-region scroll to perform, so preserve the screen invariant
            // by scrolling the whole page.
            self.cells.remove(0);
            self.cells.push(blank_row(self.cols));
        }
        self.join_next = false;
    }

    /// Move up one row, scrolling the active region at its top.
    pub fn reverse_index(&mut self) {
        if self.cursor_row == self.scroll_top {
            self.scroll_down(1);
        } else if self.cursor_row > 0 {
            self.cursor_row = self.cursor_row.saturating_sub(1);
        }
        self.join_next = false;
    }

    /// The active scroll region as zero-based, inclusive `(top, bottom)`.
    #[must_use]
    pub const fn scroll_region(&self) -> (usize, usize) {
        (self.scroll_top, self.scroll_bottom)
    }

    /// Set the scroll region (`CSI r`, DECSTBM). An invalid or one-line region
    /// is ignored, matching the terminal's "leave the old margins alone" rule.
    /// A valid region homes the cursor, as DECSTBM does.
    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        let max = self.rows.saturating_sub(1);
        let top = top.min(max);
        let bottom = bottom.min(max);
        if top < bottom {
            self.scroll_top = top;
            self.scroll_bottom = bottom;
            self.cursor_row = 0;
            self.cursor_col = 0;
            self.join_next = false;
        }
    }

    /// Scroll the active region up by `lines`, blanking the new bottom rows.
    pub fn scroll_up(&mut self, lines: usize) {
        let height = self
            .scroll_bottom
            .saturating_sub(self.scroll_top)
            .saturating_add(1);
        let count = lines.min(height);
        for _ in 0..count {
            self.cells.remove(self.scroll_top);
            self.cells.insert(self.scroll_bottom, blank_row(self.cols));
        }
    }

    /// Scroll the active region down by `lines`, blanking the new top rows.
    pub fn scroll_down(&mut self, lines: usize) {
        let height = self
            .scroll_bottom
            .saturating_sub(self.scroll_top)
            .saturating_add(1);
        let count = lines.min(height);
        for _ in 0..count {
            self.cells.remove(self.scroll_bottom);
            self.cells.insert(self.scroll_top, blank_row(self.cols));
        }
    }

    /// Insert blank lines at the cursor inside the active region (`CSI L`).
    pub fn insert_lines(&mut self, lines: usize) {
        if self.cursor_row < self.scroll_top || self.cursor_row > self.scroll_bottom {
            return;
        }
        let available = self
            .scroll_bottom
            .saturating_sub(self.cursor_row)
            .saturating_add(1);
        let count = lines.min(available);
        for _ in 0..count {
            let _ = self.cells.remove(self.scroll_bottom);
            self.cells.insert(self.cursor_row, blank_row(self.cols));
        }
        self.cursor_col = 0;
        self.join_next = false;
    }

    /// Delete lines at the cursor inside the active region (`CSI M`).
    pub fn delete_lines(&mut self, lines: usize) {
        if self.cursor_row < self.scroll_top || self.cursor_row > self.scroll_bottom {
            return;
        }
        let available = self
            .scroll_bottom
            .saturating_sub(self.cursor_row)
            .saturating_add(1);
        let count = lines.min(available);
        for _ in 0..count {
            let _ = self.cells.remove(self.cursor_row);
            self.cells.insert(self.scroll_bottom, blank_row(self.cols));
        }
        self.cursor_col = 0;
        self.join_next = false;
    }

    /// Carriage return.
    pub fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    /// Backspace without erasing. A wide base is the target, never its
    /// continuation, so a following write replaces the whole glyph.
    pub fn backspace(&mut self) {
        let mut col = self.cursor_col.saturating_sub(1);
        if self
            .cells
            .get(self.cursor_row)
            .and_then(|row| row.get(col))
            .is_some_and(|cell| cell.width == 0)
        {
            col = col.saturating_sub(1);
        }
        self.cursor_col = col;
        self.join_next = false;
    }

    /// Tab to the next multiple of eight.
    pub fn tab(&mut self) {
        let next = self.cursor_col.saturating_add(8) & !7;
        self.cursor_col = next.min(self.cols.saturating_sub(1));
        self.join_next = false;
    }

    /// Move the cursor to a zero-based cell, clamped to the grid.
    pub fn move_to(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows.saturating_sub(1));
        let col = col.min(self.cols.saturating_sub(1));
        self.cursor_col = if self
            .cells
            .get(self.cursor_row)
            .and_then(|line| line.get(col))
            .is_some_and(|cell| cell.width == 0)
        {
            col.saturating_sub(1)
        } else {
            col
        };
        self.join_next = false;
    }

    /// Erase the whole screen and home the cursor.
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            row.fill(Cell::blank());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.join_next = false;
    }

    /// Erase from the cursor to the end of the current line.
    pub fn erase_to_end_of_line(&mut self) {
        if let Some(row) = self.cells.get_mut(self.cursor_row) {
            clear_block(row, self.cursor_col, self.cols);
        }
        self.join_next = false;
    }

    /// Erase from the start of the current line to the cursor, inclusive.
    pub fn erase_to_start_of_line(&mut self) {
        if let Some(row) = self.cells.get_mut(self.cursor_row) {
            clear_block(row, 0, self.cursor_col.saturating_add(1));
        }
        self.join_next = false;
    }

    /// The visible rows with trailing blanks removed.
    #[must_use]
    pub fn text_lines(&self) -> Vec<String> {
        self.cells
            .iter()
            .map(|row| {
                let line: String = row
                    .iter()
                    .filter(|cell| cell.width != 0)
                    .map(|cell| cell.text.as_str())
                    .collect();
                line.trim_end().to_owned()
            })
            .collect()
    }

    /// One cell, for drawing and diagnostics.
    #[must_use]
    pub fn cell_at(&self, row: usize, col: usize) -> Option<&Cell> {
        self.cells.get(row).and_then(|line| line.get(col))
    }

    /// One cell's base character, for tests and diagnostics.
    #[must_use]
    pub fn cell(&self, row: usize, col: usize) -> Option<char> {
        self.cell_at(row, col)
            .and_then(|cell| cell.text.chars().next())
    }
}

fn repair_row(row: &mut [Cell]) {
    let len = row.len();
    for index in 0..len {
        let width = row.get(index).map_or(1, |cell| cell.width);
        if width == 0 {
            let owned = index
                .checked_sub(1)
                .and_then(|before| row.get(before))
                .is_some_and(|cell| cell.width == 2);
            if !owned {
                if let Some(cell) = row.get_mut(index) {
                    *cell = Cell::blank();
                }
            }
        } else if width == 2 && index.saturating_add(1) >= len {
            if let Some(cell) = row.get_mut(index) {
                *cell = Cell::blank();
            }
        }
    }
}

fn clear_block(row: &mut [Cell], start: usize, end: usize) {
    let len = row.len();
    let start = start.min(len);
    let end = end.min(len);
    for index in start..end {
        let width = row.get(index).map_or(1, |cell| cell.width);
        let base = if width == 0 {
            index.saturating_sub(1)
        } else {
            index
        };
        let span = if row.get(base).is_some_and(|cell| cell.width == 2) {
            2
        } else {
            1
        };
        let stop = base.saturating_add(span).min(len);
        for at in base..stop {
            if let Some(cell) = row.get_mut(at) {
                *cell = Cell::blank();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{display_width, CellStyle, Color, Screen};

    fn styled(fg: Color) -> CellStyle {
        CellStyle {
            fg,
            ..CellStyle::default()
        }
    }

    #[test]
    fn sgr_styles_are_fixed_per_cell_and_do_not_leak_to_the_next_cell() {
        let mut screen = Screen::new(8, 2);
        let mut red = styled(Color::Indexed(9));
        red.attrs.bold = true;
        red.attrs.underline = true;
        screen.put_styled('R', red);
        screen.put('N');
        assert_eq!(screen.cell_at(0, 0).map(|cell| cell.style), Some(red));
        assert_eq!(
            screen.cell_at(0, 1).map(|cell| cell.style),
            Some(CellStyle::default())
        );
        assert_eq!(screen.text_lines(), ["RN", ""]);
    }

    #[test]
    fn wide_characters_reserve_two_columns_and_overwrite_cleanly() {
        let mut screen = Screen::new(6, 2);
        screen.put_styled('界', styled(Color::Indexed(2)));
        assert_eq!(screen.cursor_col, 2);
        assert_eq!(screen.cell_at(0, 0).map(|cell| cell.width), Some(2));
        assert_eq!(screen.cell_at(0, 1).map(|cell| cell.width), Some(0));
        screen.put('A');
        assert_eq!(
            screen.cell_at(0, 2).map(|cell| cell.text.as_str()),
            Some("A")
        );

        screen.move_to(0, 0);
        screen.put('X');
        assert_eq!(
            screen.cell_at(0, 0).map(|cell| cell.text.as_str()),
            Some("X")
        );
        assert_eq!(screen.cell_at(0, 1).map(|cell| cell.width), Some(1));
    }

    #[test]
    fn a_wide_character_that_does_not_fit_wraps_instead_of_being_split() {
        let mut screen = Screen::new(3, 2);
        screen.put('a');
        screen.put('b');
        screen.put('c');
        screen.put('界');
        assert_eq!(screen.text_lines(), ["abc", "界"]);
        assert_eq!(screen.cursor_row, 1);
        assert_eq!(screen.cell_at(1, 0).map(|cell| cell.width), Some(2));
    }

    #[test]
    fn combining_marks_attach_to_the_base_and_do_not_advance() {
        let mut screen = Screen::new(8, 1);
        screen.put('ก');
        screen.put('\u{0E31}');
        assert_eq!(screen.cell(0, 0), Some('ก'));
        assert_eq!(
            screen.cell_at(0, 0).map(|cell| cell.text.as_str()),
            Some("กั")
        );
        assert_eq!(screen.cursor_col, 1);
        screen.put('า');
        assert_eq!(
            screen.cell_at(0, 1).map(|cell| cell.text.as_str()),
            Some("า")
        );
    }

    #[test]
    fn a_combining_mark_after_a_wide_base_stays_in_that_cluster() {
        let mut screen = Screen::new(8, 1);
        screen.put('界');
        screen.put('\u{0301}');
        assert_eq!(
            screen.cell_at(0, 0).map(|cell| cell.text.as_str()),
            Some("界\u{0301}")
        );
        assert_eq!(screen.cursor_col, 2);
    }

    #[test]
    fn backspace_and_erase_never_leave_half_a_wide_character() {
        let mut screen = Screen::new(8, 2);
        screen.put('界');
        screen.backspace();
        assert_eq!(screen.cursor_col, 0);
        screen.put('X');
        assert_eq!(
            screen.cell_at(0, 0).map(|cell| cell.text.as_str()),
            Some("X")
        );
        assert_eq!(screen.cell_at(0, 1).map(|cell| cell.width), Some(1));

        screen.put('界');
        screen.move_to(0, 2);
        assert_eq!(screen.cursor_col, 1, "cursor must move to the wide base");
        screen.erase_to_end_of_line();
        assert_eq!(screen.text_lines(), ["X", ""]);
        assert!(screen.cell_at(0, 1).is_some_and(|cell| cell.width == 1));
    }

    #[test]
    fn display_width_covers_cjk_thai_and_emoji() {
        assert_eq!(display_width('a'), 1);
        assert_eq!(display_width('界'), 2);
        assert_eq!(display_width('ก'), 1);
        assert_eq!(display_width('\u{0E31}'), 0);
        assert_eq!(display_width('\u{1F600}'), 2);
        assert_eq!(display_width('\u{200D}'), 0);
    }

    #[test]
    fn resizing_repairs_a_wide_character_cut_by_the_edge() {
        let mut screen = Screen::new(3, 1);
        screen.put('界');
        screen.resize(1, 1);
        assert_eq!(screen.text_lines(), [""]);
        assert_eq!(screen.cell_at(0, 0).map(|cell| cell.width), Some(1));
    }
    #[test]
    fn colour_requests_resolve_to_ansi_256_and_truecolor_pixels() {
        assert_eq!(Color::Default.to_argb(0xFF11_2233), 0xFF11_2233);
        assert_eq!(Color::Indexed(9).to_argb(0), 0xFFFF_0000);
        assert_eq!(Color::Indexed(196).to_argb(0), 0xFFFF_0000);
        assert_eq!(Color::Indexed(232).to_argb(0), 0xFF08_0808);
        assert_eq!(Color::Rgb(1, 2, 3).to_argb(0), 0xFF01_0203);
    }

    #[test]
    fn a_style_is_copied_so_the_next_line_starts_with_the_default() {
        let mut screen = Screen::new(4, 2);
        let red = styled(Color::Indexed(1));
        screen.put_styled('A', red);
        screen.line_feed();
        screen.carriage_return();
        screen.put('B');
        assert_eq!(screen.cell_at(0, 0).map(|cell| cell.style), Some(red));
        assert_eq!(
            screen.cell_at(1, 0).map(|cell| cell.style),
            Some(CellStyle::default())
        );
    }
    fn labelled_screen() -> Screen {
        let mut screen = Screen::new(8, 5);
        for (row, label) in ["A", "B", "C", "D", "E"].iter().enumerate() {
            screen.move_to(row, 0);
            screen.put(label.chars().next().unwrap_or('?'));
        }
        screen
    }

    #[test]
    fn a_scroll_region_scrolls_only_its_rows() {
        let mut screen = labelled_screen();
        screen.set_scroll_region(1, 3);
        screen.move_to(3, 0);
        screen.line_feed();
        assert_eq!(screen.text_lines(), ["A", "C", "D", "", "E"]);
        assert_eq!(screen.scroll_region(), (1, 3));
    }

    #[test]
    fn reverse_index_scrolls_the_region_down_without_touching_outside_rows() {
        let mut screen = labelled_screen();
        screen.set_scroll_region(1, 3);
        screen.move_to(1, 0);
        screen.reverse_index();
        assert_eq!(screen.text_lines(), ["A", "", "B", "C", "E"]);
    }

    #[test]
    fn insert_and_delete_lines_stay_inside_the_region() {
        let mut screen = labelled_screen();
        screen.set_scroll_region(1, 3);
        screen.move_to(2, 0);
        screen.insert_lines(1);
        assert_eq!(screen.text_lines(), ["A", "B", "", "C", "E"]);
        screen.move_to(2, 0);
        screen.delete_lines(1);
        assert_eq!(screen.text_lines(), ["A", "B", "C", "", "E"]);
    }

    #[test]
    fn explicit_scroll_up_and_down_are_bounded_by_the_region() {
        let mut screen = labelled_screen();
        screen.set_scroll_region(1, 3);
        screen.scroll_up(9);
        assert_eq!(screen.text_lines(), ["A", "", "", "", "E"]);
        screen.scroll_down(9);
        assert_eq!(screen.text_lines(), ["A", "", "", "", "E"]);
    }

    #[test]
    fn resizing_clamps_the_scroll_region_and_repairs_it() {
        let mut screen = labelled_screen();
        screen.set_scroll_region(2, 4);
        screen.resize(4, 3);
        assert_eq!(screen.scroll_region(), (0, 2));
    }
}
