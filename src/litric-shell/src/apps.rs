//! The shell's application models: Files, Litric Browser, the Defender and
//! the desktop's icon grid.
//!
//! # Layer position
//!
//! These are *models*, not views: they hold state and produce decisions, and
//! [`crate::views`] draws whatever the model reports. Nothing here touches the
//! framebuffer.
//!
//! # Files — modelled on the Windows Explorer feature set
//!
//! Research baseline (Windows 11 File Explorer): navigation pane with pinned
//! places, breadcrumb address bar, command bar, details pane, list/details
//! view modes, search, tabs. This model implements the state machine behind
//! that surface on LitricOS primitives: logical paths resolve through
//! [`crate::fsbridge`]'s no-symlink-follow root, listing maps to a table with
//! sortable columns, a details selection, a search filter, and pinned places
//! (Quick access). The model also owns the Files clipboard (copy/cut) and the
//! pending dialog state (rename, new folder), so the window manager's input
//! handler stays a thin router.
//!
//! # Browser — modelled on the Chrome window model
//!
//! Research baseline (Chromium): a browser process owns the tab strip and the
//! omnibox; each tab holds its own session. The omnibox builds URLs the way
//! Chrome's does — scheme first, then a plausible host, then a web search,
//! with the search engine configurable and Google as the default.
//!
//! # Defender — a scan model around the ADR-0022 engine
//!
//! The engine in `litric-defender` is pure: it takes bytes and returns a
//! verdict. This model owns everything around it that is still not I/O:
//! the queue of files to scan, progress, the verdict list, and the summary
//! the notification banner shows.

use std::collections::VecDeque;

use crate::shell::Hot;
use crate::terminal_screen::{CellStyle, Color, Screen};
use litric_defender::verdict::Decision;
use litric_input::keys::Modifiers;

/// A text file opened in the viewer, with its own scroll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileViewer {
    /// The logical path being shown.
    pub path: String,
    /// The text (lossy-decoded, size-capped by the bridge).
    pub text: String,
    /// First visible line.
    pub scroll: usize,
    /// Byte offset of each line's first character, built once at open.
    ///
    /// `window()` used to collect the whole text into a `Vec<String>` on every
    /// redraw, and `draw_viewer` cloned the viewer first: a 64 KiB file was
    /// split into owned lines about sixty times a second to show the twenty
    /// that fit. The offsets never change after the file is read, so they are
    /// indexed once, and a page is a slice of the text that is already there.
    line_starts: Vec<usize>,
}

impl FileViewer {
    /// A viewer over `text`, with its line index built.
    #[must_use]
    pub fn new(path: String, text: String) -> Self {
        let mut line_starts = Vec::new();
        if !text.is_empty() {
            line_starts.push(0);
            for (i, _) in text.match_indices('\n') {
                // A newline at the very end does not open a line: `lines()`
                // and every editor show "a\n" as one line, not two.
                let next = i.saturating_add(1);
                if next < text.len() {
                    line_starts.push(next);
                }
            }
        }
        Self {
            path,
            text,
            scroll: 0,
            line_starts,
        }
    }

    /// The visible lines, given how many fit, and the line count.
    ///
    /// The returned slices borrow this viewer, so a caller that needs to mutate
    /// the shell while drawing copies only the handful of lines it shows.
    #[must_use]
    pub fn window(&self, rows: usize) -> (Vec<&str>, usize) {
        let total = self.line_starts.len();
        if total == 0 {
            return (Vec::new(), 0);
        }
        let start = self.scroll.min(total.saturating_sub(1));
        let end = start.saturating_add(rows).min(total);
        let mut lines = Vec::with_capacity(end.saturating_sub(start));
        for i in start..end {
            let from = match self.line_starts.get(i) {
                Some(&at) => at,
                None => break,
            };
            let to = self
                .line_starts
                .get(i.saturating_add(1))
                .map_or(self.text.len(), |next| next.saturating_sub(1));
            // The last line's end is the end of the file, which may itself be a
            // newline; the others end just before theirs. Either way the line
            // ending is not content, and a CRLF file would otherwise draw a
            // stray glyph at every row end.
            let line = self.text.get(from..to).unwrap_or_default();
            let line = line.strip_suffix('\n').unwrap_or(line);
            lines.push(line.strip_suffix('\r').unwrap_or(line));
        }
        (lines, total)
    }
}

/// Decode a protocol field value (`name=%64oc.txt`). `-` means empty.
fn decode(v: &str) -> String {
    if v == "-" {
        return String::new();
    }
    let bytes = v.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut iter = bytes.iter().copied().peekable();
    while let Some(b) = iter.next() {
        if b == b'%' {
            // Two hex digits follow? Peek both BEFORE consuming: only a full
            // valid pair is eaten, so %zz, %z and a trailing % stay literal.
            let pair = iter.peek().copied().zip(iter.clone().nth(1));
            if let Some((h, l)) = pair {
                if let (Some(hi), Some(lo)) =
                    (char::from(h).to_digit(16), char::from(l).to_digit(16))
                {
                    out.push((hi.wrapping_mul(16).wrapping_add(lo)) as u8);
                    iter.next(); // consume the high digit
                    iter.next(); // consume the low digit
                    continue;
                }
            }
        }
        out.push(b);
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// One row in a Files listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRow {
    /// Display name (percent-decoded).
    pub name: String,
    /// Directory, file, link or other.
    pub kind: FileKind,
    /// Bytes, for files.
    pub size: u64,
    /// Unix mtime seconds.
    pub mtime: u64,
}

/// The kind of a listed entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// A directory.
    Dir,
    /// A regular file.
    File,
    /// A symbolic link.
    Link,
    /// Anything else.
    Other,
}

impl FileKind {
    /// From the protocol's `kind=` token.
    #[must_use]
    pub fn from_token(token: &str) -> Self {
        match token {
            "dir" => Self::Dir,
            "file" => Self::File,
            "link" => Self::Link,
            _ => Self::Other,
        }
    }

    /// The protocol's token, for round-trips.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Dir => "dir",
            Self::File => "file",
            Self::Link => "link",
            Self::Other => "other",
        }
    }
}

/// Sort keys for the details view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortKey {
    /// Name, case-insensitive.
    Name,
    /// Size, largest first.
    Size,
    /// Modification time, newest first.
    Modified,
}

/// Which pane the Files window is interacting with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesFocus {
    /// The places sidebar.
    Places,
    /// The entry table.
    Table,
}

/// What a Files dialog is asking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    /// Create a folder with the name in the field.
    NewFolder,
    /// Create an empty text file with the name in the field.
    NewFile,
    /// Rename the stored logical path to the name in the field.
    Rename {
        /// The full logical path of the item being renamed.
        path: String,
        /// The name it has now (pre-selected in the field).
        current: String,
    },
    /// The user was asked to confirm; proceed when submitted.
    Confirm {
        /// What will happen.
        title: String,
        /// The detail line.
        detail: String,
        /// The action to carry out on confirm.
        action: FilesOp,
    },

    /// An error to acknowledge.
    Error(String),
}

/// A dialog the desktop itself owns, as opposed to one a window owns.
///
/// Deliberately a different type from [`DialogKind`], not a variant of it. The
/// two dialogs share a look and share nothing else: a Files dialog's action is a
/// [`FilesOp`], which names a path inside the storage sandbox, and this one's is
/// a power transition, which names the machine. One enum would let a file
/// operation be retargeted at the machine and a power transition be handed to a
/// window, and the compiler could not tell the difference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopDialog {
    /// Create a folder under `/Documents`.
    NewFolder,
    /// Install a local application bundle from `litric/incoming`.
    InstallApp,
    /// Remove an installed application after the user confirms it.
    UninstallApp {
        /// The reverse-DNS id the daemon installed it under.
        id: String,
        /// The name the launcher showed for it.
        name: String,
    },
    /// Confirm a power transition named by the user.
    PowerConfirm {
        /// Which transition, from [`crate::shell::PowerCommand::ALL`].
        command: crate::shell::PowerCommand,
    },
    /// Something the desktop tried to do failed.
    Error(String),
}

/// A filesystem operation a dialog or menu item can ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilesOp {
    /// Move to trash.
    Trash(String),
    /// Move several selected entries to trash.
    TrashMany(Vec<String>),
    /// Delete for ever.
    Delete(String),
    /// Permanently delete several selected entries.
    DeleteMany(Vec<String>),
    /// Copy from the clipboard into the shown folder.
    PasteCopy,
    /// Move from the clipboard into the shown folder.
    PasteCut,
    /// Compress the clipboard selection into `name`.
    Compress(String),
    /// Compress several selected entries into one archive.
    CompressMany {
        /// The archive file's name.
        name: String,
        /// The selected logical file paths to add.
        sources: Vec<String>,
    },
    /// Extract an archive into a folder beside it.
    Extract(String),
}

/// The Files application model.
#[derive(Debug, Clone)]
pub struct FilesApp {
    /// Places shown in the navigation pane, in order.
    pub places: Vec<(String, String)>,
    /// Index into `places` currently highlighted, if any.
    pub place_index: Option<usize>,
    /// How far the places list is scrolled, in device pixels.
    ///
    /// The view clamps it against the pane's real height, so a window that
    /// grows hands the space back instead of keeping an offset that hides the
    /// first places behind nothing.
    pub places_scroll: f32,
    /// The current directory's logical path, e.g. `/Documents`.
    pub path: String,
    /// The listing of `path` as received (already decoded rows).
    pub rows: Vec<FileRow>,
    /// Selected row index.
    pub selected: Option<usize>,
    /// Selected row names, stable when the user changes sort order.
    pub selected_rows: std::collections::BTreeSet<String>,
    /// The primary selected row's name, stable across sorting and filtering.
    pub selected_name: Option<String>,
    /// A row to select as soon as its listing arrives.
    ///
    /// Navigation clears the selection and the listing is read on the next
    /// frame, so a caller in another window that names a file (a download
    /// opened from the Browser) records the name here and the table selects it
    /// when the rows exist. It is spent once, never left to surprise a later
    /// folder.
    pub pending_selection: Option<String>,
    /// Current sort.
    pub sort: (SortKey, bool),
    /// Search filter; empty means no filter.
    pub filter: String,
    /// Details pane visible?
    pub details: bool,
    /// Which pane has keyboard focus.
    pub focus: FilesFocus,
    /// Back history (previous paths), newest last.
    pub back: Vec<String>,
    /// Forward history, filled by Back, drained by navigation.
    pub forward: Vec<String>,
    /// The path whose listing is currently loaded. `None` (or a value
    /// different from `path`) means the view must refresh from the filesystem.
    pub listed: Option<String>,
    /// The last listing error, shown in place of an empty table.
    pub error: Option<String>,
    /// The Files clipboard. `true` = cut (move), `false` = copy.
    pub clipboard: Option<(bool, String)>,
    /// A modal dialog over the window, if one is open.
    pub dialog: Option<DialogKind>,
    /// Whether the rename/new-name field has keyboard focus.
    pub dialog_focus: bool,
    /// The name being typed into the dialog field.
    pub dialog_field: String,
    /// Byte-range selection inside `dialog_field` (start, end), if any.
    pub dialog_selection: Option<(usize, usize)>,
    /// A text file open in the viewer pane.
    pub viewer: Option<FileViewer>,
    /// Whether a drag-payload ghost is being carried (visual only).
    pub dragging: bool,
}

impl Default for FilesApp {
    fn default() -> Self {
        Self {
            places_scroll: 0.0,
            places: Self::DEFAULT_PLACES
                .iter()
                .map(|(p, l)| ((*p).to_owned(), (*l).to_owned()))
                .collect(),
            place_index: None,
            path: "/Documents".to_owned(),
            rows: Vec::new(),
            selected: None,
            selected_rows: std::collections::BTreeSet::new(),
            selected_name: None,
            pending_selection: None,
            sort: (SortKey::Name, true),
            filter: String::new(),
            details: true,
            focus: FilesFocus::Table,
            back: Vec::new(),
            forward: Vec::new(),
            listed: None,
            error: None,
            clipboard: None,
            dialog: None,
            dialog_focus: false,
            dialog_field: String::new(),
            dialog_selection: None,
            viewer: None,
            dragging: false,
        }
    }
}

impl FilesApp {
    /// The navigation pane's default places (Quick access).
    ///
    /// The first row is the home root, the way Explorer starts at This PC: a
    /// place that gathers every folder under the account in one list, so a
    /// person who does not yet know where a file lives has somewhere to start.
    pub const DEFAULT_PLACES: [(&str, &str); 7] = [
        ("/", "This Litric"),
        ("/Documents", "Documents"),
        ("/Pictures", "Pictures"),
        ("/Music", "Music"),
        ("/Videos", "Videos"),
        ("/Downloads", "Downloads"),
        ("/Projects", "Projects"),
    ];

    /// Rows after the search filter, in the current sort order.
    #[must_use]
    pub fn visible(&self) -> Vec<&FileRow> {
        let needle = self.filter.to_lowercase();
        let mut rows: Vec<&FileRow> = self
            .rows
            .iter()
            .filter(|r| needle.is_empty() || r.name.to_lowercase().contains(&needle))
            .collect();
        let dirs_first =
            |a: &FileRow, b: &FileRow| match (a.kind == FileKind::Dir, b.kind == FileKind::Dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };
        match self.sort {
            (SortKey::Name, asc) => {
                rows.sort_by(|a, b| {
                    dirs_first(a, b).then_with(|| {
                        let c = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                        if asc {
                            c
                        } else {
                            c.reverse()
                        }
                    })
                });
            }
            (SortKey::Size, asc) => {
                rows.sort_by(|a, b| {
                    dirs_first(a, b).then_with(|| {
                        let c = a.size.cmp(&b.size);
                        if asc {
                            c
                        } else {
                            c.reverse()
                        }
                    })
                });
            }
            (SortKey::Modified, asc) => {
                rows.sort_by(|a, b| {
                    dirs_first(a, b).then_with(|| {
                        let c = a.mtime.cmp(&b.mtime);
                        if asc {
                            c
                        } else {
                            c.reverse()
                        }
                    })
                });
            }
        }
        rows
    }

