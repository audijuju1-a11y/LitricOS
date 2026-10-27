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
use litric_defender::verdict::Decision;

/// A text file opened in the viewer, with its own scroll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileViewer {
    /// The logical path being shown.
    pub path: String,
    /// The text (lossy-decoded, size-capped by the bridge).
    pub text: String,
    /// First visible line.
    pub scroll: usize,
}

impl FileViewer {
    /// The visible lines, given how many fit. Returns owned lines so the
    /// window can lay them out without borrowing the model.
    #[must_use]
    pub fn window(&self, rows: usize) -> (Vec<String>, usize) {
        let lines: Vec<String> = self.text.lines().map(String::from).collect();
        let total = lines.len();
        let start = self.scroll.min(total.saturating_sub(1));
        let end = start.saturating_add(rows).min(total);
        (lines.get(start..end).unwrap_or(&[]).to_vec(), total)
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
    pub const DEFAULT_PLACES: [(&str, &str); 6] = [
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
        let selected = if self.selected_rows.is_empty() {
            self.selected_name
                .clone()
                .or_else(|| {
                    self.selected
                        .and_then(|index| rows.get(index).map(|row| row.name.clone()))
                })
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
        } else {
            self.selected_rows.clone()
        };
        let base = self.path.trim_end_matches('/');
        rows.into_iter()
            .enumerate()
            .filter(|(_, row)| selected.contains(&row.name))
            .map(|(_, row)| {
                if base.is_empty() {
                    format!("/{}", row.name)
                } else {
                    format!("{base}/{}", row.name)
                }
            })
            .collect()
    }

    /// Whether a visible row is part of the multi-selection.
    #[must_use]
    pub fn row_is_selected(&self, index: usize) -> bool {
        self.visible().get(index).is_some_and(|row| {
            self.selected_rows.contains(&row.name)
                || (self.selected_rows.is_empty()
                    && (self.selected_name.as_deref() == Some(row.name.as_str())
                        || (self.selected_name.is_none() && self.selected == Some(index))))
        })
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
        let url = if raw.contains("://") || raw.starts_with("litric://") {
            raw
        } else if raw.contains('.') && !raw.contains(' ') {
            format!("https://{raw}")
        } else {
            self.engine.search_url(&raw)
        };
        self.navigate_active_to(&url);
        Some(url)
    }

    /// Navigate the selected tab and append a real history entry.
    pub fn navigate_active_to(&mut self, url: &str) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            if tab
                .history
                .get(tab.history_index)
                .is_none_or(|current| current != url)
            {
                tab.history.truncate(tab.history_index.saturating_add(1));
                tab.history.push(url.to_owned());
                if tab.history.len() > 100 {
                    tab.history.remove(0);
                }
                tab.history_index = tab.history.len().saturating_sub(1);
            }
            tab.url = url.to_owned();
            tab.error = false;
            tab.title = url.to_owned();
        }
        self.record_visit(url);
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

/// The Defender window model. Scanning itself happens in
/// [`crate::defender`]; this holds what the window shows.
#[derive(Debug, Clone, Default)]
pub struct DefenderApp {
    /// The state of the current or last scan.
    pub state: ScanState,
    /// One row per scanned file that matters (detections always; a bounded
    /// sample of clean files while scanning).
    pub rows: Vec<ThreatRow>,
    /// What the status line says.
    pub message: String,
    /// Definitions loaded, for the status line.
    pub definitions: usize,
    /// Whether the last scan found anything (drives the top-bar badge).
    pub last_had_threats: bool,
}

impl DefenderApp {
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
        let (detected, verdict) = match decision {
            Decision::Allow => (false, "Clean".to_owned()),
            Decision::AllowUnknown { reason } => (false, format!("Unknown ({reason})")),
            Decision::Deny { tier, id } => (true, format!("DETECTED [{tier:?}] {id}")),
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
    /// A breadcrumb segment.
    Crumb(usize),
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
}

/// Map a Files hotspot onto the model. Navigation returns the path to load,
/// or `None` when the action only changed local state.
#[must_use]
pub fn files_action(app: &mut FilesApp, hot: FilesHot) -> Option<String> {
    match hot {
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
    /// Scan the home folders with the Defender.
    ScanNow,
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
                MenuItem::Item("Scan now", MenuAction::ScanNow),
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
        };
        Self {
            at,
            surface,
            tab: None,
            items,
        }
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

/// The Terminal application model: a command line the guest OS answers.
///
/// Commands run for real — `/bin/sh` on the guest — through a bounded
/// thread whose output arrives as lines. The model is deliberately dumb:
/// a scrollback of strings, one input line, a busy flag. The shell owns
/// the process side, because the process side is the part that must
/// never block the frame loop.
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
        }
    }

    /// Append one output line, dropping the oldest past the ceiling.
    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push_back(line.into());
        while self.lines.len() > Self::MAX_LINES {
            self.lines.pop_front();
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
        self.lines.clear();
        self.input.clear();
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
        let path = files_action(&mut f, FilesHot::Place(1)).expect("navigates");
        assert_eq!(path, "/Pictures");
        assert_eq!(f.place_index, Some(1));
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
        assert_eq!(b.back().as_deref(), Some("https://one.example"));
        assert_eq!(b.forward().as_deref(), Some("https://two.example"));
        assert_eq!(b.back().as_deref(), Some("https://one.example"));
        b.navigate_active_to("https://three.example");
        assert!(
            !b.can_go_forward(),
            "a new route drops the old forward branch"
        );
        assert_eq!(
            b.history.front().map(String::as_str),
            Some("https://three.example")
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
            Some("https://example.com")
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
        assert_eq!(b.commit_omnibox().as_deref(), Some("https://example.com"));
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
