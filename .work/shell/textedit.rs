//! Single-line text editing for the shell's own fields: the browser address
//! bar, and the file manager's rename, path and search fields.
//!
//! A model only. Drawing is the renderer's job; this decides what a key does
//! to the text, which is the part that has to be right.
//!
//! Positions are byte offsets that always sit on a `char` boundary. Scripts
//! with combining marks (Thai vowel and tone marks, for instance) are edited
//! per `char`, not per grapheme cluster, so the caret can stop between a base
//! character and its mark. That is a known limitation of this model, not a
//! crash: every offset stays valid.

use litric_input::keys::{Key, Modifiers};

/// What a key did to a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditOutcome {
    /// The text changed.
    Changed,
    /// Only the caret or the selection moved.
    Moved,
    /// Enter: the caller should act on the text.
    Submit,
    /// Escape: the caller should abandon the edit.
    Cancel,
    /// The key means nothing to a text field; someone else may use it.
    Ignored,
}

/// A single line of editable text with a caret and an optional selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    text: String,
    caret: usize,
    anchor: Option<usize>,
    max_chars: usize,
}

impl Default for TextEdit {
    fn default() -> Self {
        Self::new()
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl TextEdit {
    /// The limit used by [`Self::new`]. Longer than any URL a browser keeps
    /// and any path the file protocol accepts.
    pub const DEFAULT_MAX_CHARS: usize = 8192;

    /// An empty field.
    #[must_use]
    pub fn new() -> Self {
        Self::with_limit(Self::DEFAULT_MAX_CHARS)
    }

    /// An empty field that holds at most `max_chars` characters.
    #[must_use]
    pub fn with_limit(max_chars: usize) -> Self {
        Self {
            text: String::new(),
            caret: 0,
            anchor: None,
            max_chars,
        }
    }

    /// The text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether there is no text.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The caret, as a byte offset on a character boundary.
    #[must_use]
    pub const fn caret(&self) -> usize {
        self.caret
    }

    /// The text before the caret, for measuring where to draw it.
    #[must_use]
    pub fn before_caret(&self) -> &str {
        self.text.get(..self.caret).unwrap_or("")
    }

    /// Replace the whole text. Control characters are dropped, the limit is
    /// applied, the caret goes to the end and any selection is cleared.
    pub fn set_text(&mut self, text: &str) {
        self.text = text
            .chars()
            .filter(|c| !c.is_control())
            .take(self.max_chars)
            .collect();
        self.caret = self.text.len();
        self.anchor = None;
    }

    /// Remove everything.
    pub fn clear(&mut self) {
        self.set_text("");
    }

    /// Select the whole text, caret at the end.
    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.caret = self.text.len();
    }

    /// Select a byte range. Each end is clamped to the text and moved back to
    /// the nearest character boundary at or before it. Used to pre-select a
    /// file name's stem when renaming.
    pub fn select_range(&mut self, start: usize, end: usize) {
        let a = self.floor_boundary(start.min(self.text.len()));
        let b = self.floor_boundary(end.min(self.text.len()));
        self.anchor = Some(a);
        self.caret = b;
    }

    /// The selection as an ordered byte range, or `None` when nothing (or an
    /// empty range) is selected.
    #[must_use]
    pub fn selection(&self) -> Option<(usize, usize)> {
        let anchor = self.anchor?;
        if anchor == self.caret {
            return None;
        }
        Some((anchor.min(self.caret), anchor.max(self.caret)))
    }

    /// The selected text.
    #[must_use]
    pub fn selected_text(&self) -> Option<&str> {
        let (a, b) = self.selection()?;
        self.text.get(a..b)
    }

    /// Remove the selection and return it (for cut).
    pub fn take_selection(&mut self) -> Option<String> {
        let (a, b) = self.selection()?;
        let taken = self.text.get(a..b)?.to_owned();
        self.text.replace_range(a..b, "");
        self.caret = a;
        self.anchor = None;
        Some(taken)
    }

    fn floor_boundary(&self, mut i: usize) -> usize {
        while i > 0 && !self.text.is_char_boundary(i) {
            i = i.saturating_sub(1);
        }
        i
    }

    fn prev_boundary(&self, i: usize) -> usize {
        self.text
            .get(..i)
            .and_then(|s| s.char_indices().next_back())
            .map_or(0, |(j, _)| j)
    }

    fn next_boundary(&self, i: usize) -> usize {
        self.text
            .get(i..)
            .and_then(|s| s.chars().next())
            .map_or(self.text.len(), |c| i.saturating_add(c.len_utf8()))
    }

    fn char_before(&self, i: usize) -> Option<char> {
        self.text.get(..i).and_then(|s| s.chars().next_back())
    }

    fn char_at(&self, i: usize) -> Option<char> {
        self.text.get(i..).and_then(|s| s.chars().next())
    }

    /// Start of the word at or before `i`: skip separators, then the word.
    fn word_left(&self, mut i: usize) -> usize {
        while self.char_before(i).is_some_and(|c| !is_word(c)) {
            i = self.prev_boundary(i);
        }
        while self.char_before(i).is_some_and(is_word) {
            i = self.prev_boundary(i);
        }
        i
    }

    /// End of the word at or after `i`: skip separators, then the word.
    fn word_right(&self, mut i: usize) -> usize {
        while self.char_at(i).is_some_and(|c| !is_word(c)) {
            i = self.next_boundary(i);
        }
        while self.char_at(i).is_some_and(is_word) {
            i = self.next_boundary(i);
        }
        i
    }

    /// Insert text at the caret, replacing any selection. Control characters
    /// are dropped and only as many characters as fit under the limit are
    /// inserted. Returns whether the text changed.
    pub fn insert(&mut self, s: &str) -> bool {
        let clean: Vec<char> = s.chars().filter(|c| !c.is_control()).collect();
        let had_selection = self.take_selection().is_some();
        let room = self.max_chars.saturating_sub(self.text.chars().count());
        let piece: String = clean.into_iter().take(room).collect();
        if piece.is_empty() {
            return had_selection;
        }
        self.text.insert_str(self.caret, &piece);
        self.caret = self.caret.saturating_add(piece.len());
        self.anchor = None;
        true
    }

    /// Backspace: delete the selection, or the character (or word) before the
    /// caret. Returns whether the text changed.
    pub fn backspace(&mut self, word: bool) -> bool {
        if self.take_selection().is_some() {
            return true;
        }
        let from = if word {
            self.word_left(self.caret)
        } else {
            self.prev_boundary(self.caret)
        };
        if from == self.caret {
            return false;
        }
        self.text.replace_range(from..self.caret, "");
        self.caret = from;
        true
    }

    /// Delete: remove the selection, or the character (or word) after the
    /// caret. Returns whether the text changed.
    pub fn delete_forward(&mut self, word: bool) -> bool {
        if self.take_selection().is_some() {
            return true;
        }
        let to = if word {
            self.word_right(self.caret)
        } else {
            self.next_boundary(self.caret)
        };
        if to == self.caret {
            return false;
        }
        self.text.replace_range(self.caret..to, "");
        true
    }

    fn move_to(&mut self, to: usize, select: bool) -> bool {
        let before = (self.caret, self.anchor);
        if select {
            if self.anchor.is_none() {
                self.anchor = Some(self.caret);
            }
        } else {
            self.anchor = None;
        }
        self.caret = to;
        (self.caret, self.anchor) != before
    }

    /// Move one character (or word) left. Without `select`, an existing
    /// selection collapses to its start instead of moving, which is what every
    /// text field does.
    pub fn move_left(&mut self, select: bool, word: bool) -> bool {
        if !select {
            if let Some((a, _)) = self.selection() {
                return self.move_to(a, false);
            }
        }
        let to = if word {
            self.word_left(self.caret)
        } else {
            self.prev_boundary(self.caret)
        };
        self.move_to(to, select)
    }

    /// Move one character (or word) right; collapses a selection to its end.
    pub fn move_right(&mut self, select: bool, word: bool) -> bool {
        if !select {
            if let Some((_, b)) = self.selection() {
                return self.move_to(b, false);
            }
        }
        let to = if word {
            self.word_right(self.caret)
        } else {
            self.next_boundary(self.caret)
        };
        self.move_to(to, select)
    }

    /// Caret to the start.
    pub fn home(&mut self, select: bool) -> bool {
        self.move_to(0, select)
    }

    /// Caret to the end.
    pub fn end(&mut self, select: bool) -> bool {
        self.move_to(self.text.len(), select)
    }

    /// Apply a key press. Printable characters arrive separately as text (see
    /// [`Self::handle_text`]), because the layout decides what a key types.
    /// Clipboard shortcuts are left to the caller, which owns the clipboard.
    pub fn handle_key(&mut self, key: Key, mods: Modifiers) -> EditOutcome {
        let moved = |b: bool| {
            if b {
                EditOutcome::Moved
            } else {
                EditOutcome::Ignored
            }
        };
        let changed = |b: bool| {
            if b {
                EditOutcome::Changed
            } else {
                EditOutcome::Moved
            }
        };
        match key {
            Key::Enter => EditOutcome::Submit,
            Key::Escape => EditOutcome::Cancel,
            Key::Backspace => changed(self.backspace(mods.ctrl)),
            Key::Delete => changed(self.delete_forward(mods.ctrl)),
            Key::Left => {
                self.move_left(mods.shift, mods.ctrl);
                EditOutcome::Moved
            }
            Key::Right => {
                self.move_right(mods.shift, mods.ctrl);
                EditOutcome::Moved
            }
            Key::Home => {
                self.home(mods.shift);
                EditOutcome::Moved
            }
            Key::End => {
                self.end(mods.shift);
                EditOutcome::Moved
            }
            Key::Char('a' | 'A') if mods.ctrl && !mods.alt && !mods.meta => {
                self.select_all();
                moved(true)
            }
            _ => EditOutcome::Ignored,
        }
    }

    /// Apply typed text. Ignored when it contains nothing printable, or when a
    /// Ctrl/Alt/Meta chord produced it (those are shortcuts, not typing).
    pub fn handle_text(&mut self, text: &str, mods: Modifiers) -> EditOutcome {
        if mods.ctrl || mods.meta || (mods.alt && !mods.altgr) {
            return EditOutcome::Ignored;
        }
        if self.insert(text) {
            EditOutcome::Changed
        } else {
            EditOutcome::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m() -> Modifiers {
        Modifiers::NONE
    }

    fn shift() -> Modifiers {
        Modifiers {
            shift: true,
            ..Modifiers::NONE
        }
    }

    fn ctrl() -> Modifiers {
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        }
    }

    fn field(s: &str) -> TextEdit {
        let mut t = TextEdit::new();
        t.set_text(s);
        t
    }

    #[test]
    fn typing_inserts_at_the_caret_and_moves_it() {
        let mut t = TextEdit::new();
        assert_eq!(t.handle_text("exa", m()), EditOutcome::Changed);
        t.handle_key(Key::Left, m());
        t.handle_key(Key::Left, m());
        t.handle_text("Z", m());
        assert_eq!(t.text(), "eZxa");
        assert_eq!(t.before_caret(), "eZ");
    }

    #[test]
    fn backspace_and_delete_remove_one_character_each_side() {
        let mut t = field("abcd");
        t.handle_key(Key::Left, m());
        t.handle_key(Key::Left, m());
        assert_eq!(t.handle_key(Key::Backspace, m()), EditOutcome::Changed);
        assert_eq!(t.text(), "acd");
        assert_eq!(t.handle_key(Key::Delete, m()), EditOutcome::Changed);
        assert_eq!(t.text(), "ad");
        t.handle_key(Key::Home, m());
        assert_eq!(
            t.handle_key(Key::Backspace, m()),
            EditOutcome::Moved,
            "backspace at the start changes nothing"
        );
        assert_eq!(t.text(), "ad");
    }

    #[test]
    fn multibyte_text_is_edited_by_character_without_ever_splitting_utf8() {
        // Every offset must stay on a char boundary, or the next slice panics.
        let mut t = field("กข€😀");
        for _ in 0..10 {
            t.handle_key(Key::Left, m());
            assert!(t.text().is_char_boundary(t.caret()));
        }
        assert_eq!(t.caret(), 0);
        t.handle_key(Key::Delete, m());
        assert_eq!(t.text(), "ข€😀");
        t.handle_key(Key::End, m());
        t.handle_key(Key::Backspace, m());
        assert_eq!(t.text(), "ข€");
        assert!(t.text().is_char_boundary(t.caret()));
    }

    #[test]
    fn shift_arrows_select_and_typing_replaces_the_selection() {
        let mut t = field("hello world");
        t.handle_key(Key::Home, m());
        for _ in 0..5 {
            t.handle_key(Key::Right, shift());
        }
        assert_eq!(t.selected_text(), Some("hello"));
        t.handle_text("goodbye", m());
        assert_eq!(t.text(), "goodbye world");
        assert_eq!(t.selection(), None);
    }

    #[test]
    fn an_arrow_without_shift_collapses_the_selection_to_the_right_end() {
        let mut t = field("abcdef");
        t.select_range(1, 4);
        t.handle_key(Key::Left, m());
        assert_eq!((t.caret(), t.selection()), (1, None));
        t.select_range(1, 4);
        t.handle_key(Key::Right, m());
        assert_eq!((t.caret(), t.selection()), (4, None));
    }

    #[test]
    fn ctrl_a_selects_everything_and_backspace_clears_it() {
        let mut t = field("https://example.com/");
        assert_eq!(t.handle_key(Key::Char('a'), ctrl()), EditOutcome::Moved);
        assert_eq!(t.selected_text(), Some("https://example.com/"));
        assert_eq!(t.handle_key(Key::Backspace, m()), EditOutcome::Changed);
        assert!(t.is_empty());
    }

    #[test]
    fn ctrl_arrows_and_ctrl_backspace_work_by_word() {
        let mut t = field("open the file_name.txt now");
        t.handle_key(Key::End, m());
        t.handle_key(Key::Left, ctrl());
        assert_eq!(t.before_caret(), "open the file_name.txt ");
        t.handle_key(Key::Left, ctrl());
        assert_eq!(t.before_caret(), "open the file_name.");
        t.handle_key(Key::Backspace, ctrl());
        assert_eq!(t.text(), "open the txt now", "underscore is part of a word");
        t.handle_key(Key::Home, m());
        t.handle_key(Key::Right, ctrl());
        assert_eq!(t.before_caret(), "open");
    }

    #[test]
    fn control_characters_never_enter_the_field() {
        // A newline in a file name or an address bar is a protocol break.
        let mut t = TextEdit::new();
        t.handle_text("a\nb\tc\u{7}d", m());
        assert_eq!(t.text(), "abcd");
        t.set_text("x\r\ny");
        assert_eq!(t.text(), "xy");
    }

    #[test]
    fn the_character_limit_is_enforced_on_typing_and_on_set_text() {
        let mut t = TextEdit::with_limit(5);
        t.handle_text("abcdefgh", m());
        assert_eq!(t.text(), "abcde");
        assert_eq!(t.handle_text("z", m()), EditOutcome::Ignored);
        t.select_range(0, 2);
        assert_eq!(t.handle_text("XYZ", m()), EditOutcome::Changed);
        // Replacing "ab" frees two characters of room, so two of the three
        // typed characters fit; the limit holds and no existing text is lost.
        assert_eq!(t.text(), "XYcde");
        t.set_text("0123456789");
        assert_eq!(t.text(), "01234");
    }

    #[test]
    fn enter_submits_escape_cancels_and_unrelated_keys_are_ignored() {
        let mut t = field("abc");
        assert_eq!(t.handle_key(Key::Enter, m()), EditOutcome::Submit);
        assert_eq!(t.handle_key(Key::Escape, m()), EditOutcome::Cancel);
        assert_eq!(t.handle_key(Key::Tab, m()), EditOutcome::Ignored);
        assert_eq!(t.handle_key(Key::Up, m()), EditOutcome::Ignored);
        assert_eq!(t.text(), "abc");
    }

    #[test]
    fn chord_text_is_not_typed() {
        let mut t = TextEdit::new();
        assert_eq!(t.handle_text("c", ctrl()), EditOutcome::Ignored);
        assert!(t.is_empty());
    }

    #[test]
    fn select_range_snaps_inside_multibyte_characters_and_clamps() {
        let mut t = field("aก");
        t.select_range(2, 99); // 2 is inside the three-byte ก
        assert_eq!(t.selection(), Some((1, 4)));
        assert_eq!(t.selected_text(), Some("ก"));
    }

    #[test]
    fn cut_removes_the_selection_and_returns_it() {
        let mut t = field("report-final.pdf");
        t.select_range(0, 12);
        assert_eq!(t.take_selection().as_deref(), Some("report-final"));
        assert_eq!(t.text(), ".pdf");
        assert_eq!(t.caret(), 0);
        assert_eq!(t.take_selection(), None);
    }
}