    /// Breadcrumb segments for the current path (`/Documents/Work` →
    /// `["Documents", "Work"]`).
    #[must_use]
    pub fn breadcrumbs(&self) -> Vec<String> {
        self.path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// The path a breadcrumb click navigates to (`i` is the segment index;
    /// the root breadcrumb is not drawn, matching Explorer's drive root).
    /// The index clamps; the empty root path yields `/`.
    #[must_use]
    pub fn breadcrumb_path(&self, i: usize) -> String {
        let parts = self.breadcrumbs();
        if parts.is_empty() {
            return "/".to_owned();
        }
        let mut out = String::new();
        for p in parts.iter().take(i.checked_add(1).unwrap_or(parts.len())) {
            out.push('/');
            out.push_str(p);
        }
        out
    }

    /// Navigate to a path, keeping history.
    pub fn navigate(&mut self, path: &str) {
        if path == self.path {
            return;
        }
        self.back.push(std::mem::take(&mut self.path));
        self.forward.clear();
        self.path = path.to_owned();
        self.selected = None;
        self.selected_rows.clear();
        self.selected_name = None;
        self.filter.clear();
        self.place_index = self.places.iter().position(|(p, _)| p == &self.path);
    }

    /// Go back one entry; returns whether there was somewhere to go.
    pub fn go_back(&mut self) -> bool {
        let Some(prev) = self.back.pop() else {
            return false;
        };
        let cur = std::mem::replace(&mut self.path, prev);
        self.forward.push(cur);
        self.selected = None;
        self.selected_rows.clear();
        self.selected_name = None;
        true
    }

    /// Go forward one entry; returns whether there was somewhere to go.
    pub fn go_forward(&mut self) -> bool {
        let Some(next) = self.forward.pop() else {
            return false;
        };
        let cur = std::mem::replace(&mut self.path, next);
        self.back.push(cur);
        self.selected = None;
        self.selected_rows.clear();
        self.selected_name = None;
        true
    }

    /// Whether Back would move anywhere.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// Whether Forward would move anywhere.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Whether Up would move anywhere. The root is the stop.
    #[must_use]
    pub fn can_go_up(&self) -> bool {
        !self.path.is_empty() && self.path != "/"
    }

    /// Go up one level. `/Documents/Work` → `/Documents`; the root path
    /// stays put.
    pub fn go_up(&mut self) {
        let parent = match self.path.rsplit_once('/') {
            Some(("", _)) | Some(("/", _)) => "/".to_owned(),
            Some((parent, _)) => parent.to_owned(),
            None => return,
        };
        if parent != self.path {
            self.navigate(&parent);
        }
    }

    /// Toggle the sort column; a second click on the same column flips it.
    pub fn toggle_sort(&mut self, key: SortKey) {
        if self.sort.0 == key {
            self.sort.1 = !self.sort.1;
        } else {
            self.sort = (key, true);
        }
    }

    /// Parse a LIST reply (`e name=%… kind=dir size=0 mtime=0 …` lines) into
    /// rows, replacing the current listing. Unknown lines are ignored so the
    /// reply may grow without breaking the window.
    pub fn apply_list_reply(&mut self, reply: &str) {
        let mut rows = Vec::new();
        for line in reply.lines() {
            let Some(rest) = line.strip_prefix("e ") else {
                continue;
            };
            let mut name = String::new();
            let mut kind = FileKind::Other;
            let mut size = 0;
            let mut mtime = 0;
            for pair in rest.split(' ') {
                let Some((k, v)) = pair.split_once('=') else {
                    continue;
                };
                match k {
                    "name" => name = decode(v),
                    "kind" => kind = FileKind::from_token(v),
                    "size" => size = v.parse().unwrap_or(0),
                    "mtime" => mtime = v.parse().unwrap_or(0),
                    _ => {}
                }
            }
            if !name.is_empty() {
                rows.push(FileRow {
                    name,
                    kind,
                    size,
                    mtime,
                });
            }
        }
        self.rows = rows;
        self.selected = None;
        self.selected_rows.clear();
        self.selected_name = None;
    }

    /// Replace the listing from a fresh directory read. Sorting, filter and
    /// the search box survive; the selection does not (rows moved).
    pub fn apply_entries(&mut self, rows: Vec<FileRow>) {
        self.rows = rows;
        self.selected = None;
        self.selected_rows.clear();
        self.selected_name = None;
        self.error = None;
        if let Some(name) = self.pending_selection.take() {
            if let Some(index) = self.visible().iter().position(|row| row.name == name) {
                self.selected = Some(index);
                self.selected_name = Some(name.clone());
                self.selected_rows.insert(name);
            }
        }
    }

    /// Record that the directory could not be read, and drop the rows.
    ///
    /// Keeping the previous rows behind an error banner shows a listing that is
    /// no longer what is on disk. The window draws the reason in place of the
    /// table instead, which is the difference between "this folder is broken"
    /// and "this folder is empty".
    pub fn apply_error(&mut self, message: String) {
        self.rows.clear();
        self.selected = None;
        self.selected_rows.clear();
        self.selected_name = None;
        self.error = Some(message);
    }

    /// The selected row, if any.
    #[must_use]
    pub fn selection(&self) -> Option<&FileRow> {
        let vis = self.visible();
        if let Some(name) = &self.selected_name {
            vis.into_iter().find(|row| &row.name == name)
        } else {
            vis.into_iter().nth(self.selected?)
        }
    }

    /// The logical path of the selected row.
    #[must_use]
    pub fn selected_path(&self) -> Option<String> {
        let base = self.path.trim_end_matches('/').to_owned();
        self.selection().map(|r| {
            if base.is_empty() {
                format!("/{}", r.name)
            } else {
                format!("{base}/{}", r.name)
            }
        })
    }

    /// Visible logical paths in the current selection, in table order.
    #[must_use]
    pub fn selected_paths(&self) -> Vec<String> {
        let rows = self.visible();
        self.selected_paths_in(&rows)
    }

    /// [`Self::selected_paths`] over a row list the caller already built.
    ///
    /// `visible()` filters, sorts and allocates over the whole listing, so a
    /// view that needs the selection for a readout must not call
    /// `selected_paths()` after it has already built the ordered rows: that is
    /// a second full sort per frame for a number.
    #[must_use]
    pub fn selected_paths_in(&self, rows: &[&FileRow]) -> Vec<String> {
        let selected = if self.selected_rows.is_empty() {
            self.selected_name
                .clone()
                .or_else(|| {
                    self.selected
                        .and_then(|index| rows.get(index).map(|r| r.name.clone()))
                })
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
        } else {
            self.selected_rows.clone()
        };
        let base = self.path.trim_end_matches('/');
        rows.iter()
            .filter(|row| selected.contains(&row.name))
            .map(|row| {
                if base.is_empty() {
                    format!("/{}", row.name)
                } else {
                    format!("{base}/{}", row.name)
                }
            })
            .collect()
    }

    /// Scroll the places list by `dy` wheel notches.
    ///
    /// The view clamps the result: only it knows the pane's height and how
    /// many rows fit, and a model that guessed would keep an offset that no
    /// longer describes a scrollable list.
    pub fn scroll_places(&mut self, dy: f32) {
        self.places_scroll += dy * 44.0;
        if self.places_scroll < 0.0 {
            self.places_scroll = 0.0;
        }
    }

    /// Whether a visible row is part of the multi-selection.
    #[must_use]
    pub fn row_is_selected(&self, index: usize) -> bool {
        let rows = self.visible();
        self.row_is_selected_in(&rows, index)
    }

    /// [`Self::row_is_selected`] over a row list the caller already built.
    ///
    /// The Files view asks this for every drawn row. Asking `row_is_selected`
    /// did the filter-and-sort for each one -- up to 24 full sorts of a
    /// 10,000-entry listing per frame, with two lowercased temporaries per name
    /// comparison. The view now sorts once and passes the list in.
    #[must_use]
    pub fn row_is_selected_in(&self, rows: &[&FileRow], index: usize) -> bool {
        rows.get(index).is_some_and(|row| {
            self.selected_rows.contains(&row.name)
                || (self.selected_rows.is_empty()
                    && (self.selected_name.as_deref() == Some(row.name.as_str())
                        || (self.selected_name.is_none() && self.selected == Some(index))))
        })
    }

    /// Select a row by name, remembering the name when the listing has not
    /// arrived yet. This is the entry point for another window that knows the
    /// file it wants (a finished download opened from the Browser), so the user
    /// lands on that file rather than an empty selection.
    pub fn select_name(&mut self, name: &str) {
        if let Some(index) = self.visible().iter().position(|row| row.name == name) {
            self.selected = Some(index);
            self.selected_name = Some(name.to_owned());
            self.selected_rows.clear();
            self.selected_rows.insert(name.to_owned());
            self.pending_selection = None;
        } else {
            self.pending_selection = Some(name.to_owned());
        }
    }

    /// Replace the selection with one visible row.
    pub fn select_row(&mut self, index: usize) {
        if let Some(name) = self.visible().get(index).map(|row| row.name.clone()) {
            self.selected = Some(index);
            self.selected_name = Some(name.clone());
            self.selected_rows.clear();
            self.selected_rows.insert(name);
        }
    }

    /// Toggle one visible row without disturbing other selected rows.
    pub fn toggle_row(&mut self, index: usize) {
        if index >= self.visible().len() {
            return;
        }
        let Some(name) = self.visible().get(index).map(|row| row.name.clone()) else {
            return;
        };
        if self.selected_rows.is_empty() {
            if let Some(current) = self.selected_name.clone().or_else(|| {
                self.selected
                    .and_then(|current| self.visible().get(current).map(|row| row.name.clone()))
            }) {
                self.selected_rows.insert(current);
            }
        }
        if !self.selected_rows.remove(&name) {
            self.selected_rows.insert(name.clone());
            self.selected = Some(index);
            self.selected_name = Some(name);
        } else if self.selected_name.as_deref() == Some(name.as_str()) {
            let next = self.selected_rows.iter().next_back().cloned();
            self.selected_name = next.clone();
            self.selected =
                next.and_then(|name| self.visible().iter().position(|row| row.name == name));
        }
        if self.selected_rows.is_empty() {
            self.selected = None;
            self.selected_name = None;
        }
    }

    /// Select every currently visible row.
    pub fn select_all_visible(&mut self) {
        let (names, first) = {
            let visible = self.visible();
            (
                visible.iter().map(|row| row.name.clone()).collect(),
                visible.first().map(|row| row.name.clone()),
            )
        };
        self.selected_rows = names;
        self.selected_name = first;
        self.selected = self.selected_name.as_ref().map(|_| 0);
    }

    /// Copy the selection to the clipboard.
    pub fn copy_selection(&mut self) -> bool {
        match self.selected_path() {
            Some(p) => {
                self.clipboard = Some((false, p));
                true
            }
            None => false,
        }
    }

    /// Cut the selection to the clipboard.
    pub fn cut_selection(&mut self) -> bool {
        match self.selected_path() {
            Some(p) => {
                self.clipboard = Some((true, p));
                true
            }
            None => false,
        }
    }

    /// Open a dialog asking for a new folder name.
    pub fn ask_new_folder(&mut self) {
        self.dialog = Some(DialogKind::NewFolder);
        self.dialog_field = String::new();
        self.dialog_focus = true;
    }

    /// Open a dialog asking for a new text file name.
    pub fn ask_new_file(&mut self) {
        self.dialog = Some(DialogKind::NewFile);
        self.dialog_field = "untitled.txt".to_owned();
        // Pre-select the stem, the way Explorer does when it invents a name.
        self.dialog_select_stem();
        self.dialog_focus = true;
    }

    /// Open a rename dialog for the selection.
    pub fn ask_rename(&mut self) {
        let Some(path) = self.selected_path() else {
            return;
        };
        let Some(current) = self.selection().map(|r| r.name.clone()) else {
            return;
        };
        self.dialog = Some(DialogKind::Rename {
            path,
            current: current.clone(),
        });
        self.dialog_field = current;
        self.dialog_select_stem();
        self.dialog_focus = true;
    }

    /// Open a confirm dialog for a filesystem operation.
    pub fn ask_confirm(&mut self, title: &str, detail: &str, action: FilesOp) {
        self.dialog = Some(DialogKind::Confirm {
            title: title.to_owned(),
            detail: detail.to_owned(),
            action,
        });
        self.dialog_field.clear();
        self.dialog_focus = false;
    }

    /// Pre-select the file-name stem in the dialog field.
    fn dialog_select_stem(&mut self) {
        if let Some(dot) = self.dialog_field.rfind('.') {
            if dot > 0 {
                self.dialog_select_range(0, dot);
            }
        }
    }

    fn dialog_select_range(&mut self, start: usize, end: usize) {
        // Byte offsets over the field text; both are char boundaries because
        // the caller passes ASCII indexes into names it produced.
        self.dialog_selection = Some((
            start.min(self.dialog_field.len()),
            end.min(self.dialog_field.len()),
        ));
    }

    /// Close any open dialog.
    pub fn close_dialog(&mut self) {
        self.dialog = None;
        self.dialog_focus = false;
        self.dialog_field.clear();
        self.dialog_selection = None;
    }
}

/// One tab in the browser window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTab {
    /// The engine's stable id for this tab. It used to be `active + 1` on every
    /// command, so closing a tab before the focused one re-pointed the shell at
    /// a different engine target: frames from one page appeared under another
    /// page's address bar. An id is assigned once and never moves.
    pub engine_tab: u32,
    /// The address shown in the omnibox when this tab is focused.
    pub url: String,
    /// The page title (or the status line until one exists).
    pub title: String,
    /// Whether the tab's last load failed.
    pub error: bool,
    /// Per-tab navigation stack, oldest first.
    pub history: Vec<String>,
    /// Current address inside `history`.
    pub history_index: usize,
}

/// Which browser utility panel is visible below the toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowserPanel {
    /// The current page or new-tab surface.
    #[default]
    Page,
    /// Recent navigation history.
    History,
    /// Saved addresses.
    Bookmarks,
    /// Files reported by the browser broker's download directory.
    Downloads,
    /// Honest state of site permission requests exposed by the engine.
    SitePermissions,
}

/// A file the broker reports in its download directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserDownload {
    /// Final filename inside the broker's private downloads directory.
    pub name: String,
    /// File size in bytes as reported by the filesystem.
    pub size: u64,
}

/// A web search engine the omnibox can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchEngine {
    /// Google — the default.
    Google,
    /// DuckDuckGo.
    DuckDuckGo,
    /// Bing.
    Bing,
}

impl SearchEngine {
    /// Every engine, in settings order.
    pub const ALL: [Self; 3] = [Self::Google, Self::DuckDuckGo, Self::Bing];

    /// The name shown to the user.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Google => "Google",
            Self::DuckDuckGo => "DuckDuckGo",
            Self::Bing => "Bing",
        }
    }

    /// The token this engine is stored under in `settings.conf`.
    ///
    /// A token, not the display title: `title()` is translated and would make a
    /// saved file depend on the interface language it was written in.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::DuckDuckGo => "duckduckgo",
            Self::Bing => "bing",
        }
    }

    /// The engine a stored token names, or `None` for one this build does not
    /// know -- a newer build's choice, which must fall back rather than panic.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|engine| engine.token() == raw)
    }

    /// The search URL for a query.
    #[must_use]
    pub fn search_url(self, query: &str) -> String {
        let q = litric_core::pct::encode(query);
        match self {
            Self::Google => format!("https://www.google.com/search?q={q}"),
            Self::DuckDuckGo => format!("https://duckduckgo.com/?q={q}"),
            Self::Bing => format!("https://www.bing.com/search?q={q}"),
        }
    }

    /// The name shown on the engine chip and in status lines.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Google => "Google",
            Self::DuckDuckGo => "DuckDuckGo",
            Self::Bing => "Bing",
        }
    }
}

/// The Litric Browser window model — the chrome around the engine session.
#[derive(Debug, Clone)]
pub struct BrowserApp {
    /// Tabs in strip order.
    pub tabs: Vec<BrowserTab>,
    /// Focused tab index.
    pub active: usize,
    /// What the user is typing in the omnibox, when it has focus.
    pub omnibox: String,
    /// Whether the omnibox has focus.
    pub omnibox_focus: bool,
    /// Per-tab status/message line (engine errors, load results).
    pub status: String,
    /// Which search engine the omnibox uses.
    pub engine: SearchEngine,
    /// Dimensions of the last engine frame received, if any.
    pub last_frame: Option<(u32, u32)>,
    /// Whether the user asked for the engine session.
    pub wants_engine: bool,
    /// Whether the omnibox was just focused with its whole address
    /// "pre-selected": the next typed character replaces it, exactly the
    /// browser convention of focus-selects-all.
    pub omnibox_fresh: bool,
    /// Recent visited URLs, newest first.
    pub history: VecDeque<String>,
    /// Saved addresses, in the order the user added them.
    pub bookmarks: Vec<String>,
    /// Utility panel below the browser toolbar.
    pub panel: BrowserPanel,
    /// Files the browser broker found in its download folder.
    pub downloads: Vec<BrowserDownload>,
    /// Current download-list request status.
    pub downloads_status: Option<String>,
    /// Whether the active tab has asked the engine for a page and has not
    /// heard back yet.
    ///
    /// This is what makes Stop meaningful: a Stop button that is always
    /// enabled is a button that is usually pressing nothing, and reloading a
    /// page that is still arriving is not a request a person makes. Set when a
    /// navigation is sent, cleared by the tab line or the engine's error that
    /// answers it.
    pub loading: bool,
    /// The next stable engine tab id to hand out. Monotonic: an id is never
    /// reused, so a late frame can never land on a newer tab.
    next_engine_tab: u32,
    /// How far the tab strip is scrolled, in device pixels from the first
    /// tab's left edge.
    ///
    /// Only the view knows the strip's width, so the view clamps this before
    /// laying the strip out. The model deliberately keeps whatever offset it
    /// was given: a model that clamped on its own would need the window width,
    /// and would still hold an offset after the window grew wide enough to
    /// show every tab.
    pub strip_scroll: f32,
    /// Set when the focused tab changed without the pointer, so the view has
    /// to bring it into the visible part of the strip.
    ///
    /// The pointer can only click a tab that is already on screen, so this is
    /// set by the selection methods only: `Ctrl+9` must reveal the tab it
    /// selected, and a bumper click must not be undone by a reveal of the tab
    /// that was already focused.
    pub reveal_active: bool,
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self {
            tabs: vec![BrowserTab {
                engine_tab: 1,
                url: "litric://about".to_owned(),
                title: "New Tab".to_owned(),
                error: false,
                history: vec!["litric://about".to_owned()],
                history_index: 0,
            }],
            active: 0,
            omnibox: String::new(),
            omnibox_focus: false,
            status: String::new(),
            engine: SearchEngine::Google,
            last_frame: None,
            wants_engine: false,
            omnibox_fresh: false,
            history: VecDeque::new(),
            bookmarks: Vec::new(),
            panel: BrowserPanel::Page,
            downloads: Vec::new(),
            downloads_status: None,
            loading: false,
            next_engine_tab: 2,
            strip_scroll: 0.0,
            reveal_active: false,
        }
    }
}

impl BrowserApp {
    /// The focused tab, if any.
    #[must_use]
    pub fn tab(&self) -> Option<&BrowserTab> {
        self.tabs.get(self.active)
    }

    /// The focused tab's stable engine id, if there is a tab.
    #[must_use]
    pub fn active_engine_tab(&self) -> Option<u32> {
        self.tab().map(|tab| tab.engine_tab)
    }

    /// Whether the focused tab has a page the engine has rendered.
    ///
    /// The start page is the shell's own surface: asking the engine to
    /// re-capture it would replace a designed start page with Chromium's
    /// `about:blank`.
    #[must_use]
    pub fn active_tab_has_page(&self) -> bool {
        self.tab().is_some_and(|tab| tab.url != "litric://about")
    }

    /// Select a tab; the omnibox shows its address.
    pub fn select(&mut self, i: usize) {
        if let Some(t) = self.tabs.get(i) {
            // Selecting is the one action that may name a tab the strip has
            // scrolled past, so it is also the one that asks the view to
            // reveal it. Re-selecting the focused tab must not move the strip.
            if i != self.active {
                self.reveal_active = true;
            }
            self.active = i;
            self.omnibox = t.url.clone();
            self.omnibox_focus = true;
            self.panel = BrowserPanel::Page;
        }
    }

    /// Move the strip's scroll by `dx` pixels.
    ///
    /// The view clamps the result against the strip's real width, so this
    /// cannot scroll past the last tab for long enough to be drawn there.
    pub fn scroll_strip_by(&mut self, dx: f32) {
        self.strip_scroll += dx;
        if self.strip_scroll < 0.0 {
            self.strip_scroll = 0.0;
        }
    }

    /// Close a tab; the window survives with the next tab, or a fresh one
    /// when the last goes (Chrome's behaviour).
    pub fn close(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        // Removing a tab renumbers the ones after it, so the focused index has
        // to move with the tab it named. Closing a tab in front of the focused
        // one used to leave `active` pointing at the next tab instead, because
        // the only correction was a clamp at the end of the list.
        let was_active = i == self.active;
        self.tabs.remove(i);
        if self.tabs.is_empty() {
            let engine_tab = self.take_engine_tab();
            self.tabs.push(BrowserTab {
                engine_tab,
                url: "litric://about".to_owned(),
                title: "New Tab".to_owned(),
                error: false,
                history: vec!["litric://about".to_owned()],
                history_index: 0,
            });
            self.active = 0;
            self.omnibox.clear();
            self.omnibox_focus = false;
            self.panel = BrowserPanel::Page;
            // One tab always fits: an offset left from the strip that closed
            // would otherwise hide that tab behind the left bumper.
            self.strip_scroll = 0.0;
            self.reveal_active = true;
            return;
        }
        if was_active {
            self.active = i.min(self.tabs.len().saturating_sub(1));
            self.reveal_active = true;
        } else if i < self.active {
            self.active = self.active.saturating_sub(1);
        }
    }

    /// Hand out the next stable engine tab id.
    fn take_engine_tab(&mut self) -> u32 {
        let id = self.next_engine_tab;
        self.next_engine_tab = self.next_engine_tab.saturating_add(1);
        id
    }

    /// Open a new tab and focus it.
    pub fn new_tab(&mut self) {
        let engine_tab = self.take_engine_tab();
        self.tabs.push(BrowserTab {
            engine_tab,
            url: "litric://about".to_owned(),
            title: "New Tab".to_owned(),
            error: false,
            history: vec!["litric://about".to_owned()],
            history_index: 0,
        });
        self.panel = BrowserPanel::Page;
        self.select(self.tabs.len().saturating_sub(1));
    }

    /// Commit the omnibox: navigate the focused tab. Returns the address.
    ///
    /// The resolution rule is the browser-standard one: something with a
    /// scheme goes as an address, something that looks like a host goes as
    /// `https://`, and everything else is a web search through the configured
    /// engine.
    #[must_use]
    pub fn commit_omnibox(&mut self) -> Option<String> {
        let raw = self.omnibox.trim().to_owned();
        if raw.is_empty() {
            return None;
        }
        // Anything that is meant as an address -- a scheme, or a host-like word
        // -- is validated before it is sent anywhere. Before this, a malformed
        // one was handed to the engine as though it were an address, and a
        // host-like one with a bad port even skipped that. The browser's own
        // parser is the one authority for what an address is (BUG-0035).
        let address = if raw.contains("://") || raw.starts_with("litric://") {
            Some(raw.clone())
        } else if raw.contains('.') && !raw.contains(' ') {
            Some(format!("https://{raw}"))
        } else {
            None
        };
        let url = match address {
            Some(url) => match litric_browser::url::Url::parse(&url) {
                // Store the parser's canonical text, not what was typed. The
                // host is lowercased, the scheme's default port is dropped and
                // an empty path becomes `/`, so three spellings of one page are
                // one history entry, one bookmark and one Back step (BUG-0036).
                Ok(parsed) => parsed.to_url(),
                Err(error) => {
                    self.status = format!("that address is not valid: {error}");
                    if let Some(tab) = self.tabs.get_mut(self.active) {
                        tab.error = true;
                    }
                    return None;
                }
            },
            // A search URL is built by this file and its query is encoded, so
            // it is not parsed here: it is not an address and has no host to
            // validate.
            None => self.engine.search_url(&raw),
        };
        self.navigate_active_to(&url);
        Some(url)
    }

    /// The canonical spelling of an address for storage.
    ///
    /// The parser is the one authority for what an address is, and its text
    /// form is what history and bookmarks are keyed on. Anything the parser
    /// does not know -- a search URL built by this file, a test token -- is
    /// returned unchanged, because it is not an address and has no canonical
    /// form to adopt.
    #[must_use]
    pub fn canonical_address(input: &str) -> String {
        litric_browser::url::Url::parse(input).map_or_else(|_| input.to_owned(), |u| u.to_url())
    }

    /// Navigate the selected tab and append a real history entry.
    pub fn navigate_active_to(&mut self, url: &str) {
        // Every writer of a stored address goes through here, so normalising
        // at this point keeps the invariant even for callers that pass an
        // older, non-canonical bookmark: `https://EXAMPLE.com` and
        // `https://example.com:443` are the same key as `https://example.com/`.
        let url = Self::canonical_address(url);
        if let Some(tab) = self.tabs.get_mut(self.active) {
            if tab
                .history
                .get(tab.history_index)
                .is_none_or(|current| current != &url)
            {
                tab.history.truncate(tab.history_index.saturating_add(1));
                tab.history.push(url.clone());
                if tab.history.len() > 100 {
                    tab.history.remove(0);
                }
                tab.history_index = tab.history.len().saturating_sub(1);
            }
            tab.url = url.clone();
            tab.error = false;
            tab.title = url.clone();
        }
        self.record_visit(&url);
        self.omnibox_focus = false;
        self.panel = BrowserPanel::Page;
    }

    /// Record a URL in the bounded, cross-tab recent list.
    pub fn record_visit(&mut self, url: &str) {
        if url == "litric://about" {
            return;
        }
        self.history.retain(|entry| entry != url);
        self.history.push_front(url.to_owned());
        while self.history.len() > 200 {
            self.history.pop_back();
        }
    }

    /// Move the active tab back one address and return the URL for the engine.
    pub fn back(&mut self) -> Option<String> {
        let tab = self.tabs.get_mut(self.active)?;
        if tab.history_index == 0 {
            return None;
        }
        tab.history_index = tab.history_index.saturating_sub(1);
        tab.url = tab.history.get(tab.history_index)?.clone();
        tab.error = false;
        self.omnibox = tab.url.clone();
        self.panel = BrowserPanel::Page;
        Some(tab.url.clone())
    }

    /// Move the active tab forward one address and return the URL for the engine.
    pub fn forward(&mut self) -> Option<String> {
        let tab = self.tabs.get_mut(self.active)?;
        let next = tab.history_index.saturating_add(1);
        if next >= tab.history.len() {
            return None;
        }
        tab.history_index = next;
        tab.url = tab.history.get(next)?.clone();
        tab.error = false;
        self.omnibox = tab.url.clone();
        self.panel = BrowserPanel::Page;
        Some(tab.url.clone())
    }

    /// Whether the active tab can go back.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.tab().is_some_and(|tab| tab.history_index > 0)
    }

    /// Whether a load is in flight, so Stop would stop something.
    #[must_use]
    pub fn can_stop(&self) -> bool {
        self.loading
    }

    /// Whether Reload would reload a page the engine is drawing.
    ///
    /// The start page is the shell's own surface, and `about:blank` has nothing
    /// to reload; asking the engine for either produced a fresh `about:blank`
    /// frame over a designed page.
    #[must_use]
    pub fn can_reload(&self) -> bool {
        self.tab()
            .is_some_and(|tab| tab.url != "litric://about" && tab.url != "about:blank")
    }

    /// Whether the active tab can go forward.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        self.tab()
            .is_some_and(|tab| tab.history_index.saturating_add(1) < tab.history.len())
    }

    /// Toggle the active address in the bookmarks list.
    pub fn toggle_bookmark(&mut self) -> bool {
        let Some(url) = self.tab().map(|tab| tab.url.clone()) else {
            return false;
        };
        if let Some(index) = self.bookmarks.iter().position(|saved| saved == &url) {
            self.bookmarks.remove(index);
            false
        } else {
            self.bookmarks.push(url);
            true
        }
    }

    /// Whether the current URL is saved.
    #[must_use]
    pub fn is_bookmarked(&self) -> bool {
        self.tab()
            .is_some_and(|tab| self.bookmarks.iter().any(|saved| saved == &tab.url))
    }

    /// Navigate the focused tab to its own URL (the reload / engine-start
    /// path). Returns the address to send, if the tab exists.
    #[must_use]
    pub fn navigate_active(&mut self) -> Option<String> {
        self.navigate_tab(self.active)
    }

    /// The URL of tab `i`, with its error cleared as a reload does.
    ///
    /// A reload from a context menu belongs to the tab the menu was opened for,
    /// which is not necessarily the focused one.
    pub fn navigate_tab(&mut self, i: usize) -> Option<String> {
        let url = self.tabs.get(i)?.url.clone();
        if let Some(t) = self.tabs.get_mut(i) {
            t.error = false;
        }
        Some(url)
    }
}

/// One file the Defender queued, scanned or flagged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatRow {
    /// The logical path scanned.
    pub path: String,
    /// The verdict's summary line for the user.
    pub verdict: String,
    /// Whether the verdict was a detection.
    pub detected: bool,
}

/// The state of a Defender scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanState {
    /// Nothing has been asked for.
    #[default]
    Idle,
    /// A scan is running; the counts are (done, total).
    Scanning {
        /// Files scanned so far.
        done: usize,
        /// Files found to scan, when known.
        total: usize,
    },
    /// The last scan finished.
    Done {
        /// Files scanned.
        scanned: usize,
        /// Detections.
        threats: usize,
    },
}

/// How much of the home folder a Defender scan covers.
///
/// The two modes are a promise about coverage, not a speed knob: a fast scan
/// walks the folders a person keeps documents in, a deep scan walks the whole
/// home. The window names the mode it is running, so "no threats found" never
/// has to be qualified by the user guessing what was actually looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanMode {
    /// The folders people keep documents in: quick, bounded, run often.
    #[default]
    Fast,
    /// Every folder in the home directory: slower, for a deliberate check.
    Deep,
}

impl ScanMode {
    /// Both modes, in the order the window offers them.
    pub const ALL: [Self; 2] = [Self::Fast, Self::Deep];

    /// The button label and the word the result sentence uses.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Fast => "Fast scan",
            Self::Deep => "Deep scan",
        }
    }

    /// Where the scan starts, as logical paths inside the file service.
    #[must_use]
    pub fn roots(self) -> &'static [&'static str] {
        match self {
            Self::Fast => &[
                "/Documents",
                "/Downloads",
                "/Pictures",
                "/Music",
                "/Videos",
                "/Projects",
            ],
            Self::Deep => &["/"],
        }
    }

    /// How many folder levels below a root the walk descends.
    #[must_use]
    pub fn depth(self) -> usize {
        match self {
            Self::Fast => 8,
            Self::Deep => 24,
        }
    }

    /// The largest number of files one run will queue.
    ///
    /// The queue is a list of paths held while the worker runs, so this is a
    /// memory ceiling as much as a time one: 2,000 short paths in a fast scan
    /// and 40,000 in a deep scan stay inside the daemon's own budget, and the
    /// result sentence reports the entries the cap refused.
    #[must_use]
    pub fn file_cap(self) -> usize {
        match self {
            Self::Fast => 2_000,
            Self::Deep => 40_000,
        }
    }

    /// What this mode covers, in the words the window shows beside it.
    #[must_use]
    pub fn coverage(self) -> &'static str {
        match self {
            Self::Fast => {
                "Documents, Downloads, Pictures, Music, Videos and Projects. Finishes quickly."
            }
            Self::Deep => {
                "Every folder in your home, 24 levels deep. Takes longer; run it when you have time."
            }
        }
    }
}

/// The Defender window model. Scanning itself happens in
/// [`crate::defender`]; this holds what the window shows.
#[derive(Debug, Clone, Default)]
pub struct DefenderApp {
    /// The state of the current or last scan.
    pub state: ScanState,
    /// How much of the home folder the current or last scan covers.
    pub mode: ScanMode,
    /// One row per scanned file that matters (detections always; a bounded
    /// sample of clean files while scanning).
    pub rows: Vec<ThreatRow>,
    /// What the status line says.
    pub message: String,
    /// Definitions loaded, for the status line.
    pub definitions: usize,
    /// Whether the last scan found anything (drives the top-bar badge).
    pub last_had_threats: bool,
    /// What is actually contained right now, read from the records on disk
    /// when the window opens and after every move.
    pub quarantined: Vec<crate::quarantine::Quarantined>,
}

impl DefenderApp {
    /// Begin a scan in a chosen mode: clears the last scan's rows and records
    /// the coverage the window will claim when the run finishes.
    pub fn begin(&mut self, mode: ScanMode, total: usize) {
        self.mode = mode;
        self.start(total);
    }

    /// Begin a scan: clears the last scan's rows.
    pub fn start(&mut self, total: usize) {
        self.state = ScanState::Scanning { done: 0, total };
        self.rows.clear();
        self.message = "Scanning…".to_owned();
        self.last_had_threats = false;
    }

    /// Record one scanned file. `limit_clean` bounds how many clean rows the
    /// window keeps so a big scan cannot grow the model without end.
    pub fn record(&mut self, path: &str, decision: &Decision) {
        // The row speaks the verdict a person needs: whether the file is safe,
        // whether it is a threat, or whether it was not looked at. The tier and
        // the signature id stay in the scan report for the audit trail; they are
        // not what the window is for.
        let (detected, verdict) = match decision {
            Decision::Allow => (false, "Safe".to_owned()),
            Decision::AllowUnknown { .. } => (false, "Not checked".to_owned()),
            Decision::Deny { .. } => (true, "Threat found".to_owned()),
        };
        if detected {
            self.last_had_threats = true;
            self.rows.push(ThreatRow {
                path: path.to_owned(),
                verdict,
                detected: true,
            });
        } else if self.rows.len() < 200 {
            // Keep a sample of clean rows so an in-progress window is not
            // empty, but never let a big scan grow the model unbounded.
            self.rows.push(ThreatRow {
                path: path.to_owned(),
                verdict,
                detected: false,
            });
        }
        if let ScanState::Scanning { done, total } = &mut self.state {
            *done = done.saturating_add(1);
            if *done > *total {
                *total = *done;
            }
        }
    }

    /// The scan ended.
    pub fn finish(&mut self) {
        let (scanned, threats) = match self.state {
            ScanState::Scanning { done, .. } => {
                (done, self.rows.iter().filter(|r| r.detected).count())
            }
            ScanState::Done { scanned, threats } => (scanned, threats),
            ScanState::Idle => (0, 0),
        };
        self.state = ScanState::Done { scanned, threats };
        self.message = if threats > 0 {
            format!(
                "{threats} threat{} found in {scanned} files",
                if threats == 1 { "" } else { "s" }
            )
        } else {
            format!("No threats found in {scanned} files")
        };
    }
}

/// One icon on the desktop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopIcon {
    /// The label under the icon.
    pub label: String,
    /// What opening it does.
    pub target: IconTarget,
}

/// What opening a desktop icon does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconTarget {
    /// Open a built-in application.
    App(crate::shortcuts::App),
    /// Open Files at a folder — the "shortcut to another app's folder" case.
    Folder(String),
}

/// The desktop's icon grid. Icons live in a left-to-right, top-to-bottom
/// column-major grid like every desktop; positions are derived from the
/// index so the model never stores coordinates.
#[derive(Debug, Clone, Default)]
pub struct DesktopIcons {
    /// The icons, in grid order.
    pub items: Vec<DesktopIcon>,
    /// Which icon is selected, if any.
    pub selected: Option<usize>,
}

impl DesktopIcons {
    /// The clean first-boot desktop: four useful destinations, not a copy of
    /// every application that is already available from the dock and launcher.
    #[must_use]
    pub fn minimal() -> Self {
        use crate::shortcuts::App;
        Self {
            items: vec![
                DesktopIcon {
                    label: "Files".to_owned(),
                    target: IconTarget::App(App::Files),
                },
                DesktopIcon {
                    label: "Browser".to_owned(),
                    target: IconTarget::App(App::Browser),
                },
                DesktopIcon {
                    label: "Documents".to_owned(),
                    target: IconTarget::Folder("/Documents".to_owned()),
                },
                DesktopIcon {
                    label: "Downloads".to_owned(),
                    target: IconTarget::Folder("/Downloads".to_owned()),
                },
            ],
            selected: None,
        }
    }

    /// The desktop every machine starts with: the working apps and shortcuts
    /// to the folders a file manager would show.
    #[must_use]
    pub fn standard() -> Self {
        use crate::shortcuts::App;
        Self {
            items: vec![
                DesktopIcon {
                    label: "Files".to_owned(),
                    target: IconTarget::App(App::Files),
                },
                DesktopIcon {
                    label: "Browser".to_owned(),
                    target: IconTarget::App(App::Browser),
                },
                DesktopIcon {
                    label: "Terminal".to_owned(),
                    target: IconTarget::App(App::Terminal),
                },
                DesktopIcon {
                    label: "Defender".to_owned(),
                    target: IconTarget::App(App::Defender),
                },
                DesktopIcon {
                    label: "Settings".to_owned(),
                    target: IconTarget::App(App::Settings),
                },
                DesktopIcon {
                    label: "Trash".to_owned(),
                    target: IconTarget::App(App::Trash),
                },
                DesktopIcon {
                    label: "Documents".to_owned(),
                    target: IconTarget::Folder("/Documents".to_owned()),
                },
                DesktopIcon {
                    label: "Downloads".to_owned(),
                    target: IconTarget::Folder("/Downloads".to_owned()),
                },
                DesktopIcon {
                    label: "Pictures".to_owned(),
                    target: IconTarget::Folder("/Pictures".to_owned()),
                },
                DesktopIcon {
                    label: "Projects".to_owned(),
                    target: IconTarget::Folder("/Projects".to_owned()),
                },
            ],
            selected: None,
        }
    }

    /// Select an icon, or clear the selection with `None`.
    pub fn select(&mut self, i: Option<usize>) {
        self.selected = if let Some(idx) = i {
            if idx < self.items.len() {
                Some(idx)
            } else {
                None
            }
        } else {
            None
        };
    }
}

/// Hotspots the Files window registers.
///
/// Wrapped by [`crate::shell::Hot::Files`] so the shell's single hotspot
/// registry stays one type; the view and the input handler agree on these
/// without a second list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilesHot {
    /// A place in the navigation pane.
    Place(usize),
    /// The navigation pane itself, behind its rows: the wheel over the pane
    /// scrolls the list even where it is empty.
    PlacesScroll,
    /// A breadcrumb segment.
    Crumb(usize),
    /// The root crumb. Always drawn, always a target, so a deep path has a
    /// way home that does not depend on the window being wide enough to show
    /// the first segment.
    Root,
    /// A column header (sort toggle).
    Header(SortKey),
    /// A row in the table.
    Row(usize),
    /// Toggle a row in the multi-selection.
    ToggleRow(usize),
    /// The Back button.
    Back,
    /// The Forward button.
    Forward,
    /// The Up button.
    Up,
    /// The details-pane toggle.
    Details,
    /// The New button (opens the create menu as a context menu).
    New,
    /// The search / filter field.
    Filter,
    /// The paste button.
    Paste,
    /// Close the text viewer.
    ViewerClose,
    /// Scroll the viewer up a page.
    ViewerUp,
    /// Scroll the viewer down a page.
    ViewerDown,
    /// The compress (ZIP) button.
    Compress,
    /// The extract button.
    Extract,
    /// Re-read the current folder from disk.
    Refresh,
    /// The empty state's way out: open the new-folder dialog.
    EmptyNewFolder,
}

/// Map a Files hotspot onto the model. Navigation returns the path to load,
/// or `None` when the action only changed local state.
#[must_use]
pub fn files_action(app: &mut FilesApp, hot: FilesHot) -> Option<String> {
    match hot {
        FilesHot::EmptyNewFolder => {
            app.ask_new_folder();
            None
        }
        FilesHot::PlacesScroll => None,
        FilesHot::Place(i) => {
            let path = app.places.get(i).map(|(p, _)| p.clone())?;
            app.place_index = Some(i);
            app.navigate(&path);
            Some(app.path.clone())
        }
        FilesHot::Crumb(i) => {
            let path = app.breadcrumb_path(i);
            app.navigate(&path);
            Some(app.path.clone())
        }
        FilesHot::Root => {
            app.navigate("/");
            Some(app.path.clone())
        }
        FilesHot::Header(k) => {
            app.toggle_sort(k);
            None
        }
        FilesHot::Row(i) => {
            app.select_row(i);
            None
        }
        FilesHot::ToggleRow(i) => {
            app.toggle_row(i);
            None
        }
        FilesHot::Back => {
            app.go_back();
            Some(app.path.clone())
        }
        FilesHot::Forward => {
            app.go_forward();
            Some(app.path.clone())
        }
        FilesHot::Up => {
            app.go_up();
            Some(app.path.clone())
        }
        FilesHot::Details => {
            app.details = !app.details;
            None
        }
        FilesHot::ViewerClose => {
            app.viewer = None;
            None
        }
        FilesHot::ViewerUp => {
            if let Some(v) = &mut app.viewer {
                v.scroll = v.scroll.saturating_sub(10);
            }
            None
        }
        FilesHot::ViewerDown => {
            if let Some(v) = &mut app.viewer {
                v.scroll = v.scroll.saturating_add(10);
            }
            None
        }
        FilesHot::Compress => {
            let paths = app.selected_paths();
            if let Some(p) = paths.first() {
                let name = if paths.len() == 1 {
                    format!("{}.zip", p.rsplit('/').next().unwrap_or("archive"))
                } else {
                    "Archive.zip".to_owned()
                };
                app.ask_confirm(
                    "Compress",
                    &format!("Archive {} selected items as {name}?", paths.len()),
                    if paths.len() == 1 {
                        FilesOp::Compress(name)
                    } else {
                        FilesOp::CompressMany {
                            name,
                            sources: paths,
                        }
                    },
                );
            }
            None
        }
        FilesHot::Extract => {
            if let Some(p) = app.selected_path() {
                if p.ends_with(".zip")
                    || p.ends_with(".tar")
                    || p.ends_with(".tgz")
                    || p.ends_with(".tar.gz")
                {
                    app.ask_confirm(
                        "Extract",
                        &format!("Extract {p} into a folder beside it?"),
                        FilesOp::Extract(p),
                    );
                } else {
                    app.dialog = Some(DialogKind::Error(
                        "Not an archive this system reads (.zip, .tar, .tgz)".to_owned(),
                    ));
                }
            }
            None
        }
        FilesHot::New | FilesHot::Filter | FilesHot::Paste => None,
        // The shell owns Refresh: it needs the filesystem bridge, not the model.
        FilesHot::Refresh => None,
    }
}

/// Map a Browser hotspot (`Hot` is the shell's hotspot type; browser hot
/// variants ride on the same enum). Returns the address to load, if any.
///
/// The wiring here is deliberately total: every browser hotspot either names
/// a tab action or a load, so a hotspot can never be registered that does
/// nothing.
#[must_use]
pub fn browser_action(app: &mut BrowserApp, hot: Hot) -> Option<String> {
    match hot {
        Hot::BrowserNewTab => {
            app.new_tab();
            None
        }
        Hot::BrowserTab(i) => {
            app.select(i);
            None
        }
        Hot::BrowserCloseTab(i) => {
            app.close(i);
            None
        }
        Hot::BrowserGo => app.commit_omnibox(),
        _ => None,
    }
}

/// One row of a context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    /// A headline over the rows that follow.
    Section(&'static str),
    /// A clickable action with its label.
    Item(&'static str, MenuAction),
    /// A thin rule.
    Separator,
}

/// What a menu row does when clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Create a folder (Files / desktop).
    NewFolder,
    /// Create an empty text file (Files).
    NewFile,
    /// Paste from the Files clipboard.
    Paste,
    /// Rename the selection (Files).
    Rename,
    /// Copy the selection (Files).
    Copy,
    /// Cut the selection (Files).
    Cut,
    /// Move the selection to the trash (Files).
    Trash,
    /// Compress the selection (Files).
    Compress,
    /// Extract the selection (Files).
    Extract,
    /// Sort by name (Files).
    SortName,
    /// Sort by size (Files).
    SortSize,
    /// Sort by modified time (Files).
    SortModified,
    /// Toggle the details pane (Files).
    ToggleDetails,
    /// Scan the folders people keep documents in.
    ScanNow,
    /// Scan every folder in the home directory.
    DeepScan,
    /// Stop the scan that is running.
    CancelScan,
    /// Contain one detected file.
    QuarantineFinding(usize),
    /// Put one contained file back where it came from.
    RestoreQuarantined(usize),
    /// Check one contained file against its recorded digest.
    VerifyQuarantined(usize),
    /// Start the browser engine session.
    BrowserStart,
    /// Reload the focused browser tab.
    BrowserReload,
    /// Close the browser tab under the pointer.
    BrowserCloseTab,
    /// Open the display settings.
    DisplaySettings,
    /// Open the Trash application.
    OpenTrash,
    /// Empty the trash (Trash window).
    EmptyTrash,
    /// Restore a trash entry (Trash window).
    Restore,
    /// Permanently delete the selected Trash entry after confirmation.
    PurgeTrashItem,
    /// Open the selected desktop icon.
    OpenSelection,
    /// Ask to remove the launcher tile's installed application.
    UninstallApp,
    /// Put the installed application on the dock.
    PinDesktopApp,
    /// Take the installed application off the dock.
    UnpinDesktopApp,
    /// Change the wallpaper (desktop). Cycles themes.
    NextWallpaper,
    /// Use Google for omnibox searches.
    EngineGoogle,
    /// Use DuckDuckGo for omnibox searches.
    EngineDuckDuckGo,
    /// Use Bing for omnibox searches.
    EngineBing,
}

/// An open context menu. Positions are in screen coordinates; the menu is
/// drawn by the view and hit through the ordinary hotspot table, so a menu is
/// just another surface the pointer already knows how to use.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenu {
    /// Where the pointer was when the button went down.
    pub at: (f32, f32),
    /// Which surface the click landed on.
    pub surface: MenuSurface,
    /// The browser tab the pointer was over, when the surface is a tab.
    ///
    /// A row menu belongs to the thing that was right-clicked. Chrome does not
    /// switch tabs on a right-click, so the tab cannot be read back from
    /// `BrowserApp::active` when the row runs: it has to be recorded here,
    /// while the pointer still says which tab it was.
    pub tab: Option<usize>,
    /// The installed application a launcher-tile menu belongs to.
    ///
    /// The menu row runs after the menu has closed, when the launcher may have
    /// reloaded and the tile's index no longer names the same application. The
    /// id is captured while the pointer is still over the tile, just as `tab`
    /// is for a browser tab.
    pub app_id: Option<String>,
    /// The rows, in order.
    pub items: Vec<MenuItem>,
}

/// What a right-click landed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSurface {
    /// The wallpaper.
    Desktop,
    /// The Files window's table or places pane.
    Files,
    /// The Files window's selected row.
    FilesRow,
    /// The Browser window's page area.
    Browser,
    /// A browser tab.
    BrowserTab,
    /// The Trash window's list.
    Trash,
    /// An installed application's launcher tile.
    LauncherTile,
}

impl ContextMenu {
    /// The menu for a surface, at a point. The rows are the standard set for
    /// that surface — Explorer for files, Chrome for the browser — drawn from
    /// the shell's real actions, so a menu item can never exist that the
    /// shell cannot perform.
    #[must_use]
    pub fn for_surface(surface: MenuSurface, at: (f32, f32)) -> Self {
        let items = match surface {
            MenuSurface::Desktop => vec![
                MenuItem::Item("New folder", MenuAction::NewFolder),
                MenuItem::Separator,
                MenuItem::Item("Fast scan", MenuAction::ScanNow),
                MenuItem::Item("Deep scan", MenuAction::DeepScan),
                MenuItem::Item("Open Trash", MenuAction::OpenTrash),
                MenuItem::Separator,
                MenuItem::Item("Next wallpaper", MenuAction::NextWallpaper),
                MenuItem::Item("Display settings", MenuAction::DisplaySettings),
            ],
            MenuSurface::Files => vec![
                MenuItem::Item("New folder", MenuAction::NewFolder),
                MenuItem::Item("New text file", MenuAction::NewFile),
                MenuItem::Item("Paste", MenuAction::Paste),
                MenuItem::Separator,
                MenuItem::Section("Sort by"),
                MenuItem::Item("Name", MenuAction::SortName),
                MenuItem::Item("Size", MenuAction::SortSize),
                MenuItem::Item("Date modified", MenuAction::SortModified),
                MenuItem::Separator,
                MenuItem::Item("Details pane", MenuAction::ToggleDetails),
            ],
            MenuSurface::FilesRow => vec![
                MenuItem::Item("Open", MenuAction::OpenSelection),
                MenuItem::Separator,
                MenuItem::Item("Cut", MenuAction::Cut),
                MenuItem::Item("Copy", MenuAction::Copy),
                MenuItem::Item("Rename", MenuAction::Rename),
                MenuItem::Item("Move to trash", MenuAction::Trash),
                MenuItem::Separator,
                MenuItem::Item("Compress to ZIP", MenuAction::Compress),
                MenuItem::Item("Extract here", MenuAction::Extract),
            ],
            MenuSurface::Browser => vec![
                MenuItem::Item("Start engine session", MenuAction::BrowserStart),
                MenuItem::Item("Reload", MenuAction::BrowserReload),
            ],
            MenuSurface::BrowserTab => vec![
                MenuItem::Item("Reload", MenuAction::BrowserReload),
                MenuItem::Item("Close tab", MenuAction::BrowserCloseTab),
            ],
            MenuSurface::Trash => vec![
                MenuItem::Item("Restore", MenuAction::Restore),
                MenuItem::Item("Delete permanently", MenuAction::PurgeTrashItem),
                MenuItem::Item("Empty trash", MenuAction::EmptyTrash),
            ],
            MenuSurface::LauncherTile => vec![
                MenuItem::Section("Application"),
                MenuItem::Item("Pin to dock", MenuAction::PinDesktopApp),
                MenuItem::Item("Uninstall application…", MenuAction::UninstallApp),
            ],
        };
        Self {
            at,
            surface,
            tab: None,
            app_id: None,
            items,
        }
    }

    /// The menu for an installed application's launcher tile.
    ///
    /// `pinned` decides which dock row is offered. The id is captured here,
    /// while the pointer is still over the tile, because the row runs after
    /// the menu closes and discovery may have changed the tile indices.
    #[must_use]
    pub fn for_launcher(app_id: &str, pinned: bool, at: (f32, f32)) -> Self {
        let mut menu = Self::for_surface(MenuSurface::LauncherTile, at);
        menu.app_id = Some(app_id.to_owned());
        menu.items = vec![
            MenuItem::Section("Application"),
            MenuItem::Item(
                if pinned {
                    "Unpin from dock"
                } else {
                    "Pin to dock"
                },
                if pinned {
                    MenuAction::UnpinDesktopApp
                } else {
                    MenuAction::PinDesktopApp
                },
            ),
            MenuItem::Item("Uninstall application…", MenuAction::UninstallApp),
        ];
        menu
    }

    /// Point this menu at the tab it was opened for.
    #[must_use]
    pub fn for_tab(mut self, i: usize) -> Self {
        self.tab = Some(i);
        self
    }

    /// The height the menu will draw at, one row each plus padding.
    #[must_use]
    pub fn height(&self, row_h: f32) -> f32 {
        let rows = self
            .items
            .iter()
            .filter(|i| !matches!(i, MenuItem::Separator))
            .count() as f32;
        let seps = self.items.len() as f32 - rows;
        rows * row_h + seps * row_h * 0.5 + row_h * 0.5
    }
}

/// The Trash application model: what the window shows.
#[derive(Debug, Clone, Default)]
pub struct TrashApp {
    /// Entries as last listed: (name, deleted ISO date, restore dir, is dir).
    pub entries: Vec<(String, String, String, bool)>,
    /// Which entry is selected.
    pub selected: Option<usize>,
    /// The last listing error, if any.
    pub error: Option<String>,
    /// Whether the listing is stale against the disk.
    pub listed: bool,
    /// Case-insensitive search within the live trash listing.
    pub filter: String,
    /// A destructive action that still needs an explicit second confirmation.
    pub confirm: Option<TrashConfirmation>,
}

/// The pending destructive Trash action shown in the confirmation row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrashConfirmation {
    /// Permanently remove every item currently in Trash.
    Empty,
    /// Permanently remove one named item.
    Purge(String),
}

impl TrashApp {
    /// The selected entry's name.
    #[must_use]
    pub fn selected_name(&self) -> Option<String> {
        self.selected
            .and_then(|i| self.entries.get(i).map(|e| e.0.clone()))
    }

    /// Total items.
    #[must_use]
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Indices into `entries` that match the current search, in listing order.
    #[must_use]
    pub fn visible_indices(&self) -> Vec<usize> {
        let needle = self.filter.trim().to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, (name, _, origin, _))| {
                needle.is_empty()
                    || name.to_lowercase().contains(&needle)
                    || origin.to_lowercase().contains(&needle)
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// Whether the selected item is among the visible results.
    #[must_use]
    pub fn has_visible_selection(&self) -> bool {
        self.selected
            .is_some_and(|selected| self.visible_indices().contains(&selected))
    }
}

/// Keyboard input for a dialog's name field, driven by the real
/// [`crate::textedit::TextEdit`] editor so caret and selection semantics stay
/// identical everywhere a field exists.
#[must_use]
pub fn dialog_field_key(
    field: &mut String,
    selection: &mut Option<(usize, usize)>,
    key: litric_input::keys::Key,
    mods: litric_input::keys::Modifiers,
) -> crate::textedit::EditOutcome {
    let mut editor = crate::textedit::TextEdit::new();
    editor.set_text(field);
    if let Some((s, e)) = *selection {
        editor.select_range(s, e);
    }
    let outcome = editor.handle_key(key, mods);
    if matches!(
        outcome,
        crate::textedit::EditOutcome::Changed | crate::textedit::EditOutcome::Moved
    ) {
        *field = editor.text().to_owned();
        *selection = editor.selection();
    }
    outcome
}

// ---------------------------------------------------------------------------
// Terminal
// ---------------------------------------------------------------------------

/// Which xterm mouse tracking mode a program enabled.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MouseMode {
    /// `?1000` off; the shell owns the pointer.
    #[default]
    Off,
    /// `?1000`: press and release, no motion.
    Normal,
    /// `?1002`: press/release plus motion while a button is held.
    Button,
    /// `?1003`: every pointer move, held or not.
    Any,
}

/// One button in a terminal mouse report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Primary button.
    Left,
    /// Middle button.
    Middle,
    /// Secondary button.
    Right,
    /// Wheel up (button 64).
    WheelUp,
    /// Wheel down (button 65).
    WheelDown,
}

/// What happened to a terminal mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    /// A button went down, or a wheel notch arrived.
    Press,
    /// A button came up.
    Release,
    /// The pointer moved without a button changing.
    Motion,
}

impl MouseButton {
    const fn code(self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Middle => 1,
            Self::Right => 2,
            Self::WheelUp => 64,
            Self::WheelDown => 65,
        }
    }

    const fn held_bit(self) -> u8 {
        match self {
            Self::Left => 0b001,
            Self::Middle => 0b010,
            Self::Right => 0b100,
            Self::WheelUp | Self::WheelDown => 0,
        }
    }

    const fn is_wheel(self) -> bool {
        matches!(self, Self::WheelUp | Self::WheelDown)
    }
}

/// The Terminal application model: a command line the guest OS answers.
///
/// Commands run for real in a persistent `/bin/sh` attached to a PTY. The
/// shell owns the process side, because the process side is the part that
/// must never block the frame loop; this model owns the visible scrollback,
/// the current line, and enough control-sequence handling that ordinary
/// colour and progress output does not turn into punctuation soup.
#[derive(Debug, Clone, Default)]
pub struct TerminalApp {
    /// Output lines, oldest first, bounded at [`TerminalApp::MAX_LINES`].
    pub lines: VecDeque<String>,
    /// The command being typed.
    pub input: String,
    /// Whether the input field holds the keyboard.
    pub focused: bool,
    /// A notice above the prompt: platform refusals, hints.
    pub note: Option<String>,
    /// A command is running and its output is still arriving.
    pub busy: bool,
    /// Most recent distinct commands, oldest first, bounded for the life of
    /// this shell session.
    pub history: VecDeque<String>,
    /// The current history navigation position, if Up has been pressed.
    pub history_cursor: Option<usize>,
    /// The in-progress input saved when the user first navigates backward.
    pub history_draft: String,
    /// The line being assembled from a byte stream that may not have ended
    /// with a newline yet.
    partial: String,
    /// Where the stream is inside an ANSI escape sequence. The state is kept
    /// across reads because a terminal can split an escape anywhere.
    escape: Escape,
    /// A carriage return was seen. If the next byte is a newline this is a
    /// normal CRLF; otherwise the next printable byte overwrites the line,
    /// which is how progress meters work.
    carriage_return: bool,
    /// The alternate-screen grid, when a full-screen program is active. The
    /// primary scrollback in `lines` is kept untouched and restored when the
    /// program leaves the alternate screen.
    alternate: Option<Screen>,
    /// The SGR state applied to subsequently written alternate-screen cells.
    /// It is reset on entry to and exit from the alternate screen and by
    /// `SGR 0`, so attributes cannot leak through a screen transition.
    sgr: CellStyle,
    /// Width the UI last offered to the PTY, used when a program enters the
    /// alternate screen before the next resize notification.
    screen_cols: usize,
    /// Height the UI last offered to the PTY.
    screen_rows: usize,
    /// Whether the program wants its cursor shown (`CSI ?25h/l`).
    cursor_visible: bool,
    /// The xterm mouse tracking flags a program enabled. They are separate
    /// flags because `?1000h` and `?1002h` can overlap; the highest enabled
    /// mode decides how motion is filtered.
    mouse_normal: bool,
    mouse_button: bool,
    mouse_any: bool,
    /// `?1006`: report coordinates and release in SGR form instead of X10.
    mouse_sgr: bool,
    /// The buttons currently held, so `?1002` motion can name one.
    mouse_held: u8,
    /// `?2004`: wrap pasted text in the bracketed-paste markers.
    bracketed_paste: bool,
    /// Parameters collected inside a CSI sequence.
    csi: String,
    /// The CSI began with `?`, so it is a private mode rather than a command.
    csi_private: bool,
    /// Cursor stored by `CSI s` on the alternate grid.
    saved_cursor: Option<(usize, usize)>,
}

/// Where an ANSI escape sequence is in the byte stream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Escape {
    #[default]
    None,
    /// ESC was seen and the next byte chooses the sequence kind.
    Esc,
    /// Inside a CSI sequence (`ESC [ ... final`).
    Csi,
    /// Inside an OSC sequence (`ESC ] ... BEL` or `ESC \`).
    Osc,
    /// OSC saw ESC; a following backslash terminates it.
    OscEsc,
}

impl TerminalApp {
    /// The scrollback ceiling. A `yes` loop fills this in a moment and
    /// then simply pushes the old lines out — the terminal never grows
    /// without bound because a process misbehaved.
    pub const MAX_LINES: usize = 400;
    /// Maximum command history kept in memory.
    pub const MAX_HISTORY: usize = 100;

    /// A fresh terminal with a greeting.
    #[must_use]
    pub fn new() -> Self {
        let mut lines = VecDeque::new();
        lines.push_back("LitricOS Terminal — พิมพ์คำสั่ง Linux แล้วกด Enter".to_owned());
        lines.push_back(String::new());
        Self {
            lines,
            input: String::new(),
            focused: false,
            note: None,
            busy: false,
            history: VecDeque::new(),
            history_cursor: None,
            history_draft: String::new(),
            partial: String::new(),
            escape: Escape::None,
            carriage_return: false,
            alternate: None,
            sgr: CellStyle::default(),
            screen_cols: 100,
            screen_rows: 30,
            cursor_visible: true,
            mouse_normal: false,
            mouse_button: false,
            mouse_any: false,
            mouse_sgr: false,
            mouse_held: 0,
            bracketed_paste: false,
            csi: String::new(),
            csi_private: false,
            saved_cursor: None,
        }
    }

    /// Append one output line, dropping the oldest past the ceiling.
    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push_back(line.into());
        while self.lines.len() > Self::MAX_LINES {
            self.lines.pop_front();
        }
    }

    /// Feed raw PTY output into the active screen.
    ///
    /// Ordinary output still goes to the bounded primary scrollback. When the
    /// program enters the alternate screen (`CSI ?1049h`, plus the older
    /// `?47h`/`?1047h`), output goes to a cell grid instead: cursor addressing,
    /// clear and erase then affect what the program expects, and the primary
    /// scrollback is restored unchanged on `CSI ?1049l`. SGR colour and
    /// attributes are stored per cell; OSC titles/belts are accepted and
    /// ignored. Mouse reporting and bracketed paste are parsed here; the shell
    /// turns pointer and paste events into bytes through the methods below.
    /// Sixel is still ignored.
    pub fn push_output(&mut self, data: &[u8]) {
        for ch in String::from_utf8_lossy(data).chars() {
            self.push_char(ch);
        }
    }

    /// Finish the primary scrollback line when a command prompt returns.
    ///
    /// The alternate grid has no line to flush: it is a screen, not a log.
    /// A program that printed a final line without a newline in the primary
    /// screen still gets it into the scrollback instead of losing it.
    pub fn flush_line(&mut self) {
        if self.alternate.is_some() {
            return;
        }
        if !self.partial.is_empty() {
            let line = std::mem::take(&mut self.partial);
            self.push_line(line);
        }
        self.carriage_return = false;
    }

    /// Whether a full-screen program currently owns the alternate screen.
    #[must_use]
    pub fn alt_active(&self) -> bool {
        self.alternate.is_some()
    }

    /// The alternate grid, when active.
    #[must_use]
    pub fn screen(&self) -> Option<&Screen> {
        self.alternate.as_ref()
    }

    /// Whether the program asked for a visible cursor.
    #[must_use]
    pub const fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Tell the model how large the visible terminal is.
    ///
    /// The shell also calls `terminal_resize` so the PTY reports the same
    /// numbers to the program. Keeping the model and the PTY in step is what
    /// makes `stty size` and cursor addressing agree.
    pub fn set_viewport(&mut self, cols: usize, rows: usize) {
        let cols = cols.clamp(20, 500);
        let rows = rows.clamp(5, 300);
        if cols == self.screen_cols && rows == self.screen_rows {
            return;
        }
        self.screen_cols = cols;
        self.screen_rows = rows;
        if let Some(screen) = self.alternate.as_mut() {
            screen.resize(cols, rows);
        }
    }

    /// The viewport the model believes it has.
    #[must_use]
    pub const fn viewport(&self) -> (usize, usize) {
        (self.screen_cols, self.screen_rows)
    }

    /// The effective mouse tracking mode.
    #[must_use]
    pub const fn mouse_mode(&self) -> MouseMode {
        if self.mouse_any {
            MouseMode::Any
        } else if self.mouse_button {
            MouseMode::Button
        } else if self.mouse_normal {
            MouseMode::Normal
        } else {
            MouseMode::Off
        }
    }

    /// Whether the program asked for SGR (`?1006`) mouse encoding.
    #[must_use]
    pub const fn mouse_sgr(&self) -> bool {
        self.mouse_sgr
    }

    /// Whether the program asked for bracketed paste (`?2004`).
    #[must_use]
    pub const fn bracketed_paste(&self) -> bool {
        self.bracketed_paste
    }

    /// Whether a terminal mouse button is still down.
    ///
    /// The shell uses this as pointer capture: once a press was delivered, a
    /// drag or release outside the visible grid still belongs to the program.
    #[must_use]
    pub const fn mouse_buttons_held(&self) -> bool {
        self.mouse_held != 0
    }

    /// Drop the held-button mask without sending a report.
    ///
    /// Used when focus or shell chrome interrupts a drag. The program did not
    /// receive a release, so it must not receive a phantom drag on refocus.
    pub fn clear_mouse_buttons(&mut self) {
        self.mouse_held = 0;
    }

    /// Build the bytes for one mouse event, or `None` when the current mode
    /// does not report it. Press and release are always reported while mouse
    /// tracking is on; motion follows `1000`/`1002`/`1003` exactly.
    ///
    /// SGR (`?1006`) keeps coordinates above 223 usable and uses a lowercase
    /// `m` for release. Without `?1006` the legacy X10 form is used; it cannot
    /// represent coordinates past 223, so such a report is dropped rather than
    /// producing a wrong cell.
    pub fn mouse_report(
        &mut self,
        action: MouseAction,
        button: Option<MouseButton>,
        col: usize,
        row: usize,
        mods: Modifiers,
    ) -> Option<Vec<u8>> {
        let mode = self.mouse_mode();
        if mode == MouseMode::Off {
            return None;
        }
        let motion = action == MouseAction::Motion;
        if motion {
            match mode {
                MouseMode::Normal | MouseMode::Off => return None,
                MouseMode::Button if self.mouse_held == 0 => return None,
                MouseMode::Button | MouseMode::Any => {}
            }
        }
        let button = match action {
            MouseAction::Motion => None,
            MouseAction::Press | MouseAction::Release => Some(button?),
        };
        // A release without the matching press did not start in this program
        // (for example the press landed on the window frame). Sending it would
        // leave some full-screen programs believing a button is still down.
        if action == MouseAction::Release {
            if let Some(button) = button {
                if !button.is_wheel() && self.mouse_held & button.held_bit() == 0 {
                    return None;
                }
            }
        }
        let base = if let Some(button) = button {
            button.code()
        } else {
            match self.mouse_held {
                0b001 => 0,
                0b010 => 1,
                0b100 => 2,
                _ => 3,
            }
        };
        let mut code = base;
        if motion {
            code = code.saturating_add(32);
        }
        if mods.shift {
            code = code.saturating_add(4);
        }
        if mods.alt || mods.altgr || mods.meta {
            code = code.saturating_add(8);
        }
        if mods.ctrl {
            code = code.saturating_add(16);
        }
        let release = action == MouseAction::Release;
        let bytes = if self.mouse_sgr {
            let final_char = if release { 'm' } else { 'M' };
            format!(
                "\x1b[<{code};{};{}{}",
                col.saturating_add(1),
                row.saturating_add(1),
                final_char
            )
            .into_bytes()
        } else {
            if col > 222 || row > 222 {
                return None;
            }
            // X10 has no button number on release: Cb 3 says "release", and
            // the modifier bits still travel with it.
            let base = if release {
                3u8.saturating_add(code & 0b0001_1100)
            } else {
                code
            };
            vec![
                0x1b,
                b'[',
                b'M',
                u8::try_from(32u16.saturating_add(u16::from(base))).ok()?,
                u8::try_from(32u16.saturating_add(u16::try_from(col).ok()?.saturating_add(1)))
                    .ok()?,
                u8::try_from(32u16.saturating_add(u16::try_from(row).ok()?.saturating_add(1)))
                    .ok()?,
            ]
        };
        if let Some(button) = button {
            if button.is_wheel() {
                // Wheel is a press-only event in every xterm mode.
            } else if action == MouseAction::Press {
                self.mouse_held |= button.held_bit();
            } else if release {
                self.mouse_held &= !button.held_bit();
            }
        }
        Some(bytes)
    }

    /// Wrap pasted text for the program that is running. Newlines are data and
    /// are not translated into Enter key presses. Bracketed paste is a
    /// protocol boundary: an embedded end marker would let clipboard text
    /// escape it, so it is removed rather than sent as a premature terminator.
    #[must_use]
    pub fn paste_bytes(&self, text: &str) -> Vec<u8> {
        if !self.bracketed_paste {
            return text.as_bytes().to_vec();
        }
        let safe = text.replace("\x1b[201~", "");
        let mut bytes = Vec::with_capacity(safe.len().saturating_add(12));
        bytes.extend_from_slice(b"\x1b[200~");
        bytes.extend_from_slice(safe.as_bytes());
        bytes.extend_from_slice(b"\x1b[201~");
        bytes
    }

    fn reset_input_modes(&mut self) {
        self.mouse_normal = false;
        self.mouse_button = false;
        self.mouse_any = false;
        self.mouse_sgr = false;
        self.mouse_held = 0;
        self.bracketed_paste = false;
    }

    fn enter_alt(&mut self) {
        if self.alternate.is_none() {
            self.alternate = Some(Screen::new(self.screen_cols, self.screen_rows));
            // A program that enters a new screen must start from a known input
            // state: a mouse mode left by the previous program would otherwise
            // keep forwarding clicks into an application that never asked.
            self.reset_input_modes();
        }
        self.sgr = CellStyle::default();
        self.cursor_visible = true;
        self.saved_cursor = None;
    }

    fn leave_alt(&mut self) {
        self.alternate = None;
        self.reset_input_modes();
        self.sgr = CellStyle::default();
        self.cursor_visible = true;
        self.saved_cursor = None;
        self.csi.clear();
        self.csi_private = false;
    }

    /// ANSI full reset (`ESC c`): drop the screen and every mode a program set.
    fn full_reset(&mut self) {
        self.alternate = None;
        self.lines.clear();
        self.partial.clear();
        self.escape = Escape::None;
        self.carriage_return = false;
        self.sgr = CellStyle::default();
        self.cursor_visible = true;
        self.csi.clear();
        self.csi_private = false;
        self.saved_cursor = None;
        self.reset_input_modes();
    }

    fn save_cursor(&mut self) {
        if let Some(screen) = self.alternate.as_ref() {
            self.saved_cursor = Some((screen.cursor_row, screen.cursor_col));
        }
    }

    fn restore_cursor(&mut self) {
        let Some((row, col)) = self.saved_cursor else {
            return;
        };
        if let Some(screen) = self.alternate.as_mut() {
            screen.move_to(row, col);
        }
    }

    fn push_primary_char(&mut self, ch: char) {
        match ch {
            '\u{1b}' => self.escape = Escape::Esc,
            '\r' => self.carriage_return = true,
            '\n' => {
                self.carriage_return = false;
                let line = std::mem::take(&mut self.partial);
                self.push_line(line);
            }
            '\u{8}' => {
                self.partial.pop();
                self.carriage_return = false;
            }
            '\t' => {
                self.partial.push_str("    ");
                self.carriage_return = false;
            }
            c if c.is_control() => {}
            c => {
                if self.carriage_return {
                    self.partial.clear();
                    self.carriage_return = false;
                }
                self.partial.push(c);
            }
        }
    }

    fn push_alt_char(&mut self, ch: char) {
        let Some(screen) = self.alternate.as_mut() else {
            return;
        };
        match ch {
            '\r' => screen.carriage_return(),
            '\n' => screen.line_feed(),
            '\u{8}' => screen.backspace(),
            '\t' => screen.tab(),
            c if c.is_control() => {}
            c => screen.put_styled(c, self.sgr),
        }
    }

    fn push_char(&mut self, ch: char) {
        match self.escape {
            Escape::None => {
                if self.alternate.is_some() {
                    if ch == '\u{1b}' {
                        self.escape = Escape::Esc;
                    } else {
                        self.push_alt_char(ch);
                    }
                } else {
                    self.push_primary_char(ch);
                }
            }
            Escape::Esc => {
                self.escape = match ch {
                    '[' => {
                        self.csi.clear();
                        self.csi_private = false;
                        Escape::Csi
                    }
                    ']' => Escape::Osc,
                    '7' => {
                        self.save_cursor();
                        Escape::None
                    }
                    '8' => {
                        self.restore_cursor();
                        Escape::None
                    }
                    'M' => {
                        if let Some(screen) = self.alternate.as_mut() {
                            screen.reverse_index();
                        }
                        Escape::None
                    }
                    'c' => {
                        self.full_reset();
                        Escape::None
                    }
                    _ => Escape::None,
                };
            }
            Escape::Csi => {
                if ch == '?' && self.csi.is_empty() {
                    self.csi_private = true;
                } else if ch.is_ascii_digit() || ch == ';' {
                    if self.csi.len() < 64 {
                        self.csi.push(ch);
                    }
                } else if ('\u{40}'..='\u{7e}').contains(&ch) {
                    let params = std::mem::take(&mut self.csi);
                    let private = self.csi_private;
                    self.csi_private = false;
                    self.escape = Escape::None;
                    self.apply_csi(&params, private, ch);
                }
            }
            Escape::Osc => match ch {
                '\u{7}' => self.escape = Escape::None,
                '\u{1b}' => self.escape = Escape::OscEsc,
                _ => {}
            },
            Escape::OscEsc => {
                self.escape = if ch == '\\' {
                    Escape::None
                } else {
                    Escape::Osc
                };
            }
        }
    }

    fn apply_csi(&mut self, params: &str, private: bool, final_char: char) {
        let values: Vec<usize> = params
            .split(';')
            .map(|part| part.parse::<usize>().unwrap_or(0))
            .collect();
        let param = |index: usize, default: usize| {
            values
                .get(index)
                .copied()
                .filter(|value| *value > 0)
                .unwrap_or(default)
        };

        if private {
            match final_char {
                'h' | 'l' => {
                    let enable = final_char == 'h';
                    for value in values.iter().copied() {
                        match value {
                            1049 | 1047 | 47 => {
                                if enable {
                                    self.enter_alt();
                                } else {
                                    self.leave_alt();
                                }
                            }
                            25 => self.cursor_visible = enable,
                            1000 => self.mouse_normal = enable,
                            1002 => self.mouse_button = enable,
                            1003 => self.mouse_any = enable,
                            1006 => self.mouse_sgr = enable,
                            2004 => self.bracketed_paste = enable,
                            _ => {}
                        }
                        // A button held while the program turns tracking off
                        // would otherwise make the next 1002 motion look like
                        // a drag with no press.
                        if !enable && matches!(value, 1000 | 1002 | 1003) {
                            self.mouse_held = 0;
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        if final_char == 's' {
            self.save_cursor();
            return;
        }
        if final_char == 'u' {
            self.restore_cursor();
            return;
        }
        if final_char == 'm' {
            self.apply_sgr(&values);
            return;
        }

        let Some(screen) = self.alternate.as_mut() else {
            return;
        };
        match final_char {
            'H' | 'f' => {
                screen.move_to(param(0, 1).saturating_sub(1), param(1, 1).saturating_sub(1))
            }
            'A' => {
                let row = screen.cursor_row.saturating_sub(param(0, 1));
                screen.move_to(row, screen.cursor_col);
            }
            'B' => {
                let row = screen.cursor_row.saturating_add(param(0, 1));
                screen.move_to(row, screen.cursor_col);
            }
            'C' => {
                let col = screen.cursor_col.saturating_add(param(0, 1));
                screen.move_to(screen.cursor_row, col);
            }
            'D' => {
                let col = screen.cursor_col.saturating_sub(param(0, 1));
                screen.move_to(screen.cursor_row, col);
            }
            'G' => {
                let col = param(0, 1).saturating_sub(1);
                screen.move_to(screen.cursor_row, col);
            }
            'd' => {
                let row = param(0, 1).saturating_sub(1);
                screen.move_to(row, screen.cursor_col);
            }
            'J' => match values.first().copied().unwrap_or(0) {
                2 | 3 => screen.clear(),
                _ => {}
            },
            'K' => match values.first().copied().unwrap_or(0) {
                1 => screen.erase_to_start_of_line(),
                2 => {
                    screen.erase_to_start_of_line();
                    screen.erase_to_end_of_line();
                }
                _ => screen.erase_to_end_of_line(),
            },
            'r' => {
                let bottom_default = screen.rows();
                screen.set_scroll_region(
                    param(0, 1).saturating_sub(1),
                    param(1, bottom_default).saturating_sub(1),
                );
            }
            'S' => screen.scroll_up(param(0, 1)),
            'T' => screen.scroll_down(param(0, 1)),
            'L' => screen.insert_lines(param(0, 1)),
            'M' => screen.delete_lines(param(0, 1)),
            _ => {}
        }
    }

    /// Apply one CSI `m` sequence to the SGR state.
    ///
    /// Parameters are parsed as semicolon-separated numbers, which is the form
    /// vim and ordinary terminfo use: `38;5;n` is an indexed colour and
    /// `38;2;r;g;b` is truecolor. A malformed extended colour is ignored up to
    /// its introducer rather than printing the numbers as text.
    fn apply_sgr(&mut self, values: &[usize]) {
        let mut index = 0usize;
        while let Some(code) = values.get(index).copied() {
            match code {
                0 => self.sgr.reset(),
                1 => self.sgr.attrs.bold = true,
                2 => self.sgr.attrs.dim = true,
                3 => self.sgr.attrs.italic = true,
                4 => self.sgr.attrs.underline = true,
                7 => self.sgr.attrs.reverse = true,
                22 => {
                    self.sgr.attrs.bold = false;
                    self.sgr.attrs.dim = false;
                }
                23 => self.sgr.attrs.italic = false,
                24 => self.sgr.attrs.underline = false,
                27 => self.sgr.attrs.reverse = false,
                30..=37 => {
                    self.sgr.fg =
                        Color::Indexed(u8::try_from(code.saturating_sub(30)).unwrap_or(0));
                }
                39 => self.sgr.fg = Color::Default,
                40..=47 => {
                    self.sgr.bg =
                        Color::Indexed(u8::try_from(code.saturating_sub(40)).unwrap_or(0));
                }
                49 => self.sgr.bg = Color::Default,
                90..=97 => {
                    self.sgr.fg =
                        Color::Indexed(u8::try_from(code.saturating_sub(82)).unwrap_or(8));
                }
                100..=107 => {
                    self.sgr.bg =
                        Color::Indexed(u8::try_from(code.saturating_sub(92)).unwrap_or(8));
                }
                38 | 48 => {
                    let Some(kind) = values.get(index.saturating_add(1)).copied() else {
                        break;
                    };
                    let color = if kind == 5 {
                        let Some(value) = values.get(index.saturating_add(2)).copied() else {
                            break;
                        };
                        index = index.saturating_add(2);
                        Color::Indexed(u8::try_from(value).unwrap_or(0))
                    } else if kind == 2 {
                        let Some(r) = values.get(index.saturating_add(2)).copied() else {
                            break;
                        };
                        let Some(g) = values.get(index.saturating_add(3)).copied() else {
                            break;
                        };
                        let Some(b) = values.get(index.saturating_add(4)).copied() else {
                            break;
                        };
                        index = index.saturating_add(4);
                        Color::Rgb(
                            u8::try_from(r).unwrap_or(0),
                            u8::try_from(g).unwrap_or(0),
                            u8::try_from(b).unwrap_or(0),
                        )
                    } else {
                        break;
                    };
                    if code == 38 {
                        self.sgr.fg = color;
                    } else {
                        self.sgr.bg = color;
                    }
                }
                _ => {}
            }
            index = index.saturating_add(1);
        }
    }

    /// Echo the command the user is about to run.
    pub fn echo_command(&mut self, cmd: &str) {
        self.push_line(format!("$ {cmd}"));
    }

    /// Add a successfully-started command to history, avoiding adjacent
    /// duplicates like interactive shells do.
    pub fn remember_command(&mut self, cmd: &str) {
        if !cmd.is_empty() && self.history.back().is_none_or(|last| last != cmd) {
            self.history.push_back(cmd.to_owned());
            while self.history.len() > Self::MAX_HISTORY {
                self.history.pop_front();
            }
        }
        self.history_cursor = None;
        self.history_draft.clear();
    }

    /// Move to an older command, preserving the current input as a draft.
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let cursor = match self.history_cursor {
            Some(0) => return,
            Some(index) => index.saturating_sub(1),
            None => {
                self.history_draft.clone_from(&self.input);
                self.history.len().saturating_sub(1)
            }
        };
        self.history_cursor = Some(cursor);
        if let Some(command) = self.history.get(cursor) {
            self.input.clone_from(command);
        }
    }

    /// Move toward newer commands and restore the in-progress draft at the
    /// newest edge.
    pub fn history_down(&mut self) {
        let Some(cursor) = self.history_cursor else {
            return;
        };
        if cursor.saturating_add(1) >= self.history.len() {
            self.history_cursor = None;
            self.input.clone_from(&self.history_draft);
            self.history_draft.clear();
        } else {
            let next = cursor.saturating_add(1);
            self.history_cursor = Some(next);
            if let Some(command) = self.history.get(next) {
                self.input.clone_from(command);
            }
        }
    }

    /// Clear visible output and the old input draft while retaining history.
    pub fn clear_screen(&mut self) {
        self.alternate = None;
        self.sgr = CellStyle::default();
        self.lines.clear();
        self.input.clear();
        self.partial.clear();
        self.escape = Escape::None;
        self.csi.clear();
        self.csi_private = false;
        self.saved_cursor = None;
        self.carriage_return = false;
        self.cursor_visible = true;
        self.reset_input_modes();
        self.push_line("LitricOS Terminal · session ready");
        self.history_cursor = None;
        self.history_draft.clear();
    }

    /// Start a fresh in-app command session. A command already running must be
    /// allowed to finish before its output is discarded.
    pub fn new_session(&mut self) -> bool {
        if self.busy {
            return false;
        }
        *self = Self::new();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing() -> FilesApp {
        let mut f = FilesApp::default();
        f.apply_list_reply(
            "e name=%64oc.txt kind=file size=3 mtime=100 hidden=0 target=\n\
             e name=Work kind=dir size=0 mtime=50 hidden=0 target=\n\
             e name=%41lbum kind=dir size=0 mtime=70 hidden=0 target=\n\
             truncated=1\n",
        );
        f
    }

    #[test]
    fn list_reply_parses_into_sorted_rows() {
        let f = listing();
        assert_eq!(f.rows.len(), 3);
        let names: Vec<&str> = f.visible().iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Album", "Work", "doc.txt"]);
        let kinds: Vec<FileKind> = f.rows.iter().map(|r| r.kind).collect();
        assert!(kinds.contains(&FileKind::Dir));
    }

    #[test]
    fn a_failed_directory_read_replaces_the_rows_with_the_reason() {
        // The old window kept the previous listing behind an error banner, so a
        // folder that had become unreadable still looked like a folder whose
        // contents were the rows from before. The rows are dropped: "empty" and
        // "could not be read" must not look the same.
        let mut f = listing();
        assert!(f.error.is_none());
        f.apply_error("permission denied".to_owned());
        assert!(f.rows.is_empty(), "stale rows survived a failed read");
        assert_eq!(f.error.as_deref(), Some("permission denied"));
        assert!(f.selection().is_none());
    }

    #[test]
    fn history_works_like_explorers_back_forward_stack() {
        let mut f = FilesApp::default();
        f.navigate("/Pictures");
        assert_eq!(f.path, "/Pictures");
        f.navigate("/Music");
        assert!(f.go_back());
        assert_eq!(f.path, "/Pictures");
        assert!(f.go_forward());
        assert_eq!(f.path, "/Music");
        assert!(!f.go_forward());
        f.navigate("/Deep/A/B");
        assert!(f.forward.is_empty(), "new navigation drains forward");
    }

    #[test]
    fn up_from_a_nested_path_goes_to_its_parent() {
        let mut f = FilesApp::default();
        f.navigate("/Documents/Work");
        f.go_up();
        assert_eq!(f.path, "/Documents");
        f.go_up();
        assert_eq!(f.path, "/");
    }

    #[test]
    fn breadcrumbs_and_crumb_targets_match_the_path() {
        let mut f = FilesApp::default();
        f.navigate("/Documents/Work/2026");
        assert_eq!(f.breadcrumbs(), ["Documents", "Work", "2026"]);
        assert_eq!(f.breadcrumb_path(1), "/Documents/Work");
        assert_eq!(f.breadcrumb_path(9), "/Documents/Work/2026", "clamped");
    }

    #[test]
    fn history_buttons_know_when_they_have_nowhere_to_go() {
        let mut f = FilesApp::default();
        assert!(!f.can_go_back());
        assert!(!f.can_go_forward());
        assert!(
            f.can_go_up(),
            "the default window is a folder, not the root"
        );
        f.navigate("/");
        assert!(!f.can_go_up(), "the root has no parent");
        f.navigate("/Documents");
        assert!(f.can_go_back());
        assert!(!f.can_go_forward());
        assert!(f.go_back());
        assert!(f.can_go_forward());
    }

    #[test]
    fn the_root_crumb_goes_home_from_any_depth() {
        let mut f = FilesApp::default();
        f.navigate("/Documents/Work");
        let path = files_action(&mut f, FilesHot::Root);
        assert_eq!(path.as_deref(), Some("/"));
        assert_eq!(f.path, "/");
    }

    #[test]
    fn sorting_toggles_and_switches_columns() {
        // Files only, so the flip is visible: dirs-first grouping would pin
        // directories to the top in both directions (as in Explorer).
        let mut f = FilesApp::default();
        f.apply_list_reply(
            "e name=a.txt kind=file size=1 mtime=10 hidden=0 target=\n\
             e name=b.txt kind=file size=9 mtime=20 hidden=0 target=\n",
        );
        f.toggle_sort(SortKey::Size);
        let asc: Vec<u64> = f.visible().iter().map(|r| r.size).collect();
        assert_eq!(asc, [1, 9]);
        f.toggle_sort(SortKey::Size);
        let desc: Vec<u64> = f.visible().iter().map(|r| r.size).collect();
        assert_eq!(desc, [9, 1]);
        // Switching columns resets to ascending.
        f.toggle_sort(SortKey::Modified);
        let mt: Vec<u64> = f.visible().iter().map(|r| r.mtime).collect();
        assert_eq!(mt, [10, 20]);
    }

    #[test]
    fn the_filter_narrows_without_touching_the_listing() {
        let mut f = listing();
        f.filter = "work".to_owned();
        let names: Vec<&str> = f.visible().iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Work"]);
        assert_eq!(f.rows.len(), 3);
    }

    #[test]
    fn selection_survives_through_the_visible_view() {
        let mut f = listing();
        f.selected = Some(2);
        // Visible order is dirs-first: index 2 is doc.txt.
        assert_eq!(f.selection().map(|r| r.name.as_str()), Some("doc.txt"));
        assert_eq!(
            f.selected_path().as_deref(),
            Some("/Documents/doc.txt"),
            "the logical path joins the shown folder with the row name"
        );
    }

    #[test]
    fn multiple_selection_tracks_names_when_sorted_and_compresses_together() {
        let mut f = listing();
        f.select_row(0);
        f.toggle_row(2);
        assert_eq!(
            f.selected_paths(),
            ["/Documents/Album", "/Documents/doc.txt"]
        );
        assert!(f.row_is_selected(0));
        assert!(f.row_is_selected(2));
        f.toggle_sort(SortKey::Size);
        let sorted = f.visible();
        let album = sorted.iter().position(|row| row.name == "Album").unwrap();
        assert!(
            f.row_is_selected(album),
            "selection follows Album after resort"
        );
        assert!(
            f.row_is_selected(2),
            "selection follows doc.txt after resort"
        );
        let _ = files_action(&mut f, FilesHot::Compress);
        assert!(matches!(
            f.dialog,
            Some(DialogKind::Confirm {
                action: FilesOp::CompressMany { ref sources, .. },
                ..
            }) if sources.len() == 2
        ));
    }

    #[test]
    fn select_all_uses_the_visible_rows_and_filter_is_respected() {
        let mut f = listing();
        f.filter = "work".to_owned();
        f.select_all_visible();
        assert_eq!(f.selected_paths(), ["/Documents/Work"]);
        f.filter.clear();
        f.select_all_visible();
        assert_eq!(f.selected_paths().len(), 3);
        f.apply_entries(f.rows.clone());
        assert!(
            f.selected_paths().is_empty(),
            "a fresh listing clears stale selection"
        );
    }

    #[test]
    fn a_files_place_click_navigates_and_pins() {
        let mut f = FilesApp::default();
        // The first place is the home root, the way Explorer starts at This PC.
        let path = files_action(&mut f, FilesHot::Place(0)).expect("navigates");
        assert_eq!(path, "/");
        assert_eq!(f.place_index, Some(0));
        assert_eq!(
            f.places.first().map(|(p, l)| (p.as_str(), l.as_str())),
            Some(("/", "This Litric"))
        );
        let path = files_action(&mut f, FilesHot::Place(2)).expect("navigates");
        assert_eq!(path, "/Pictures");
        assert_eq!(f.place_index, Some(2));
    }

    #[test]
    fn a_header_click_only_resorts() {
        let mut f = listing();
        assert!(files_action(&mut f, FilesHot::Header(SortKey::Size)).is_none());
        assert_eq!(f.sort, (SortKey::Size, true));
    }

    #[test]
    fn clipboard_records_copy_and_cut() {
        let mut f = listing();
        f.selected = Some(2);
        assert!(f.copy_selection());
        assert_eq!(
            f.clipboard.as_ref().map(|(cut, p)| (*cut, p.as_str())),
            Some((false, "/Documents/doc.txt"))
        );
        assert!(f.cut_selection());
        assert_eq!(
            f.clipboard.as_ref().map(|(cut, p)| (*cut, p.as_str())),
            Some((true, "/Documents/doc.txt"))
        );
        f.selected = None;
        assert!(!f.copy_selection(), "nothing selected, nothing copied");
    }

    #[test]
    fn dialogs_open_with_the_right_field_and_selection() {
        let mut f = listing();
        f.selected = Some(2);
        f.ask_rename();
        match &f.dialog {
            Some(DialogKind::Rename { path, current }) => {
                assert_eq!(path, "/Documents/doc.txt");
                assert_eq!(current, "doc.txt");
            }
            other => panic!("rename dialog expected, got {other:?}"),
        }
        assert_eq!(f.dialog_field, "doc.txt");
        assert_eq!(f.dialog_selection, Some((0, 3)), "the stem is pre-selected");
        f.close_dialog();
        assert!(f.dialog.is_none());

        f.ask_new_folder();
        assert!(matches!(f.dialog, Some(DialogKind::NewFolder)));
        assert!(f.dialog_field.is_empty());
        f.close_dialog();

        f.ask_new_file();
        assert_eq!(f.dialog_field, "untitled.txt");
        assert_eq!(f.dialog_selection, Some((0, 8)), "stem of untitled.txt");
    }

    #[test]
    fn browser_tabs_open_close_and_keep_the_window_alive() {
        let mut b = BrowserApp::default();
        b.new_tab();
        b.select(0);
        assert_eq!(b.tab().map(|t| t.url.as_str()), Some("litric://about"));
        b.close(0);
        assert_eq!(b.tabs.len(), 1, "fresh tab replaces the last close");
    }

    #[test]
    fn browser_close_keeps_the_focused_tab_focused() {
        let mut b = BrowserApp::default();
        b.tabs.clear();
        for url in ["a", "b", "c", "d"] {
            b.new_tab();
            b.navigate_active_to(url);
        }
        b.select(2); // "c"
        b.close(0);
        assert_eq!(b.tabs.len(), 3);
        assert_eq!(
            b.tab().map(|t| t.url.as_str()),
            Some("c"),
            "closing an earlier tab must not hand focus to the next one"
        );
        // Closing the focused tab selects the tab that takes its place.
        b.close(b.active);
        assert_eq!(b.tab().map(|t| t.url.as_str()), Some("d"));
    }

    #[test]
    fn browser_history_back_forward_and_new_navigation_trim_forward() {
        let mut b = BrowserApp::default();
        b.navigate_active_to("https://one.example");
        b.navigate_active_to("https://two.example");
        assert_eq!(b.back().as_deref(), Some("https://one.example/"));
        assert_eq!(b.forward().as_deref(), Some("https://two.example/"));
        assert_eq!(b.back().as_deref(), Some("https://one.example/"));
        b.navigate_active_to("https://three.example");
        assert!(
            !b.can_go_forward(),
            "a new route drops the old forward branch"
        );
        assert_eq!(
            b.history.front().map(String::as_str),
            Some("https://three.example/")
        );
    }

    #[test]
    fn one_site_has_one_entry_however_it_is_typed() {
        // BUG-0036: the shell used the typed text as the key, so the same page
        // reached by three spellings filled the recent list, added three Back
        // steps and left the bookmark looking unset after a second visit.
        let mut b = BrowserApp::default();
        for spelling in [
            "https://example.com",
            "https://EXAMPLE.com/",
            "https://example.com:443",
        ] {
            b.omnibox = spelling.to_owned();
            assert_eq!(
                b.commit_omnibox().as_deref(),
                Some("https://example.com/"),
                "{spelling} must resolve to the canonical text"
            );
        }
        assert_eq!(
            b.history.front().map(String::as_str),
            Some("https://example.com/"),
            "the recent list holds one spelling"
        );
        assert_eq!(
            b.tabs.first().map(|tab| tab.history_index),
            Some(1),
            "the tab gained one entry, not three"
        );
        assert!(
            b.toggle_bookmark(),
            "the page can be bookmarked from the canonical address"
        );
        b.omnibox = "https://EXAMPLE.com:443/".to_owned();
        assert_eq!(b.commit_omnibox().as_deref(), Some("https://example.com/"));
        assert!(
            b.is_bookmarked(),
            "a second spelling must not read as unbookmarked"
        );
        assert!(!b.toggle_bookmark(), "toggling removes the saved address");
        assert!(b.bookmarks.is_empty());
    }

    #[test]
    fn canonicalisation_leaves_search_urls_and_plain_tokens_alone() {
        // `canonical_address` is applied on the navigation path, which also
        // carries the engine's own search URL. That URL is not an address the
        // parser knows (its query is `/search?q=...` on a known host, so it is
        // an address), but a non-URL token must pass through untouched.
        assert_eq!(
            BrowserApp::canonical_address("litric://about"),
            "litric://about"
        );
        assert_eq!(BrowserApp::canonical_address("c"), "c");
        assert_eq!(
            BrowserApp::canonical_address("https://www.google.com/search?q=litric%20docs"),
            "https://www.google.com/search?q=litric%20docs"
        );
    }

    #[test]
    fn browser_bookmarks_toggle_and_remain_unique() {
        let mut b = BrowserApp::default();
        b.navigate_active_to("https://litricos.example");
        assert!(b.toggle_bookmark());
        assert!(b.is_bookmarked());
        assert!(!b.toggle_bookmark());
        assert!(!b.is_bookmarked());
        assert!(b.bookmarks.is_empty());
    }

    #[test]
    fn decoding_covers_percent_and_dash_forms() {
        assert_eq!(decode("%64oc.txt"), "doc.txt");
        assert_eq!(decode("-"), "");
        assert_eq!(decode("plain"), "plain");
        assert_eq!(decode("%zz"), "%zz", "bad hex stays literal");
    }

    #[test]
    fn browser_hotspots_map_totally() {
        let mut b = BrowserApp::default();
        assert!(browser_action(&mut b, Hot::BrowserNewTab).is_none());
        assert_eq!(b.tabs.len(), 2);
        assert!(browser_action(&mut b, Hot::BrowserTab(1)).is_none());
        assert_eq!(b.active, 1);
        b.omnibox = "example.com".to_owned();
        assert_eq!(
            browser_action(&mut b, Hot::BrowserGo).as_deref(),
            Some("https://example.com/")
        );
        assert!(browser_action(&mut b, Hot::BrowserCloseTab(1)).is_none());
        assert_eq!(b.tabs.len(), 1);
    }

    #[test]
    fn the_omnibox_searches_google_by_default() {
        let mut b = BrowserApp {
            omnibox: "example.com".to_owned(),
            ..BrowserApp::default()
        };
        assert_eq!(
            b.commit_omnibox().as_deref(),
            Some("https://example.com/"),
            "the omnibox returns the canonical address"
        );
        b.omnibox = "litric docs".to_owned();
        assert_eq!(
            b.commit_omnibox().as_deref(),
            Some("https://www.google.com/search?q=litric%20docs"),
            "the default engine is Google"
        );
        b.omnibox = "file:///etc/hostname".to_owned();
        assert_eq!(b.commit_omnibox().as_deref(), Some("file:///etc/hostname"));
        b.omnibox = "  ".to_owned();
        assert_eq!(b.commit_omnibox(), None);
    }

    #[test]
    fn a_malformed_address_is_refused_in_the_omnibox_instead_of_searched_or_sent() {
        let mut b = BrowserApp {
            omnibox: "https://example.com:abc/".to_owned(),
            ..BrowserApp::default()
        };
        assert_eq!(b.commit_omnibox(), None);
        assert!(b.status.contains("not valid"), "{}", b.status);
        assert!(
            b.tabs.first().is_some_and(|tab| tab.error),
            "the tab carries the error"
        );
        assert_eq!(
            b.tabs.first().map(|tab| tab.url.as_str()),
            Some("litric://about"),
            "the tab did not move"
        );
        // The hotspot path returns nothing to load, so no engine request is
        // made for a malformed address.
        b.omnibox = "ftp://example.com/".to_owned();
        assert!(browser_action(&mut b, Hot::BrowserGo).is_none());
        assert!(b.status.contains("not valid"), "{}", b.status);
        // A host-like word with a bad port is an address attempt too, and used
        // to skip validation because it carried no `://`.
        b.omnibox = "example.com:abc".to_owned();
        assert!(browser_action(&mut b, Hot::BrowserGo).is_none());
        assert!(b.status.contains("not valid"), "{}", b.status);
        // Valid addresses and plain-text searches resolve exactly as before,
        // and a successful navigation clears the tab's error.
        b.omnibox = "example.com".to_owned();
        assert_eq!(
            browser_action(&mut b, Hot::BrowserGo).as_deref(),
            Some("https://example.com/")
        );
        assert!(b.tabs.first().is_some_and(|tab| !tab.error));
        b.omnibox = "litric docs".to_owned();
        assert_eq!(
            b.commit_omnibox().as_deref(),
            Some("https://www.google.com/search?q=litric%20docs")
        );
    }

    #[test]
    fn search_engines_round_trip_through_their_stored_tokens() {
        for engine in SearchEngine::ALL {
            assert_eq!(SearchEngine::parse(engine.token()), Some(engine));
        }
        assert_eq!(SearchEngine::parse("klingon"), None);
        assert_eq!(SearchEngine::parse(""), None);
        // The token is not the translated title: a saved file must not depend
        // on the language the session was in when it was written.
        let tokens: Vec<&str> = SearchEngine::ALL.iter().map(|e| e.token()).collect();
        assert_eq!(tokens, vec!["google", "duckduckgo", "bing"]);
    }

    #[test]
    fn search_engines_build_their_urls_and_round_trip() {
        assert_eq!(
            SearchEngine::Google.search_url("cat dog"),
            "https://www.google.com/search?q=cat%20dog"
        );
        assert_eq!(
            SearchEngine::DuckDuckGo.search_url("cat dog"),
            "https://duckduckgo.com/?q=cat%20dog"
        );
        assert_eq!(
            SearchEngine::Bing.search_url("cat dog"),
            "https://www.bing.com/search?q=cat%20dog"
        );
        let mut b = BrowserApp {
            engine: SearchEngine::DuckDuckGo,
            omnibox: "rust book".to_owned(),
            ..BrowserApp::default()
        };
        assert_eq!(
            b.commit_omnibox().as_deref(),
            Some("https://duckduckgo.com/?q=rust%20book")
        );
    }

    #[test]
    fn the_defender_model_tracks_a_scan_end_to_end() {
        let mut d = DefenderApp::default();
        d.start(3);
        d.record("/Documents/a.txt", &Decision::Allow);
        d.record(
            "/Downloads/eicar.com",
            &Decision::Deny {
                tier: litric_defender::verdict::Tier::Hash,
                id: "eicar-test-file".to_owned(),
            },
        );
        d.record("/Documents/b.txt", &Decision::Allow);
        assert_eq!(d.state, ScanState::Scanning { done: 3, total: 3 });
        assert!(d.last_had_threats);
        d.finish();
        assert_eq!(
            d.state,
            ScanState::Done {
                scanned: 3,
                threats: 1
            }
        );
        assert!(d.message.contains("1 threat found"), "{}", d.message);
        assert_eq!(d.rows.iter().filter(|r| r.detected).count(), 1);
    }

    #[test]
    fn the_defender_model_stays_bounded_on_big_scans() {
        let mut d = DefenderApp::default();
        d.start(600);
        for i in 0..600 {
            d.record(&format!("/Documents/f{i}.txt"), &Decision::Allow);
        }
        assert!(d.rows.len() <= 200, "clean rows must be capped");
        assert_eq!(
            d.state,
            ScanState::Scanning {
                done: 600,
                total: 600
            }
        );
    }

    #[test]
    fn the_desktop_grid_has_the_standard_icons() {
        let g = DesktopIcons::standard();
        assert!(g.items.len() >= 8);
        assert!(g
            .items
            .iter()
            .any(|i| matches!(i.target, IconTarget::Folder(ref p) if p == "/Projects")));
        let mut g2 = DesktopIcons::standard();
        g2.select(Some(1));
        assert_eq!(g2.selected, Some(1));
        g2.select(Some(99));
        assert_eq!(g2.selected, None, "an out-of-range click selects nothing");
        g2.select(None);
        assert_eq!(g2.selected, None);
    }

    #[test]
    fn a_viewer_shows_only_the_lines_that_fit() {
        let text: String = (0..100).map(|i| format!("line {i}\n")).collect();
        let mut v = FileViewer::new("/Documents/long.txt".to_owned(), text);
        v.scroll = 50;
        let (lines, total) = v.window(5);
        assert_eq!(total, 100, "every line is counted without materializing it");
        assert_eq!(
            lines,
            ["line 50", "line 51", "line 52", "line 53", "line 54"]
        );

        v.scroll = 98;
        let (lines, total) = v.window(10);
        assert_eq!(total, 100);
        assert_eq!(lines, ["line 98", "line 99"], "the page clamps at the end");

        v.scroll = 9_999;
        let (lines, _) = v.window(3);
        assert_eq!(
            lines,
            ["line 99"],
            "a scroll past the end shows the last page"
        );
    }

    #[test]
    fn a_viewer_counts_lines_the_way_an_editor_does() {
        // A trailing newline ends the last line rather than opening an empty
        // one, CRLF keeps no carriage return in the drawn text, and an empty
        // file has no lines at all.
        let v = FileViewer::new("a".to_owned(), "one\ntwo\n".to_owned());
        assert_eq!(v.window(10).1, 2);
        assert_eq!(v.window(10).0, ["one", "two"]);

        let v = FileViewer::new("b".to_owned(), "one\r\ntwo\r\n".to_owned());
        assert_eq!(v.window(10).0, ["one", "two"]);

        let v = FileViewer::new("c".to_owned(), String::new());
        let (lines, total) = v.window(10);
        assert_eq!(total, 0);
        assert!(lines.is_empty());

        let v = FileViewer::new("d".to_owned(), "\n".to_owned());
        assert_eq!(v.window(10).1, 1, "one empty line, not two");
    }

    #[test]
    fn a_viewer_slices_multibyte_lines_on_their_boundaries() {
        // The index is in bytes, and a slice that started one byte inside a
        // Thai character would panic or lose text. `get` on the str keeps that
        // impossible, but the test holds it to the UTF-8 contract.
        let v = FileViewer::new("ไทย.txt".to_owned(), "ไทย\nภาษาไทย\nok".to_owned());
        let (lines, total) = v.window(3);
        assert_eq!(total, 3);
        assert_eq!(lines, ["ไทย", "ภาษาไทย", "ok"]);
    }

    #[test]
    fn selection_asked_once_per_built_list_matches_the_one_shot_query() {
        let mut f = FilesApp {
            path: "/Documents".to_owned(),
            ..FilesApp::default()
        };
        for (name, size) in [("b.txt", 3u64), ("A.txt", 1), ("c.txt", 2)] {
            f.rows.push(FileRow {
                name: name.to_owned(),
                kind: FileKind::File,
                size,
                mtime: 0,
            });
        }
        f.filter = "t".to_owned();
        f.select_row(2);
        let rows = f.visible();
        assert_eq!(rows.len(), 3);
        for i in 0..rows.len() {
            assert_eq!(
                f.row_is_selected_in(&rows, i),
                f.row_is_selected(i),
                "row {i} answered differently from the one-shot query"
            );
        }
        assert_eq!(f.selected_paths_in(&rows), f.selected_paths());
    }

    #[test]
    fn first_boot_desktop_keeps_only_frequent_destinations() {
        let grid = DesktopIcons::minimal();
        assert_eq!(grid.items.len(), 4);
        // `get` rather than indexing: the assertion is about *which* entries
        // exist, so "there is no fourth entry" has to be a failure with a
        // message rather than a panic behind it.
        let target = |n: usize| grid.items.get(n).map(|item| &item.target);
        assert!(matches!(
            target(0),
            Some(IconTarget::App(crate::shortcuts::App::Files))
        ));
        assert!(matches!(
            target(1),
            Some(IconTarget::App(crate::shortcuts::App::Browser))
        ));
        assert!(matches!(target(2), Some(IconTarget::Folder(path)) if path == "/Documents"));
        assert!(matches!(target(3), Some(IconTarget::Folder(path)) if path == "/Downloads"));
    }
}

#[cfg(test)]
mod scan_mode_tests {
    use super::ScanMode;

    /// The two buttons are two promises. If they ever cover the same ground,
    /// one of them is a lie and the window should not offer it.
    #[test]
    fn the_two_modes_promise_different_coverage() {
        assert!(ScanMode::Deep.depth() > ScanMode::Fast.depth());
        assert!(ScanMode::Deep.file_cap() > ScanMode::Fast.file_cap());
        assert_eq!(ScanMode::Fast.roots().len(), 6);
        assert_eq!(ScanMode::Deep.roots(), &["/"][..]);
        assert_ne!(ScanMode::Fast.coverage(), ScanMode::Deep.coverage());
        assert_ne!(ScanMode::Fast.label(), ScanMode::Deep.label());
        assert_eq!(ScanMode::default(), ScanMode::Fast);
    }

    /// The queue caps are memory bounds as much as time bounds, so they are
    /// asserted by value: raising one has to be a deliberate edit here, not a
    /// silent change to how much a scan may hold.
    #[test]
    fn the_scan_queue_caps_are_pinned() {
        assert_eq!(ScanMode::Fast.file_cap(), 2_000);
        assert_eq!(ScanMode::Deep.file_cap(), 40_000);
        assert!(ScanMode::Fast.depth() <= ScanMode::Deep.depth());
    }
}

#[cfg(test)]
mod terminal_output_tests {
    use super::{MouseAction, MouseButton, MouseMode, TerminalApp};
    use crate::terminal_screen::{CellStyle, Color};
    use litric_input::keys::Modifiers;

    #[test]
    fn terminal_output_drops_ansi_and_handles_crlf() {
        let mut term = TerminalApp::new();
        term.push_output(b"\x1b[31mred\x1b[0m\r\nnext\n");
        assert!(
            term.lines.iter().any(|line| line == "red"),
            "{:?}",
            term.lines
        );
        assert!(
            term.lines.iter().any(|line| line == "next"),
            "{:?}",
            term.lines
        );
        assert!(
            !term.lines.iter().any(|line| line.contains('\u{1b}')),
            "an escape reached the scrollback"
        );
    }

    #[test]
    fn terminal_output_overwrites_a_progress_line_on_carriage_return() {
        let mut term = TerminalApp::new();
        term.push_output(b"10%\r20%\r\n");
        assert!(
            term.lines.iter().any(|line| line == "20%"),
            "{:?}",
            term.lines
        );
        assert!(
            !term.lines.iter().any(|line| line == "10%"),
            "the progress line was not overwritten"
        );
    }

    #[test]
    fn terminal_output_flushes_a_line_without_a_newline_at_the_prompt() {
        let mut term = TerminalApp::new();
        term.push_output(b"done");
        term.flush_line();
        assert!(term.lines.iter().any(|line| line == "done"));
    }

    #[test]
    fn alternate_screen_keeps_the_primary_scrollback_and_restores_the_cursor() {
        let mut term = TerminalApp::new();
        term.set_viewport(40, 12);
        term.push_output(b"before-alt\n");
        term.push_output(b"\x1b[?1049h\x1b[2J\x1b[2;3HALT");
        assert!(term.alt_active(), "CSI ?1049h did not open the grid");
        let screen = term.screen().expect("alternate screen");
        assert_eq!((screen.cols(), screen.rows()), (40, 12));
        assert_eq!(screen.cell(1, 2), Some('A'));
        assert!(term.lines.iter().any(|line| line == "before-alt"));

        term.push_output(b"\x1b[?25l");
        assert!(!term.cursor_visible(), "CSI ?25l was ignored");
        term.set_viewport(52, 18);
        assert_eq!(
            term.screen().map(|screen| (screen.cols(), screen.rows())),
            Some((52, 18))
        );

        term.push_output(b"\x1b[?1049lAFTER");
        assert!(!term.alt_active(), "CSI ?1049l did not restore primary");
        assert!(term.cursor_visible());
        assert!(term.lines.iter().any(|line| line == "before-alt"));
        term.push_output(b"\n");
        assert!(term.lines.iter().any(|line| line == "AFTER"));
    }

    #[test]
    fn alternate_screen_clear_and_erase_do_not_touch_scrollback() {
        let mut term = TerminalApp::new();
        term.set_viewport(20, 5);
        term.push_output(b"kept\n");
        term.push_output(b"\x1b[?47h\x1b[2Jone\ntwo\x1b[K\x1b[?47l");
        assert!(!term.alt_active());
        assert!(term.lines.iter().any(|line| line == "kept"));
        assert!(!term.lines.iter().any(|line| line.contains("one")));
    }
    #[test]
    fn alternate_sgr_stores_16_256_truecolor_and_attributes_per_cell() {
        let mut term = TerminalApp::new();
        term.set_viewport(20, 4);
        term.push_output(
            b"\x1b[?1049h\x1b[31;1;4mA\x1b[0mB\x1b[94mC\x1b[38;5;196mD\x1b[38;2;1;2;3mE\x1b[48;5;17mF\x1b[7mG",
        );
        let screen = term.screen().expect("alternate screen");
        let style = |col: usize| {
            screen
                .cell_at(0, col)
                .map(|cell| cell.style)
                .expect("styled cell")
        };
        let a = style(0);
        assert_eq!(a.fg, Color::Indexed(1));
        assert!(a.attrs.bold && a.attrs.underline);
        assert_eq!(style(1), CellStyle::default(), "SGR 0 did not reset");
        assert_eq!(style(2).fg, Color::Indexed(12));
        assert_eq!(style(3).fg, Color::Indexed(196));
        assert_eq!(style(4).fg, Color::Rgb(1, 2, 3));
        assert_eq!(style(5).bg, Color::Indexed(17));
        assert!(style(6).attrs.reverse);
        assert_eq!(screen.text_lines(), ["ABCDEFG", "", "", "", ""]);
    }

    #[test]
    fn alternate_sgr_resets_all_attributes_and_does_not_leak_across_transitions() {
        let mut term = TerminalApp::new();
        term.set_viewport(20, 4);
        term.push_output(b"\x1b[31;1m");
        term.push_output(b"\x1b[?1049hA");
        assert_eq!(
            term.screen()
                .and_then(|screen| screen.cell_at(0, 0))
                .map(|cell| cell.style),
            Some(CellStyle::default()),
            "entering alternate screen did not reset SGR"
        );

        term.push_output(b"\x1b[2;3;7;4;1mB\x1b[22;23;24;27mC\x1b[38;5;9mD\x1b[mE");
        let screen = term.screen().expect("alternate screen");
        let b = screen.cell_at(0, 1).expect("B").style;
        assert!(b.attrs.bold && b.attrs.dim && b.attrs.italic);
        assert!(b.attrs.underline && b.attrs.reverse);
        let c = screen.cell_at(0, 2).expect("C").style;
        assert!(!c.attrs.bold && !c.attrs.dim && !c.attrs.italic);
        assert!(!c.attrs.underline && !c.attrs.reverse);
        assert_eq!(c.fg, Color::Default);
        assert_eq!(
            screen.cell_at(0, 3).map(|cell| cell.style.fg),
            Some(Color::Indexed(9))
        );
        assert_eq!(
            screen.cell_at(0, 4).map(|cell| cell.style),
            Some(CellStyle::default()),
            "CSI m without parameters did not reset"
        );

        term.push_output(b"\x1b[?1049l");
        term.push_output(b"\x1b[?1049hC");
        assert_eq!(
            term.screen()
                .and_then(|screen| screen.cell_at(0, 0))
                .map(|cell| cell.style),
            Some(CellStyle::default()),
            "re-entering alternate screen inherited the old SGR state"
        );
    }

    #[test]
    fn alternate_grid_places_wide_and_combining_text_through_the_parser() {
        let mut term = TerminalApp::new();
        term.set_viewport(20, 4);
        term.push_output("\x1b[?1049h界AกัB".as_bytes());
        let screen = term.screen().expect("alternate screen");
        assert_eq!(
            screen.cell_at(0, 0).map(|cell| cell.text.as_str()),
            Some("界")
        );
        assert_eq!(screen.cell_at(0, 1).map(|cell| cell.width), Some(0));
        assert_eq!(
            screen.cell_at(0, 2).map(|cell| cell.text.as_str()),
            Some("A")
        );
        assert_eq!(
            screen.cell_at(0, 3).map(|cell| cell.text.as_str()),
            Some("กั")
        );
        assert_eq!(
            screen.cell_at(0, 4).map(|cell| cell.text.as_str()),
            Some("B")
        );
        assert_eq!(screen.cursor_col, 5);
    }
    #[test]
    fn private_mouse_and_paste_modes_track_and_clear_on_screen_switches() {
        let mut term = TerminalApp::new();
        term.push_output(b"\x1b[?1000h\x1b[?1006h\x1b[?2004h");
        assert_eq!(term.mouse_mode(), MouseMode::Normal);
        assert!(term.mouse_sgr());
        assert!(term.bracketed_paste());

        term.push_output(b"\x1b[?1002h");
        assert_eq!(term.mouse_mode(), MouseMode::Button);
        term.push_output(b"\x1b[?1000l");
        assert_eq!(
            term.mouse_mode(),
            MouseMode::Button,
            "1002 still owns motion"
        );
        term.push_output(b"\x1b[?1003h\x1b[?1002l");
        assert_eq!(term.mouse_mode(), MouseMode::Any);
        term.push_output(b"\x1b[?2004l");
        assert!(!term.bracketed_paste());

        // Entering a screen clears modes that belonged to the old surface.
        term.push_output(b"\x1b[?1049h");
        assert_eq!(term.mouse_mode(), MouseMode::Off);
        assert!(!term.mouse_sgr());
        assert!(!term.bracketed_paste());
        term.push_output(b"\x1b[?1000h\x1b[?1006h\x1b[?2004h");
        term.push_output(b"\x1b[?1049l");
        assert_eq!(term.mouse_mode(), MouseMode::Off);
        assert!(!term.mouse_sgr());
        assert!(!term.bracketed_paste());

        // A full reset must not leave a mode that can capture the next app.
        term.push_output(b"\x1b[?1003h\x1b[?2004h\x1b[?1006h");
        term.push_output(b"\x1bc");
        assert_eq!(term.mouse_mode(), MouseMode::Off);
        assert!(!term.mouse_sgr());
        assert!(!term.bracketed_paste());
    }

    #[test]
    fn sgr_mouse_reports_follow_normal_button_and_any_event_rules() {
        let mut term = TerminalApp::new();
        let none = Modifiers::NONE;
        assert!(
            term.mouse_report(MouseAction::Press, Some(MouseButton::Left), 0, 0, none)
                .is_none(),
            "mouse reporting is off before a program enables it"
        );
        term.push_output(b"\x1b[?1000h\x1b[?1006h");
        assert_eq!(
            term.mouse_report(MouseAction::Press, Some(MouseButton::Left), 2, 1, none)
                .expect("press"),
            b"\x1b[<0;3;2M"
        );
        assert!(
            term.mouse_report(MouseAction::Motion, None, 3, 1, none)
                .is_none(),
            "1000 does not report motion"
        );
        assert_eq!(
            term.mouse_report(MouseAction::Release, Some(MouseButton::Left), 3, 1, none)
                .expect("release"),
            b"\x1b[<0;4;2m"
        );
        assert_eq!(
            term.mouse_report(MouseAction::Press, Some(MouseButton::WheelUp), 0, 0, none)
                .expect("wheel"),
            b"\x1b[<64;1;1M"
        );

        term.push_output(b"\x1b[?1002h");
        assert_eq!(
            term.mouse_report(MouseAction::Press, Some(MouseButton::Left), 2, 1, none)
                .expect("press"),
            b"\x1b[<0;3;2M"
        );
        assert_eq!(
            term.mouse_report(MouseAction::Motion, None, 3, 1, none)
                .expect("drag motion"),
            b"\x1b[<32;4;2M"
        );
        assert_eq!(
            term.mouse_report(MouseAction::Release, Some(MouseButton::Left), 3, 1, none)
                .expect("release"),
            b"\x1b[<0;4;2m"
        );
        assert!(
            term.mouse_report(MouseAction::Motion, None, 4, 1, none)
                .is_none(),
            "1002 stops reporting motion after the button is released"
        );
        assert!(
            term.mouse_report(MouseAction::Release, Some(MouseButton::Right), 4, 1, none)
                .is_none(),
            "a release without its press did not start in this program"
        );

        // Turning tracking off drops the held-button state. Re-enabling it
        // must not turn a later hover into a phantom drag.
        term.push_output(b"\x1b[?1002l");
        term.push_output(b"\x1b[?1002h");
        assert!(
            term.mouse_report(MouseAction::Motion, None, 4, 1, none)
                .is_none(),
            "a button held across a mode reset does not survive"
        );

        term.push_output(b"\x1b[?1003h");
        assert_eq!(
            term.mouse_report(MouseAction::Motion, None, 0, 0, none)
                .expect("free motion"),
            b"\x1b[<35;1;1M"
        );
        let ctrl_shift = Modifiers {
            ctrl: true,
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(
            term.mouse_report(
                MouseAction::Press,
                Some(MouseButton::Right),
                0,
                0,
                ctrl_shift
            )
            .expect("modified press"),
            b"\x1b[<22;1;1M"
        );
    }

    #[test]
    fn legacy_mouse_encoding_has_x10_offsets_and_a_release_button() {
        let mut term = TerminalApp::new();
        term.push_output(b"\x1b[?1000h");
        assert_eq!(
            term.mouse_report(
                MouseAction::Press,
                Some(MouseButton::Left),
                0,
                0,
                Modifiers::NONE
            )
            .expect("press"),
            [0x1b, b'[', b'M', 32, 33, 33]
        );
        assert_eq!(
            term.mouse_report(
                MouseAction::Release,
                Some(MouseButton::Left),
                0,
                0,
                Modifiers::NONE
            )
            .expect("release"),
            [0x1b, b'[', b'M', 35, 33, 33]
        );
    }

    #[test]
    fn bracketed_paste_wraps_once_and_preserves_newlines_and_large_payloads() {
        let mut term = TerminalApp::new();
        let text = "first\nsecond\r\nตั\u{0e27}อย่าง";
        assert_eq!(term.paste_bytes(text), text.as_bytes());

        term.push_output(b"\x1b[?2004h");
        let wrapped = term.paste_bytes(text);
        assert_eq!(
            wrapped,
            format!("\x1b[200~{text}\x1b[201~").into_bytes(),
            "newlines are paste data, not Enter keys"
        );

        let large = "x".repeat(50_000);
        let wrapped = term.paste_bytes(&large);
        assert_eq!(wrapped.len(), large.len().saturating_add(12));
        assert!(wrapped.starts_with(b"\x1b[200~"));
        assert!(wrapped.ends_with(b"\x1b[201~"));
        assert!(wrapped
            .windows(large.len())
            .any(|window| window == large.as_bytes()));

        // A clipboard cannot be allowed to terminate its own paste early.
        let hostile = "safe\x1b[201~still-safe";
        let wrapped = term.paste_bytes(hostile);
        assert_eq!(wrapped, b"\x1b[200~safestill-safe\x1b[201~");
    }

    #[test]
    fn alternate_scroll_region_sequences_keep_outside_rows_still() {
        let mut term = TerminalApp::new();
        term.set_viewport(8, 5);
        term.push_output(b"\x1b[?1049h\x1b[1;1HA\x1b[2;1HB\x1b[3;1HC\x1b[4;1HD\x1b[5;1HE");
        term.push_output(b"\x1b[2;4r");
        let screen = term.screen().expect("alternate screen");
        assert_eq!(screen.scroll_region(), (1, 3));
        term.push_output(b"\x1b[4;1H\x1b[S");
        let screen = term.screen().expect("alternate screen");
        assert_eq!(screen.text_lines(), ["A", "C", "D", "", "E"]);
    }
}
