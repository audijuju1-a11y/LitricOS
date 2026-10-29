//! Finding the applications this machine has.
//!
//! A freedesktop desktop entry is a small INI file: a name, an argument vector,
//! an icon name and a handful of flags. This module reads the ones that are
//! real applications, validates them against the machine they are on, and
//! returns a list the launcher can offer. It never invents an entry and never
//! offers one whose program is missing.
//!
//! # Bounds
//!
//! Discovery is deliberately cheap and bounded: at most
//! [`MAX_FILES_PER_DIR`] entries are looked at per directory, a file larger
//! than [`MAX_FILE_BYTES`] is skipped without being read, and the result is
//! capped at [`MAX_APPS`]. The list is the only thing retained; file contents
//! are dropped as soon as one entry has been parsed.
//!
//! # What is refused
//!
//! * a `Type` other than `Application`;
//! * `NoDisplay=true`, `Hidden=true`, and terminal applications (this image
//!   hosts no terminal emulator to give them);
//! * an `Exec` that is not an absolute path to an executable file, after
//!   resolving a bare name against `PATH`;
//! * an argument vector containing shell metacharacters. A desktop entry is
//!   data, and the launcher passes it to `execve` as a vector, but refusing the
//!   characters outright means a file that was written as a shell command can
//!   never become one later.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The most directory entries inspected per applications directory.
pub const MAX_FILES_PER_DIR: usize = 512;

/// The largest desktop file read. A real entry is a few hundred bytes.
pub const MAX_FILE_BYTES: u64 = 64 * 1024;

/// The most applications offered.
pub const MAX_APPS: usize = 128;

/// One application found on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopApp {
    /// The desktop-file id: the file name without `.desktop`.
    pub id: String,
    /// The name a person reads, localised when the entry provides it.
    pub name: String,
    /// The argument vector, already split and validated.
    pub exec: Vec<String>,
    /// The entry's `Icon=` name.
    pub icon: Option<String>,
    /// The PNG the icon name resolved to on this machine, when it resolved.
    pub icon_path: Option<PathBuf>,
    /// The file this came from.
    pub source: PathBuf,
}

/// The directories a desktop entry may live in, in precedence order.
///
/// `$XDG_DATA_HOME/applications` (or `~/.local/share/applications`), then each
/// `$XDG_DATA_DIRS/applications` (default `/usr/local/share:/usr/share`). The
/// earlier directory wins when two entries share an id.
#[must_use]
pub fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    if let Some(data) = home {
        dirs.push(data.join("applications"));
    }
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_owned());
    for dir in data_dirs.split(':') {
        if dir.trim().is_empty() {
            continue;
        }
        dirs.push(PathBuf::from(dir).join("applications"));
    }
    dirs
}

/// Discover the applications in the machine's XDG directories, resolving each
/// entry's icon against the machine's icon themes.
#[must_use]
pub fn discover() -> Vec<DesktopApp> {
    discover_in_with(&data_dirs(), &crate::themeicons::theme_dirs())
}

/// Discover the applications in `dirs`, earlier directories taking precedence.
#[must_use]
pub fn discover_in(dirs: &[PathBuf]) -> Vec<DesktopApp> {
    discover_in_with(dirs, &[])
}

/// What one discovery read: the list it produced and how much work it did.
///
/// The counts are what makes an incremental reload provable rather than
/// claimed: a test can assert that a second pass read one directory instead of
/// four, and the shell keeps the last numbers as diagnostics.
#[derive(Debug, Clone, Default)]
pub struct Discovery {
    /// The applications, deduplicated by id, in display order.
    pub apps: Vec<DesktopApp>,
    /// Application directories actually read from disk.
    pub dirs_read: usize,
    /// Desktop files actually read from disk.
    pub files_read: usize,
    /// Distinct `Icon=` names this pass asked the theme tree for. A name shared
    /// by several entries is asked once, so this is not the entry count.
    pub icons_resolved: usize,
    /// Memo entries a change made this pass decide about: each was either kept
    /// (its providing directory had not moved) or re-resolved. Zero when the
    /// icon tree did not move, which is the common case.
    pub icons_validated: usize,
}

/// Discover the applications in `dirs` and resolve their icons in `icon_dirs`.
///
/// Resolution is a handful of `stat` calls per entry, done once here rather
/// than in a frame; a name that does not resolve leaves `icon_path` empty and
/// the launcher draws its generated glyph.
#[must_use]
pub fn discover_in_with(dirs: &[PathBuf], icon_dirs: &[PathBuf]) -> Vec<DesktopApp> {
    discover_tracked(dirs, icon_dirs).apps
}

/// Discover everything in `dirs`, reporting how much was read.
#[must_use]
pub fn discover_tracked(dirs: &[PathBuf], icon_dirs: &[PathBuf]) -> Discovery {
    let mut names = IconNames::new(icon_dirs);
    discover_with_cache(dirs, icon_dirs, &mut names)
}

/// Discover everything in `dirs` using `names` to memoise icon lookups.
#[must_use]
pub fn discover_with_cache(
    dirs: &[PathBuf],
    icon_dirs: &[PathBuf],
    names: &mut IconNames,
) -> Discovery {
    reload_with_cache(dirs, icon_dirs, &[], dirs, true, names)
}

/// Discover `dirs`, re-reading only `reread` and reusing `keep` for the rest.
///
/// `keep` is the previous list, whose entries remember the file they came from
/// (`DesktopApp::source`), so a directory that did not change contributes its
/// old entries instead of being parsed again. Precedence is the same as a full
/// discovery: the first directory that names an id owns it, whether that entry
/// was kept or read.
///
/// `resolve_icons` says whether the icon tree moved. When it did not, a kept
/// entry keeps the path it already resolved and only a newly read entry asks
/// the themes; when it did, every entry asks again, because a name that
/// resolved to one file a moment ago may resolve to another now.
#[must_use]
pub fn reload_tracked(
    dirs: &[PathBuf],
    icon_dirs: &[PathBuf],
    keep: &[DesktopApp],
    reread: &[PathBuf],
    resolve_icons: bool,
) -> Discovery {
    let mut names = IconNames::new(icon_dirs);
    reload_with_cache(dirs, icon_dirs, keep, reread, resolve_icons, &mut names)
}

/// As [`reload_tracked`], reusing `names` for the icon lookups.
///
/// The cache is valid for one state of the icon tree: if a directory under it
/// moved, every name is looked up again before it is trusted, so a name is never
/// carried across a theme change.
#[must_use]
pub fn reload_with_cache(
    dirs: &[PathBuf],
    icon_dirs: &[PathBuf],
    keep: &[DesktopApp],
    reread: &[PathBuf],
    resolve_icons: bool,
    names: &mut IconNames,
) -> Discovery {
    let resolved_before = names.resolved();
    let validated_before = names.validated();
    names.refresh(icon_dirs);
    let mut claimed: Vec<String> = Vec::new();
    let mut apps: Vec<DesktopApp> = Vec::new();
    let mut dirs_read = 0usize;
    let mut files_read = 0usize;
    'dirs: for dir in dirs {
        if !reread.iter().any(|read| read == dir) {
            // Not this directory's turn to be read: reuse what it produced last
            // time. Its files were validated then, so nothing here is trusted
            // that was not already checked.
            let mut kept: Vec<&DesktopApp> = keep
                .iter()
                .filter(|app| app.source.parent() == Some(dir.as_path()))
                .collect();
            kept.sort_by(|a, b| a.source.cmp(&b.source));
            for app in kept {
                if apps.len() >= MAX_APPS {
                    break 'dirs;
                }
                if claimed.iter().any(|seen| seen == &app.id) {
                    continue;
                }
                claimed.push(app.id.clone());
                apps.push(app.clone());
            }
            continue;
        }
        dirs_read = dirs_read.saturating_add(1);
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".desktop"))
            .collect();
        // Deterministic within a directory, so two runs on one machine cannot
        // show two different orders.
        names.sort();
        for name in names {
            if claimed.len() >= MAX_FILES_PER_DIR {
                break;
            }
            if apps.len() >= MAX_APPS {
                break 'dirs;
            }
            let id = name.trim_end_matches(".desktop").to_owned();
            // The first directory that names an id owns it, even when its own
            // file turns out to be unusable: that is what shadowing means.
            if claimed.iter().any(|seen| seen == &id) {
                continue;
            }
            claimed.push(id.clone());
            let path = dir.join(&name);
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            // A symlink is followed by `read` only when its target is a regular
            // file; a dangling one is skipped rather than reported as an app.
            if !meta.file_type().is_file() && !meta.file_type().is_symlink() {
                continue;
            }
            if meta.len() > MAX_FILE_BYTES {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            files_read = files_read.saturating_add(1);
            let text = String::from_utf8_lossy(&bytes);
            if let Some(app) = parse_entry(&text, path, id) {
                apps.push(app);
            }
        }
    }
    // One pass, once: a kept entry already has its path unless the icon tree
    // moved, and a new entry has none yet.
    if resolve_icons {
        for app in &mut apps {
            if let Some(icon) = app.icon.as_deref() {
                app.icon_path = names.resolve(icon_dirs, icon);
            }
        }
    } else {
        for app in &mut apps {
            if app.icon_path.is_none() {
                if let Some(icon) = app.icon.as_deref() {
                    app.icon_path = names.resolve(icon_dirs, icon);
                }
            }
        }
    }
    apps.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    apps.truncate(MAX_APPS);
    let icons_resolved =
        usize::try_from(names.resolved().saturating_sub(resolved_before)).unwrap_or(usize::MAX);
    let icons_validated =
        usize::try_from(names.validated().saturating_sub(validated_before)).unwrap_or(usize::MAX);
    Discovery {
        apps,
        dirs_read,
        files_read,
        icons_resolved,
        icons_validated,
    }
}

/// The most icon names one cache holds.
///
/// A machine offers at most [`MAX_APPS`] entries, so a load holds at most 128
/// names; 512 is headroom for a session that reloads while the list churns, and
/// the map is cleared rather than grown past it.
pub const MAX_ICON_NAMES: usize = 512;

/// The most leaf directories one theme contributes to the deep scan.
///
/// The leaves are the directories `themeicons::find_in` reads inside a theme:
/// its `scalable` directory and each numeric size directory, one level further
/// per category. A theme with eleven size directories would offer sixty leaves;
/// sixteen keeps the vector small while still covering `scalable` (where a
/// theme without rasters keeps every icon) and the sizes nearest the launcher's
/// 48 px lookup. Leaves past the cap are not watched, and that is a stated limit
/// rather than a silent one.
pub const MAX_LEAVES_PER_THEME: usize = 16;

/// The most leaf directories the deep scan tracks across every theme.
///
/// Four themes at the per-theme cap. Memory is one path pair and one stamp each,
/// counted by [`IconNames::bytes`].
pub const MAX_THEME_LEAVES: usize = 64;

/// The most leaves one deep scan inspects.
///
/// This is what bounds a check: a `read_dir` per leaf, so a theme whose `apps`
/// directory holds hundreds of icons costs the same as one that holds three, as
/// long as the slice is the same size. At the caps a full cycle over the tracked
/// leaves takes six checks, and a check happens at most once per
/// [`WATCH_INTERVAL_MS`].
pub const LEAVES_PER_SCAN: usize = 12;

/// The most resolved files one check re-stamps.
///
/// The leaf scan notices a file *appearing*; this notices the file behind a
/// name being *rewritten*, which the leaf cannot see because the entry count does
/// not move. A `metadata` call per file, not a read: 32 of them cost less than
/// one leaf `read_dir`. At the cap of [`MAX_ICON_NAMES`] names a full cycle takes
/// sixteen checks.
pub const FILES_PER_SCAN: usize = 32;

/// The directories `find_in_theme` reads inside one theme, deepest first.
///
/// # Why the theme's own stamp is not enough
///
/// A file added to `<theme>/scalable/apps` changes nothing the theme directory
/// reports: measured on this machine's filesystem, creating a file leaves the
/// parent directory's modification time byte-identical (`probe: mtime unchanged
/// after file create: true`), so only the entry count moves — and that count
/// belongs to the leaf, not to the theme. One level deeper is the cheapest place
/// a new icon is visible at all.
fn theme_leaves(theme: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(theme) else {
        return Vec::new();
    };
    let mut scalable: Option<PathBuf> = None;
    let mut sized: Vec<(u32, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "scalable" {
            scalable = Some(entry.path());
            continue;
        }
        let Some((width, height)) = name.split_once('x') else {
            continue;
        };
        let (Ok(width), Ok(height)) = (width.parse::<u32>(), height.parse::<u32>()) else {
            continue;
        };
        // Nearest to the launcher's own lookup size first, so the leaves most
        // likely to answer are the ones the cap keeps.
        sized.push((
            width.abs_diff(48).saturating_add(height.abs_diff(48)),
            entry.path(),
        ));
    }
    sized.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let mut leaves: Vec<PathBuf> = Vec::new();
    for base in scalable.iter().chain(sized.iter().map(|(_, path)| path)) {
        for category in crate::themeicons::CATEGORIES {
            if leaves.len() >= MAX_LEAVES_PER_THEME {
                return leaves;
            }
            // Pushed whether or not it exists yet: a category directory that
            // appears later is the same change as a file appearing in one that
            // is already there.
            leaves.push(base.join(category));
        }
    }
    leaves
}

/// What one resolved icon file looked like: its length and modification time.
///
/// Length and mtime together, not a hash: this runs on every watched check, and
/// a read per icon is the cost the image cache exists to avoid. Measured on this
/// machine, rewriting a file in place leaves **its own mtime** byte-identical
/// too, so length is what actually moves; the mtime is kept for filesystems that
/// do report it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    mtime: Option<std::time::SystemTime>,
}

/// The stamp of the file `path`, or `None` when it is not a regular file.
fn file_stamp(path: &Path) -> Option<FileStamp> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    Some(FileStamp {
        len: meta.len(),
        mtime: meta.modified().ok(),
    })
}

/// One memoised `Icon=` answer.
#[derive(Debug, Clone)]
struct Cached {
    /// The file the name resolved to, or `None` for a remembered miss.
    path: Option<PathBuf>,
    /// Index in [`IconNames::order`] of the directory that answered, or `None`
    /// for a miss (a miss is witnessed by the whole tree, not one directory).
    witness: Option<usize>,
    /// That directory's stamp when it answered.
    stamp: Option<Stamp>,
    /// The resolved file's own stamp when it answered. The image cache is keyed
    /// by path, so a file rewritten in place keeps the same path and would
    /// otherwise keep its old pixels.
    file: Option<FileStamp>,
}

/// `Icon=` names resolved against the icon tree, validated per name.
///
/// # Why it exists
///
/// §56 re-resolved every entry's name whenever the icon tree moved, and §57
/// memoised names but dropped the whole map on any move, so installing one new
/// theme re-resolved every name it already knew. This keeps, per name, *which
/// directory answered* and that directory's stamp, and invalidates only the
/// names a change can actually affect.
///
/// # The invariant a cached hit stands on
///
/// [`IconNames::order`] is the search order the lookup uses, conservatively:
/// every icon root's flat directory first (in `dirs` order), then each root's
/// theme directories, `hicolor` first and then by name. For a tree whose themes
/// do not inherit, that is exactly the order `themeicons::find_in` walks, so:
///
/// * a hit witnessed at index `k` is still the answer iff no directory at index
///   `<= k` changed and its file still exists — an earlier or equal directory is
///   the only thing that can shadow it, and a later one loses to it;
/// * a miss is still a miss iff nothing in the order changed at all;
/// * a directory appended at the end of the order shadows nothing, so a new
///   theme at the tail costs only the names it actually supplies.
///
/// `Inherits=` breaks that flat order (a parent theme is searched after every
/// direct theme, not where its name sorts), so a change in a directory that
/// declares inheritance clears the whole map instead. That is the conservative
/// direction: it re-resolves more than necessary, never less.
///
/// # Bounds
///
/// At most [`MAX_ICON_NAMES`] names and at most [`MAX_WATCH_DIRS`] directories
/// in the order; past either, the map is cleared rather than grown. Memory is
/// counted by [`IconNames::bytes`].
#[derive(Debug, Clone, Default)]
pub struct IconNames {
    dirs: Vec<PathBuf>,
    order: Vec<(PathBuf, Stamp)>,
    names: HashMap<String, Cached>,
    /// `(leaf directory, the theme it belongs to, its stamp when the list was
    /// built)`. Recorded when the order is built, not on the first scan: a file
    /// written between arming the cache and the first check would otherwise be
    /// recorded as part of the baseline and never reported.
    leaves: Vec<(PathBuf, PathBuf, Stamp)>,
    leaf_cursor: usize,
    leaf_scans: u64,
    leaves_moved: u64,
    /// Names in the order they were resolved, for the round-robin file scan.
    /// A `HashMap` has no order, and sorting it on every check would cost more
    /// than the `stat` calls this exists to make.
    slots: Vec<String>,
    file_cursor: usize,
    file_stats: u64,
    files_changed: u64,
    resolved: u64,
    hits: u64,
    validated: u64,
}

/// The directories a lookup consults, in the order it consults them, with their
/// stamps: each root's flat directory, then each root's themes.
fn icon_order(dirs: &[PathBuf]) -> Vec<(PathBuf, Stamp)> {
    let mut order: Vec<(PathBuf, Stamp)> = Vec::new();
    for dir in dirs {
        if order.len() >= MAX_WATCH_DIRS {
            break;
        }
        order.push((dir.clone(), stamp_of(dir)));
    }
    for dir in dirs {
        if order.len() >= MAX_WATCH_DIRS {
            break;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut themes: Vec<PathBuf> = entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.path())
            .collect();
        // The order `find_in` uses: `hicolor` first, then by name.
        themes.sort_by_key(|path| {
            usize::from(path.file_name().and_then(|name| name.to_str()) != Some("hicolor"))
        });
        for theme in themes {
            if order.len() >= MAX_WATCH_DIRS {
                break;
            }
            let stamp = stamp_of(&theme);
            order.push((theme, stamp));
        }
    }
    order
}

/// Whether a theme declares `Inherits=`, which the flat order cannot model.
fn declares_inherits(theme: &Path) -> bool {
    let path = theme.join("index.theme");
    let Ok(meta) = std::fs::metadata(&path) else {
        return false;
    };
    if !meta.is_file() || meta.len() > 64 * 1024 {
        return false;
    }
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    text.lines().any(|line| {
        line.trim()
            .strip_prefix("Inherits=")
            .is_some_and(|names| !names.trim().is_empty())
    })
}

impl IconNames {
    /// An empty cache armed for `icon_dirs` as they are now.
    #[must_use]
    pub fn new(icon_dirs: &[PathBuf]) -> Self {
        let mut cache = Self {
            dirs: icon_dirs.to_vec(),
            order: icon_order(icon_dirs),
            names: HashMap::new(),
            leaves: Vec::new(),
            leaf_cursor: 0,
            leaf_scans: 0,
            leaves_moved: 0,
            slots: Vec::new(),
            file_cursor: 0,
            file_stats: 0,
            files_changed: 0,
            resolved: 0,
            hits: 0,
            validated: 0,
        };
        cache.capture_leaves();
        cache
    }

    /// Re-stamp the tree and drop the names a change can affect.
    fn refresh(&mut self, icon_dirs: &[PathBuf]) {
        let fresh = icon_order(icon_dirs);
        // The order the old witnesses refer to is still intact up to the first
        // position whose directory changed: renamed, replaced, or simply no
        // longer the same directory. Everything from there on is treated as
        // changed, which is what makes an appended directory cheap (it shadows
        // nothing) and a removed one safe (the entries that pointed into it go).
        let common = self
            .order
            .iter()
            .zip(fresh.iter())
            .take_while(|(old, new)| old.0 == new.0)
            .count();
        let first_changed = if common == self.order.len() && common == fresh.len() {
            // Same directories in the same places: only a stamp can have moved.
            self.order
                .iter()
                .zip(fresh.iter())
                .position(|(old, new)| old.1 != new.1)
                .unwrap_or(usize::MAX)
        } else {
            common
        };
        if first_changed == usize::MAX {
            return;
        }
        if fresh
            .iter()
            .skip(first_changed)
            .any(|(dir, _)| declares_inherits(dir))
        {
            // An inherited theme can be searched before a theme whose name sorts
            // later, which this flat order cannot express. Dropping everything is
            // the safe direction.
            self.names.clear();
            self.slots.clear();
            self.file_cursor = 0;
            self.dirs = icon_dirs.to_vec();
            self.order = fresh;
            self.capture_leaves();
            return;
        }
        let mut kept: HashMap<String, Cached> = HashMap::with_capacity(self.names.len());
        for (name, entry) in self.names.drain() {
            self.validated = self.validated.saturating_add(1);
            let still_good = match entry.witness {
                // A hit: only an earlier or equal directory can shadow it.
                Some(index) => {
                    index < first_changed
                        && entry.stamp == self.order.get(index).map(|(_, stamp)| *stamp)
                        && entry.path.as_deref().is_some_and(Path::is_file)
                }
                // A miss: any change anywhere could have supplied the name.
                None => false,
            };
            if still_good {
                kept.insert(name, entry);
            }
        }
        self.names = kept;
        self.dirs = icon_dirs.to_vec();
        self.order = fresh;
        self.capture_leaves();
    }

    /// Rebuild the leaf list from the current order, stamping each leaf now.
    ///
    /// Called wherever the order is rebuilt, and once when a cache is armed. It
    /// costs one `read_dir` per leaf, bounded by [`MAX_THEME_LEAVES`], at load
    /// or on a theme change -- never in a frame and never per check.
    fn capture_leaves(&mut self) {
        self.leaves.clear();
        self.leaf_cursor = 0;
        for (dir, _) in &self.order {
            for leaf in theme_leaves(dir) {
                if self.leaves.len() >= MAX_THEME_LEAVES {
                    return;
                }
                let stamp = stamp_of(&leaf);
                self.leaves.push((leaf, dir.clone(), stamp));
            }
            if self.leaves.len() >= MAX_THEME_LEAVES {
                return;
            }
        }
    }

    /// Look at the next slice of theme leaf directories.
    ///
    /// Returns whether anything moved. When it did, every name witnessed in the
    /// theme that owns the changed leaf is dropped, and every remembered miss,
    /// because a name that did not resolve a moment ago may resolve now. Names
    /// witnessed in another theme are left alone: that is the per-name property
    /// §58 established, now fed by a signal one directory deeper.
    ///
    /// Bounded to [`LEAVES_PER_SCAN`] directories, so one check costs a fixed
    /// number of `read_dir` calls rather than one per icon on the machine.
    pub fn scan_leaves(&mut self) -> bool {
        if self.leaves.is_empty() {
            self.capture_leaves();
            return false;
        }
        let total = self.leaves.len();
        let count = total.min(LEAVES_PER_SCAN);
        let mut moved: Vec<PathBuf> = Vec::new();
        for _ in 0..count {
            // No modulo: a cursor that never leaves `0..total` cannot divide by
            // a zero total or trip the crate's arithmetic lint.
            let index = if self.leaf_cursor < total {
                self.leaf_cursor
            } else {
                0
            };
            self.leaf_cursor = index.saturating_add(1);
            if self.leaf_cursor >= total {
                self.leaf_cursor = 0;
            }
            self.leaf_scans = self.leaf_scans.saturating_add(1);
            let Some(leaf) = self.leaves.get_mut(index) else {
                continue;
            };
            let fresh = stamp_of(&leaf.0);
            let changed = leaf.2 != fresh;
            leaf.2 = fresh;
            if changed && !moved.iter().any(|theme| theme == &leaf.1) {
                moved.push(leaf.1.clone());
            }
        }
        if moved.is_empty() {
            return false;
        }
        self.leaves_moved = self.leaves_moved.saturating_add(moved.len() as u64);
        self.forget_themes(&moved);
        true
    }

    /// Drop every name a change in `themes` can affect: the hits witnessed there
    /// and every miss, which nothing witnesses.
    fn forget_themes(&mut self, themes: &[PathBuf]) {
        let mut kept: HashMap<String, Cached> = HashMap::with_capacity(self.names.len());
        for (name, entry) in self.names.drain() {
            self.validated = self.validated.saturating_add(1);
            let still_good = match entry.witness {
                Some(index) => self
                    .order
                    .get(index)
                    .is_some_and(|(dir, _)| !themes.iter().any(|theme| theme == dir)),
                // A miss is witnessed by the whole tree, not one directory.
                None => false,
            };
            if still_good {
                kept.insert(name, entry);
            }
        }
        self.names = kept;
    }

    /// Remember a name for the round-robin file scan, rebuilding the list from
    /// the map when it would grow past the map's own bound.
    fn remember_slot(&mut self, name: &str) {
        if self.slots.len() >= MAX_ICON_NAMES {
            let mut keys: Vec<String> = self.names.keys().cloned().collect();
            keys.sort();
            self.slots = keys;
            self.file_cursor = 0;
        }
        self.slots.push(name.to_owned());
    }

    /// Re-stamp the next slice of resolved files.
    ///
    /// Returns whether any of them changed. A file rewritten in place keeps its
    /// path, so the answer a name has is still the right path -- what is stale is
    /// the *pixels* cached against that path. The caller drops the decoded copies
    /// and repaints; nothing is re-resolved, because the search answer did not
    /// move.
    ///
    /// Bounded to [`FILES_PER_SCAN`] `metadata` calls, round-robin, so the cost
    /// of a check does not grow with the number of applications.
    pub fn scan_files(&mut self) -> bool {
        if self.slots.is_empty() {
            return false;
        }
        let total = self.slots.len();
        let count = total.min(FILES_PER_SCAN);
        let mut changed = false;
        for _ in 0..count {
            let index = if self.file_cursor < total {
                self.file_cursor
            } else {
                0
            };
            self.file_cursor = index.saturating_add(1);
            if self.file_cursor >= total {
                self.file_cursor = 0;
            }
            let Some(name) = self.slots.get(index).map(String::as_str) else {
                continue;
            };
            let Some(entry) = self.names.get_mut(name) else {
                // Dropped by a theme change since its slot was written: the slot
                // is stale and the next pass skips it.
                continue;
            };
            let Some(path) = entry.path.as_deref() else {
                // A remembered miss has no file to stamp.
                continue;
            };
            self.file_stats = self.file_stats.saturating_add(1);
            let fresh = file_stamp(path);
            if fresh != entry.file {
                entry.file = fresh;
                changed = true;
            }
        }
        if changed {
            self.files_changed = self.files_changed.saturating_add(1);
        }
        changed
    }

    /// Resolved files re-stamped, across the life of this cache.
    #[must_use]
    pub fn file_stats(&self) -> u64 {
        self.file_stats
    }

    /// Checks that found a rewritten file.
    #[must_use]
    pub fn files_changed(&self) -> u64 {
        self.files_changed
    }

    /// Names the file scan walks, at most [`MAX_ICON_NAMES`].
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// The file `name` resolves to under `icon_dirs`.
    ///
    /// Private on purpose: [`Self::refresh`] must have run for the current state
    /// of the tree first, and [`reload_with_cache`] is the one place that does
    /// both in that order. A public entry point here would be an API that is easy
    /// to call stale.
    fn resolve(&mut self, icon_dirs: &[PathBuf], name: &str) -> Option<PathBuf> {
        if let Some(found) = self.names.get(name) {
            self.hits = self.hits.saturating_add(1);
            return found.path.clone();
        }
        if self.names.len() >= MAX_ICON_NAMES {
            // A hard bound, not an eviction policy: the entries that follow will
            // resolve again, which is cheaper than tracking ages for a map this
            // small.
            self.names.clear();
            self.slots.clear();
            self.file_cursor = 0;
        }
        self.resolved = self.resolved.saturating_add(1);
        let path = crate::themeicons::find_in(icon_dirs, name, 48);
        // The witness is the deepest directory in the order that contains the
        // file: a theme directory for a themed icon, the root for a flat one.
        let witness = path.as_deref().and_then(|found| {
            self.order
                .iter()
                .enumerate()
                .filter(|(_, (dir, _))| found.starts_with(dir))
                .max_by_key(|(_, (dir, _))| dir.as_os_str().len())
                .map(|(index, _)| index)
        });
        let stamp = witness.and_then(|index| self.order.get(index).map(|(_, stamp)| *stamp));
        let file = path.as_deref().and_then(file_stamp);
        self.names.insert(
            name.to_owned(),
            Cached {
                path: path.clone(),
                witness,
                stamp,
                file,
            },
        );
        self.remember_slot(name);
        path
    }

    /// Leaf directories the deep scan has looked at, across the life of this
    /// cache.
    #[must_use]
    pub fn leaf_scans(&self) -> u64 {
        self.leaf_scans
    }

    /// Themes a deep scan has reported moved, counted once per theme per scan.
    #[must_use]
    pub fn leaves_moved(&self) -> u64 {
        self.leaves_moved
    }

    /// Leaf directories this cache tracks, at most [`MAX_THEME_LEAVES`].
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// Names whose file was looked up, across the life of this cache.
    #[must_use]
    pub fn resolved(&self) -> u64 {
        self.resolved
    }

    /// Lookups answered from the map without touching the theme tree.
    #[must_use]
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Memo entries examined during a validation pass: the ones a change made
    /// decide about, whether they survived or not.
    #[must_use]
    pub fn validated(&self) -> u64 {
        self.validated
    }

    /// Names the map holds now. For the diagnostics line and the tests.
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether the map holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// What the map holds, in bytes: the names, the paths they resolved to, and
    /// one header per entry. This is a lower bound — it does not count the
    /// allocator's own bookkeeping — and it is what the tests assert against.
    #[must_use]
    pub fn bytes(&self) -> usize {
        let mut total = self
            .dirs
            .iter()
            .chain(self.order.iter().map(|(dir, _)| dir))
            .map(|dir| dir.as_os_str().len())
            .fold(0usize, usize::saturating_add);
        for (name, entry) in &self.names {
            total = total
                .saturating_add(name.len())
                .saturating_add(entry.path.as_ref().map_or(0, |p| p.as_os_str().len()))
                .saturating_add(std::mem::size_of::<String>())
                .saturating_add(std::mem::size_of::<Cached>())
                .saturating_add(std::mem::size_of::<usize>().saturating_mul(2));
        }
        total = total.saturating_add(
            self.slots
                .iter()
                .map(|name| name.len().saturating_add(std::mem::size_of::<String>()))
                .fold(0usize, usize::saturating_add),
        );
        for (leaf, theme, _) in &self.leaves {
            total = total
                .saturating_add(leaf.as_os_str().len())
                .saturating_add(theme.as_os_str().len())
                .saturating_add(std::mem::size_of::<(PathBuf, PathBuf, Stamp)>());
        }
        total
    }
}

/// The shortest time between two looks for installed or removed applications.
///
/// One second is the top-bar clock's own resolution and the shell's idle wake
/// cap, so the check rides on a wake-up that was already happening and never
/// asks the frame loop to wake for it.
pub const WATCH_INTERVAL_MS: u64 = 1_000;

/// The most directories one stamp inspects.
///
/// One `applications` directory per XDG data root, the icon roots, and one
/// directory per theme under them: tens on a real machine, bounded here so a
/// machine with thousands of theme directories cannot turn a check into a scan.
pub const MAX_WATCH_DIRS: usize = 96;

/// The most directory entries one stamp counts.
///
/// The count stops here so a directory with a hundred thousand entries cannot
/// make a check unbounded. Beyond this many entries in one directory, only the
/// modification time is compared.
pub const MAX_WATCH_ENTRIES: usize = 512;

/// What one directory looked like: its modification time and, up to
/// [`MAX_WATCH_ENTRIES`], how many entries it holds.
///
/// The count is not redundant. This sandbox's filesystem — and a real one on a
/// coarse clock — reports a directory's modification time to the second, so an
/// entry created and looked for inside the same second would otherwise be
/// invisible until something else touched the directory. Adding or removing a
/// `.desktop` file or a theme directory changes the count immediately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stamp {
    mtime: Option<std::time::SystemTime>,
    entries: u32,
}

fn stamp_of(path: &Path) -> Stamp {
    let mtime = std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok());
    let entries = std::fs::read_dir(path).map_or(0, |iter| {
        iter.flatten()
            .take(MAX_WATCH_ENTRIES)
            .count()
            .try_into()
            .unwrap_or(u32::MAX)
    });
    Stamp { mtime, entries }
}

/// The directories whose modification time says whether the installed set
/// changed: the application directories and the themes under each icon root.
fn watch_paths(app_dirs: &[PathBuf], icon_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for dir in app_dirs.iter().chain(icon_dirs) {
        if !paths.iter().any(|seen| seen == dir) && paths.len() < MAX_WATCH_DIRS {
            paths.push(dir.clone());
        }
    }
    for root in icon_dirs {
        if paths.len() >= MAX_WATCH_DIRS {
            break;
        }
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        let mut themes: Vec<PathBuf> = entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.path())
            .collect();
        themes.sort();
        for theme in themes {
            if paths.len() >= MAX_WATCH_DIRS {
                break;
            }
            if !paths.iter().any(|seen| seen == &theme) {
                paths.push(theme);
            }
        }
    }
    paths
}

/// What moved between two looks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Change {
    /// Indices into the application directories [`DesktopWatch::dirs`] reports.
    pub app_dirs: Vec<usize>,
    /// Whether an icon root or a theme moved: every entry's icon must be
    /// resolved again.
    pub icons: bool,
}

/// A bounded watch for applications or icon themes installed or removed while
/// the shell is running.
///
/// # Why a stamp and not a subscription
///
/// The shell crate is `#![forbid(unsafe_code)]` and takes no dependencies, so
/// there is no `inotify` call here and no watcher crate to lean on. The honest
/// bounded version is what this is: the modification time of the directories
/// that would change when something is installed, removed or renamed, read at
/// most once per [`WATCH_INTERVAL_MS`] and compared with the last look. Nothing
/// is read at all until the interval has elapsed, and nothing is read when the
/// list was placed by hand (`Shell::load_desktop_apps`), so a fixture host is
/// never touched.
///
/// # What it notices
///
/// A `.desktop` file appearing, disappearing or being renamed in an
/// applications directory; a theme directory appearing or disappearing under an
/// icon root; and a theme gaining or losing a directory of its own. Adding a
/// single icon file inside an existing theme's `scalable/apps` does not change
/// any of those mtimes, and is not claimed.
#[derive(Debug, Clone)]
pub struct DesktopWatch {
    app_dirs: Vec<PathBuf>,
    icon_dirs: Vec<PathBuf>,
    paths: Vec<PathBuf>,
    /// How many leading `paths` are application directories.
    app_path_count: usize,
    seen: Vec<Stamp>,
    next_ms: u64,
    checks: u64,
    stats: u64,
    reloads: u64,
}

impl DesktopWatch {
    /// Arm the watch over `app_dirs` and `icon_dirs`, recording the tree as it
    /// is now.
    #[must_use]
    pub fn new(app_dirs: &[PathBuf], icon_dirs: &[PathBuf], now_ms: u64) -> Self {
        let paths = watch_paths(app_dirs, icon_dirs);
        let app_path_count = paths
            .iter()
            .take_while(|path| app_dirs.iter().any(|dir| dir == *path))
            .count();
        let seen: Vec<Stamp> = paths.iter().map(|path| stamp_of(path)).collect();
        Self {
            app_dirs: app_dirs.to_vec(),
            icon_dirs: icon_dirs.to_vec(),
            paths,
            app_path_count,
            seen,
            next_ms: now_ms.saturating_add(WATCH_INTERVAL_MS),
            checks: 0,
            stats: 0,
            reloads: 0,
        }
    }

    /// What changed since the last look, or `None`.
    ///
    /// `None` before the interval has elapsed, without touching the disk: the
    /// frame loop may call this every frame and pay one comparison. When
    /// something did move, the answer says which application directories it
    /// was, so the caller can re-read those and reuse the rest.
    pub fn poll(&mut self, now_ms: u64) -> Option<Change> {
        if now_ms < self.next_ms {
            return None;
        }
        self.next_ms = now_ms.saturating_add(WATCH_INTERVAL_MS);
        self.checks = self.checks.saturating_add(1);
        let mut change = Change::default();
        for (index, path) in self.paths.iter().enumerate() {
            self.stats = self.stats.saturating_add(1);
            let fresh = stamp_of(path);
            if self.seen.get(index).copied() == Some(fresh) {
                continue;
            }
            if let Some(slot) = self.seen.get_mut(index) {
                *slot = fresh;
            }
            if index < self.app_path_count {
                change.app_dirs.push(index);
            } else {
                change.icons = true;
            }
        }
        if change.app_dirs.is_empty() && !change.icons {
            return None;
        }
        self.reloads = self.reloads.saturating_add(1);
        Some(change)
    }

    /// Whether the interval has elapsed, without touching the disk.
    ///
    /// The shell asks this before it does any work: a check that only stamps
    /// `.desktop` directories must not run when the deep leaf scan is going to
    /// run on the same tick either.
    #[must_use]
    pub fn due(&self, now_ms: u64) -> bool {
        now_ms >= self.next_ms
    }

    /// The directories this watch was armed with.
    #[must_use]
    pub fn dirs(&self) -> (&[PathBuf], &[PathBuf]) {
        (&self.app_dirs, &self.icon_dirs)
    }

    /// Directories one stamp inspects. For the diagnostics line and the tests.
    #[must_use]
    pub fn paths(&self) -> usize {
        self.paths.len()
    }

    /// Checks that reached the disk.
    #[must_use]
    pub fn checks(&self) -> u64 {
        self.checks
    }

    /// `stat` calls this watch has made.
    #[must_use]
    pub fn stats(&self) -> u64 {
        self.stats
    }

    /// Times the tree changed.
    #[must_use]
    pub fn reloads(&self) -> u64 {
        self.reloads
    }
}

/// Parse one desktop entry.
///
/// `source` is kept for diagnostics and `id` is the desktop-file id the caller
/// already deduplicated on. `None` means the file is not an application this
/// machine can run.
#[must_use]
pub fn parse_entry(text: &str, source: PathBuf, id: String) -> Option<DesktopApp> {
    let mut in_group = false;
    let mut values: HashMap<String, String> = HashMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_group = line.eq_ignore_ascii_case("[Desktop Entry]");
            continue;
        }
        if !in_group {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        // The first spelling wins: later duplicates are how a file grows two
        // values for one key, and picking the last would make the result depend
        // on where the duplicate happened to be appended.
        values
            .entry(key.trim().to_owned())
            .or_insert_with(|| value.trim().to_owned());
    }

    if values.get("Type").map(String::as_str) != Some("Application") {
        return None;
    }
    if truthy(values.get("NoDisplay")) || truthy(values.get("Hidden")) {
        return None;
    }
    if truthy(values.get("Terminal")) {
        // This image has no terminal emulator to attach, so offering the app
        // would be offering a window that cannot exist.
        return None;
    }
    if let Some(try_exec) = values.get("TryExec") {
        resolve_program(try_exec)?;
    }
    let locale = locale_prefix();
    let name = locale
        .as_ref()
        .and_then(|lang| values.get(&format!("Name[{lang}]")))
        .or_else(|| values.get("Name"))
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())?;
    let exec = parse_exec(values.get("Exec")?)?;
    Some(DesktopApp {
        id,
        name,
        exec,
        icon: values.get("Icon").cloned(),
        icon_path: None,
        source,
    })
}

/// Whether an INI boolean is on. Only the two spellings the specification
/// defines count; anything else is not a yes.
fn truthy(value: Option<&String>) -> bool {
    value.is_some_and(|v| v.eq_ignore_ascii_case("true") || v == "1")
}

/// The language part of the user's locale, when it has one.
fn locale_prefix() -> Option<String> {
    let raw = std::env::var("LANGUAGE")
        .ok()
        .and_then(|v| v.split(':').next().map(str::to_owned))
        .or_else(|| std::env::var("LC_ALL").ok())
        .or_else(|| std::env::var("LANG").ok())?;
    let language = raw
        .split(['.', '@'])
        .next()
        .unwrap_or(&raw)
        .split('_')
        .next()
        .unwrap_or(&raw)
        .trim()
        .to_owned();
    if language.is_empty() || language.eq_ignore_ascii_case("C") || language == "POSIX" {
        return None;
    }
    Some(language)
}

/// Split an `Exec` value into an argument vector, as the specification says:
/// whitespace separates arguments, double quotes group, and a backslash escapes
/// the next character.
fn tokenize(text: &str) -> Option<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut any = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let next = chars.next()?;
                current.push(next);
                any = true;
            }
            '"' => {
                quoted = !quoted;
                any = true;
            }
            ' ' | '\t' if !quoted => {
                if any {
                    out.push(std::mem::take(&mut current));
                    any = false;
                }
            }
            _ => {
                current.push(c);
                any = true;
            }
        }
    }
    if quoted {
        return None;
    }
    if any {
        out.push(current);
    }
    Some(out)
}

/// Replace the `%` field codes the specification defines, refusing an unknown
/// one rather than passing it to the program.
fn expand_field_codes(token: &str) -> Option<String> {
    let mut out = String::with_capacity(token.len());
    let mut chars = token.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            '%' => out.push('%'),
            // A file, URL, icon, name or location the launcher would have to
            // supply. This launcher supplies none of them, so the code is
            // removed: passing `%f` through would give the program a literal
            // percent sign as an argument.
            'f' | 'F' | 'u' | 'U' | 'd' | 'D' | 'n' | 'N' | 'i' | 'c' | 'k' | 'v' | 'm' => {}
            _ => return None,
        }
    }
    Some(out)
}

/// Parse and validate an `Exec` value into the vector the daemon will run.
fn parse_exec(text: &str) -> Option<Vec<String>> {
    let tokens = tokenize(text)?;
    let mut argv: Vec<String> = Vec::with_capacity(tokens.len());
    for token in tokens {
        let value = expand_field_codes(&token)?;
        if value.is_empty() {
            continue;
        }
        // A shell would interpret these. The launcher has no shell, and the
        // refusal means a file written as a command cannot become one if some
        // later path ever reintroduces one.
        if value
            .chars()
            .any(|c| matches!(c, '|' | '&' | ';' | '<' | '>' | '`' | '\n' | '\r'))
            || value.contains("$(")
        {
            return None;
        }
        argv.push(value);
    }
    let program = argv.first()?;
    let resolved = resolve_program(program)?;
    *argv.first_mut()? = resolved.to_string_lossy().into_owned();
    Some(argv)
}

/// Resolve a program name to an absolute executable path.
///
/// A name containing a slash must already be absolute: resolving a relative
/// path would make the meaning depend on the daemon's working directory.
fn resolve_program(program: &str) -> Option<PathBuf> {
    let path = Path::new(program);
    if program.contains('/') {
        if !path.is_absolute() {
            return None;
        }
        return litric_sysd::launch::is_executable(path).then(|| path.to_path_buf());
    }
    let search = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&search) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(program);
        if litric_sysd::launch::is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-desktop-entry-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[cfg(unix)]
    fn program(dir: &Path, name: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("write");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        path
    }

    #[test]
    fn a_plain_entry_parses_with_its_vector_and_name() {
        let dir = scratch("plain");
        #[cfg(unix)]
        let prog = program(&dir, "run-me");
        #[cfg(not(unix))]
        let prog = PathBuf::from("/bin/sh");
        let text = format!(
            "[Desktop Entry]\nType=Application\nName=Marker App\nExec={} --flag\nIcon=marker\n",
            prog.display()
        );
        let app = parse_entry(&text, dir.join("marker.desktop"), "marker".to_owned())
            .expect("a real application");
        assert_eq!(app.name, "Marker App");
        assert_eq!(
            app.exec.first().map(String::as_str),
            Some(prog.to_str().unwrap_or(""))
        );
        assert_eq!(app.exec.get(1).map(String::as_str), Some("--flag"));
        assert_eq!(app.icon.as_deref(), Some("marker"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn field_codes_and_quotes_are_removed_leaving_literal_arguments() {
        let dir = scratch("codes");
        let prog = program(&dir, "run-me");
        let text = format!(
            "[Desktop Entry]\nType=Application\nName=Codes\nExec={} %U %f \"two words\" 100%% a%%b\n",
            prog.display()
        );
        let app = parse_entry(&text, dir.join("codes.desktop"), "codes".to_owned()).expect("app");
        assert_eq!(
            app.exec.get(1).map(String::as_str),
            Some("two words"),
            "{:?}",
            app.exec
        );
        assert_eq!(app.exec.get(2).map(String::as_str), Some("100%"));
        // `a%b` names no field code, so the entry is refused rather than
        // passing `a%b` to the program.
        assert!(parse_entry(
            &format!(
                "[Desktop Entry]\nType=Application\nName=Bad\nExec={} a%b\n",
                prog.display()
            ),
            dir.join("bad.desktop"),
            "bad".to_owned()
        )
        .is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn entries_that_are_not_runnable_applications_are_refused() {
        let dir = scratch("refuse");
        let prog = program(&dir, "run-me");
        for text in [
            format!(
                "[Desktop Entry]\nType=Link\nName=L\nExec={}\n",
                prog.display()
            ),
            format!(
                "[Desktop Entry]\nType=Application\nName=H\nHidden=true\nExec={}\n",
                prog.display()
            ),
            format!(
                "[Desktop Entry]\nType=Application\nName=N\nNoDisplay=true\nExec={}\n",
                prog.display()
            ),
            format!(
                "[Desktop Entry]\nType=Application\nName=T\nTerminal=true\nExec={}\n",
                prog.display()
            ),
            "[Desktop Entry]\nType=Application\nName=M\nExec=/nonexistent/program\n".to_owned(),
            "[Desktop Entry]\nType=Application\nName=R\nExec=relative-program\n".to_owned(),
            format!(
                "[Desktop Entry]\nType=Application\nName=S\nExec={} a;b\n",
                prog.display()
            ),
            "[Desktop Entry]\nType=Application\nName=X\nTryExec=/nonexistent/try\nExec=/bin/sh\n"
                .to_owned(),
        ] {
            assert!(
                parse_entry(&text, dir.join("x.desktop"), "x".to_owned()).is_none(),
                "accepted: {text}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn discovery_prefers_the_earlier_directory_and_lists_real_apps_only() {
        use std::os::unix::fs::PermissionsExt;
        let root = scratch("discover");
        let home = root.join("home/applications");
        let system = root.join("system/applications");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&system).expect("system");
        let prog = program(&root, "program");
        let plain = root.join("plain");
        std::fs::write(&plain, b"not executable").expect("plain");
        std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).expect("mode");
        std::fs::write(
            home.join("zeta.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Zeta Marker\nExec={}\n",
                prog.display()
            ),
        )
        .expect("zeta");
        std::fs::write(
            home.join("missing.desktop"),
            "[Desktop Entry]\nType=Application\nName=Missing\nExec=/nonexistent/x\n",
        )
        .expect("missing");
        std::fs::write(
            home.join("plain.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Plain\nExec={}\n",
                plain.display()
            ),
        )
        .expect("plain entry");
        // The same id in the system directory loses to the home directory.
        std::fs::write(
            system.join("zeta.desktop"),
            "[Desktop Entry]\nType=Application\nName=Wrong Zeta\nExec=/bin/true\n",
        )
        .expect("shadowed");
        std::fs::write(
            system.join("alpha.desktop"),
            "[Desktop Entry]\nType=Application\nName=Alpha\nExec=/bin/true\n",
        )
        .expect("alpha");

        let apps = discover_in(&[home, system]);
        let names: Vec<&str> = apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "Zeta Marker"], "{names:?}");
        assert!(
            apps.iter().all(|a| !a.name.contains("Missing")),
            "an app whose program is absent must not be offered"
        );
        assert!(
            apps.iter().all(|a| !a.name.contains("Plain")),
            "a non-executable program must not be offered"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_oversized_entry_is_not_read() {
        let root = scratch("large");
        let big = root.join("big.desktop");
        std::fs::write(&big, vec![b'a'; (MAX_FILE_BYTES as usize) + 1]).expect("write");
        assert!(discover_in(std::slice::from_ref(&root)).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A theme directory added at the tail of the search order supplies one new
    /// name; the names already answered are validated, not resolved again.
    ///
    /// This is the §58 property: §57 dropped the whole map on any move, so this
    /// pass would report three names resolved for three entries instead of one.
    #[cfg(unix)]
    #[test]
    fn a_new_theme_directory_re_resolves_only_the_name_it_supplies() {
        let root = scratch("icons-per-name");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(icons.join("hicolor/scalable/apps")).expect("theme");
        std::fs::write(
            icons.join("hicolor/scalable/apps/blue-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#2080c0\"/></svg>",
        )
        .expect("icon");
        let prog = program(&root, "run-me");
        for name in ["Alpha", "Beta", "Gamma"] {
            std::fs::write(
                apps.join(format!("{name}.desktop")),
                format!(
                    "[Desktop Entry]\nType=Application\nName={name}\nIcon=blue-icon\nExec={}\n",
                    prog.display()
                ),
            )
            .expect("entry");
        }
        let icon_dirs = vec![icons.clone()];
        let app_dirs = vec![apps.clone()];
        let mut names = IconNames::new(&icon_dirs);
        let first = discover_with_cache(&app_dirs, &icon_dirs, &mut names);
        assert_eq!(first.apps.len(), 3);
        assert_eq!(first.icons_resolved, 1);
        assert_eq!(
            first.icons_validated, 0,
            "nothing moves on the first pass, so nothing is validated"
        );

        // One new theme directory and one new name, written while the map is warm.
        let extra = icons.join("zzz-extra/scalable/apps");
        std::fs::create_dir_all(&extra).expect("extra theme");
        std::fs::write(
            extra.join("late-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><circle cx=\"4\" cy=\"4\" r=\"3\" fill=\"#fa be 28\"/></svg>",
        )
        .expect("late icon");
        std::fs::write(
            apps.join("Late.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Late\nIcon=late-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("late entry");

        let after = reload_with_cache(
            &app_dirs,
            &icon_dirs,
            &first.apps,
            &app_dirs,
            true,
            &mut names,
        );
        assert_eq!(after.apps.len(), 4);
        assert_eq!(
            after.icons_resolved, 1,
            "only the new name is new work: {} names resolved",
            after.icons_resolved
        );
        assert_eq!(
            after.icons_validated, 1,
            "the one name already answered is decided about once, not re-resolved"
        );
        assert_eq!(names.len(), 2);
        assert!(after.apps.iter().all(|app| app.icon_path.is_some()));
        println!(
            "per-name cache: {} names, {} bytes, entry {} bytes",
            names.len(),
            names.bytes(),
            std::mem::size_of::<Cached>()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A name whose file appears in a directory consulted earlier resolves to the
    /// new file; the cached hit is not served.
    #[cfg(unix)]
    #[test]
    fn a_name_that_moves_to_an_earlier_directory_is_re_resolved() {
        let root = scratch("icons-moved");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(icons.join("hicolor/scalable/apps")).expect("theme");
        let old = icons.join("hicolor/scalable/apps/move-icon.svg");
        std::fs::write(
            &old,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#c02020\"/></svg>",
        )
        .expect("old icon");
        let prog = program(&root, "run-me");
        std::fs::write(
            apps.join("mover.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Mover\nIcon=move-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("entry");
        let icon_dirs = vec![icons.clone()];
        let app_dirs = vec![apps.clone()];
        let mut names = IconNames::new(&icon_dirs);
        let first = discover_with_cache(&app_dirs, &icon_dirs, &mut names);
        assert_eq!(
            first.apps.first().and_then(|app| app.icon_path.as_deref()),
            Some(old.as_path()),
            "the theme answers first"
        );

        // A flat file at the icon root, which the lookup consults before any theme.
        let flat = icons.join("move-icon.svg");
        std::fs::write(
            &flat,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#20a020\"/></svg>",
        )
        .expect("flat icon");
        let after = reload_with_cache(&app_dirs, &icon_dirs, &first.apps, &[], true, &mut names);
        assert_eq!(
            after.apps.first().and_then(|app| app.icon_path.as_deref()),
            Some(flat.as_path()),
            "the earlier directory now shadows the theme"
        );
        assert_eq!(after.icons_resolved, 1, "the cached hit was not trusted");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Removing the theme directory that answered a name drops the name rather
    /// than leaving a path to a file that is gone.
    #[cfg(unix)]
    #[test]
    fn removing_a_theme_directory_drops_the_names_it_answered() {
        let root = scratch("icons-removed");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(icons.join("zzz-extra/scalable/apps")).expect("theme");
        std::fs::write(
            icons.join("zzz-extra/scalable/apps/only-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#2080c0\"/></svg>",
        )
        .expect("icon");
        let prog = program(&root, "run-me");
        std::fs::write(
            apps.join("only.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Only\nIcon=only-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("entry");
        let icon_dirs = vec![icons.clone()];
        let app_dirs = vec![apps.clone()];
        let mut names = IconNames::new(&icon_dirs);
        let first = discover_with_cache(&app_dirs, &icon_dirs, &mut names);
        assert!(first
            .apps
            .first()
            .is_some_and(|app| app.icon_path.is_some()));

        std::fs::remove_dir_all(icons.join("zzz-extra")).expect("uninstall");
        let after = reload_with_cache(&app_dirs, &icon_dirs, &first.apps, &[], true, &mut names);
        let gone = after.apps.first().map(|app| app.icon_path.clone());
        assert!(
            gone.as_ref().is_some_and(Option::is_none),
            "a cached hit into a removed directory must not survive: {gone:?}"
        );
        assert_eq!(after.icons_resolved, 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A theme that declares `Inherits=` cannot be modelled by the flat order, so
    /// a change in it clears the map instead of keeping names that might now be
    /// shadowed by a parent.
    #[cfg(unix)]
    #[test]
    fn a_change_in_an_inheriting_theme_re_resolves_everything() {
        let root = scratch("icons-inherits");
        let apps = root.join("applications");
        let icons = root.join("icons");
        let theme = icons.join("hicolor/scalable/apps");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(&theme).expect("theme");
        std::fs::write(
            icons.join("hicolor/index.theme"),
            "[Icon Theme]\nName=Hicolor\nInherits=Adwaita,gnome\n",
        )
        .expect("index.theme");
        std::fs::write(
            theme.join("blue-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#2080c0\"/></svg>",
        )
        .expect("icon");
        let prog = program(&root, "run-me");
        for (name, icon) in [("One", "blue-icon"), ("Two", "red-icon")] {
            if icon == "red-icon" {
                std::fs::write(
                    theme.join("red-icon.svg"),
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#c02020\"/></svg>",
                )
                .expect("red icon");
            }
            std::fs::write(
                apps.join(format!("{name}.desktop")),
                format!(
                    "[Desktop Entry]\nType=Application\nName={name}\nIcon={icon}\nExec={}\n",
                    prog.display()
                ),
            )
            .expect("entry");
        }
        let icon_dirs = vec![icons.clone()];
        let app_dirs = vec![apps.clone()];
        let mut names = IconNames::new(&icon_dirs);
        let first = discover_with_cache(&app_dirs, &icon_dirs, &mut names);
        assert_eq!(first.icons_resolved, 2, "two entries, two names");

        // The theme directory itself moves: a file arrives next to `index.theme`.
        std::fs::write(
            icons.join("hicolor/extra.txt"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#c02020\"/></svg>",
        )
        .expect("second icon");
        let after = reload_with_cache(&app_dirs, &icon_dirs, &first.apps, &[], true, &mut names);
        assert_eq!(
            after.icons_resolved, 2,
            "an inheriting theme's change falls back to resolving every name"
        );
        assert_eq!(
            after.icons_validated, 0,
            "the fallback does not pretend to validate anything"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// What a pass costs when nothing moved: the common case, every frame that
    /// is not after an install.
    ///
    /// The number is printed rather than asserted, so a slow machine does not
    /// fail the suite; `§58` in the tracker records what it reported here.
    #[cfg(unix)]
    #[test]
    fn a_pass_over_an_unchanged_tree_is_cheap() {
        let root = scratch("icons-cost");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(icons.join("hicolor/scalable/apps")).expect("theme");
        let prog = program(&root, "run-me");
        for index in 0..16 {
            std::fs::write(
                apps.join(format!("app-{index}.desktop")),
                format!(
                    "[Desktop Entry]\nType=Application\nName=App {index}\nIcon=icon-{}\nExec={}\n",
                    index % 4,
                    prog.display()
                ),
            )
            .expect("entry");
        }
        for index in 0..4 {
            std::fs::write(
                icons.join(format!("hicolor/scalable/apps/icon-{index}.svg")),
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#2080c0\"/></svg>",
            )
            .expect("icon");
        }
        let icon_dirs = vec![icons.clone()];
        let app_dirs = vec![apps.clone()];
        let mut names = IconNames::new(&icon_dirs);
        let first = discover_with_cache(&app_dirs, &icon_dirs, &mut names);
        assert_eq!(first.apps.len(), 16);
        assert_eq!(first.icons_resolved, 4, "four names, sixteen entries");

        let passes = 2_000u32;
        let started = std::time::Instant::now();
        for _ in 0..passes {
            let again =
                reload_with_cache(&app_dirs, &icon_dirs, &first.apps, &[], false, &mut names);
            assert_eq!(again.icons_resolved, 0);
            assert_eq!(again.icons_validated, 0);
        }
        let each = started.elapsed() / passes;
        println!(
            "unchanged pass: {each:?} each over {} passes, {} names, {} bytes",
            passes,
            names.len(),
            names.bytes()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_bounds_are_pinned() {
        assert_eq!(MAX_APPS, 128);
        assert_eq!(MAX_FILES_PER_DIR, 512);
        assert_eq!(MAX_FILE_BYTES, 64 * 1024);
    }

    #[test]
    fn the_watch_reads_nothing_before_its_interval() {
        let root = scratch("watch-idle");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(&icons).expect("icons");
        let mut watch = DesktopWatch::new(
            std::slice::from_ref(&apps),
            std::slice::from_ref(&icons),
            10,
        );
        assert!(watch.paths() >= 2, "both directories are watched");
        assert!(
            watch.poll(10 + WATCH_INTERVAL_MS - 1).is_none(),
            "inside the interval"
        );
        assert_eq!(watch.stats(), 0, "an early poll must not touch the disk");
        assert_eq!(watch.checks(), 0);
        assert!(
            watch.poll(10 + WATCH_INTERVAL_MS).is_none(),
            "nothing changed"
        );
        assert_eq!(
            watch.stats(),
            u64::try_from(watch.paths()).unwrap_or(u64::MAX)
        );
        assert!(
            watch.poll(10 + WATCH_INTERVAL_MS + 1).is_none(),
            "the interval restarts"
        );
        assert_eq!(
            watch.stats(),
            u64::try_from(watch.paths()).unwrap_or(u64::MAX),
            "one batch per second, not per call"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn an_installed_entry_and_a_new_theme_are_both_a_change() {
        let root = scratch("watch-install");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(&icons).expect("icons");
        let mut watch =
            DesktopWatch::new(std::slice::from_ref(&apps), std::slice::from_ref(&icons), 0);
        assert!(
            watch.poll(WATCH_INTERVAL_MS).is_none(),
            "the tree as armed is not a change"
        );

        let prog = program(&root, "run-me");
        std::fs::write(
            apps.join("late.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Late App\nExec={}\nIcon=late-icon\n",
                prog.display()
            ),
        )
        .expect("entry");
        let change = watch
            .poll(WATCH_INTERVAL_MS * 2)
            .expect("a new .desktop must be noticed");
        assert_eq!(change.app_dirs, vec![0], "the applications directory moved");
        assert!(!change.icons, "no icon directory moved");
        assert_eq!(watch.reloads(), 1);

        // The reload the shell performs is the same discovery: the entry is
        // really there, with the name it declared.
        let found = discover_in(std::slice::from_ref(&apps));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found.first().map(|a| a.name.as_str()), Some("Late App"));

        // A theme installed afterwards is a change through the icon root.
        let mut watch =
            DesktopWatch::new(std::slice::from_ref(&apps), std::slice::from_ref(&icons), 0);
        std::fs::create_dir_all(icons.join("brand-new-theme/scalable/apps")).expect("theme");
        let change = watch
            .poll(WATCH_INTERVAL_MS)
            .expect("a new theme directory must be noticed");
        assert!(change.icons, "an icon directory moved");
        assert!(
            change.app_dirs.is_empty(),
            "no applications directory moved"
        );

        // And removing the entry is a change too.
        std::fs::remove_file(apps.join("late.desktop")).expect("remove");
        assert!(
            watch.poll(WATCH_INTERVAL_MS * 2).is_some(),
            "a removed entry"
        );
        assert!(discover_in(std::slice::from_ref(&apps)).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn entries_sharing_one_icon_name_resolve_it_once() {
        let root = scratch("icons-shared");
        let apps = root.join("applications");
        let icons = root.join("icons/hicolor/scalable/apps");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(&icons).expect("theme");
        std::fs::write(
            icons.join("shared-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#2080c0\"/></svg>",
        )
        .expect("icon");
        let prog = program(&root, "run-me");
        for name in ["Alpha", "Beta", "Gamma"] {
            std::fs::write(
                apps.join(format!("{name}.desktop")),
                format!(
                    "[Desktop Entry]\nType=Application\nName={name}\nIcon=shared-icon\nExec={}\n",
                    prog.display()
                ),
            )
            .expect("entry");
        }
        let icon_dirs = vec![root.join("icons")];
        let mut names = IconNames::new(&icon_dirs);
        let first = discover_with_cache(std::slice::from_ref(&apps), &icon_dirs, &mut names);
        assert_eq!(first.apps.len(), 3);
        assert_eq!(
            first.icons_resolved, 1,
            "three entries, one name: the theme tree is asked once"
        );
        assert!(first.apps.iter().all(|app| app.icon_path.is_some()));
        assert_eq!(names.len(), 1);
        assert_eq!(names.hits(), 2, "the other two entries were memo hits");
        println!(
            "icon name cache: {} names, {} bytes",
            names.len(),
            names.bytes()
        );

        // A second pass with the same tree resolves nothing at all.
        let again = reload_with_cache(
            std::slice::from_ref(&apps),
            &icon_dirs,
            &first.apps,
            &[],
            true,
            &mut names,
        );
        assert_eq!(again.icons_resolved, 0, "every name was already known");
        assert_eq!(again.apps.len(), 3);
        assert!(names.bytes() <= MAX_ICON_NAMES * (255 + 512 + 96));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn a_theme_change_does_not_reuse_a_negative_lookup() {
        let root = scratch("icons-stale");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(&icons).expect("icons");
        let prog = program(&root, "run-me");
        std::fs::write(
            apps.join("late.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Late\nIcon=late-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("entry");
        let icon_dirs = std::slice::from_ref(&icons);
        let mut names = IconNames::new(icon_dirs);
        let first = discover_with_cache(std::slice::from_ref(&apps), icon_dirs, &mut names);
        assert_eq!(first.icons_resolved, 1);
        assert!(
            first
                .apps
                .first()
                .and_then(|app| app.icon_path.as_ref())
                .is_none(),
            "the theme does not exist yet"
        );

        // The theme arrives: the icon tree stamp moves.
        let theme = icons.join("hicolor/scalable/apps");
        std::fs::create_dir_all(&theme).expect("theme");
        std::fs::write(
            theme.join("late-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#2080c0\"/></svg>",
        )
        .expect("icon");
        let after = reload_with_cache(
            std::slice::from_ref(&apps),
            icon_dirs,
            &first.apps,
            &[],
            true,
            &mut names,
        );
        assert_eq!(
            after.icons_resolved, 1,
            "the cached miss must not survive the theme change"
        );
        let path = after
            .apps
            .first()
            .and_then(|app| app.icon_path.as_ref())
            .expect("the new theme resolved the name");
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("svg"));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A file written into an existing theme's `scalable/apps` is noticed by the
    /// bounded deep scan, and only the names that theme answered are forgotten.
    ///
    /// A theme directory's own stamp cannot see this: creating a file in a
    /// directory leaves that directory's modification time byte-identical on
    /// this filesystem (measured), so §58's theme-level stamp is blind to it.
    /// The leaf stamp is one directory deeper and sees the entry count move.
    #[cfg(unix)]
    #[test]
    fn a_new_icon_in_an_existing_theme_is_noticed_by_the_deep_scan() {
        let root = scratch("icons-deep-leaf");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(icons.join("hicolor/scalable/apps")).expect("theme");
        std::fs::create_dir_all(icons.join("zzz-other/scalable/apps")).expect("other theme");
        std::fs::write(
            icons.join("zzz-other/scalable/apps/known-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#20a040\"/></svg>",
        )
        .expect("known icon");
        let prog = program(&root, "run-me");
        std::fs::write(
            apps.join("Known.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Known\nIcon=known-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("known entry");
        std::fs::write(
            apps.join("Late.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Late\nIcon=late-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("late entry");

        let app_dirs = vec![apps.clone()];
        let icon_dirs = vec![icons.clone()];
        let mut names = IconNames::new(&icon_dirs);
        assert!(
            names.leaf_count() >= 10,
            "both themes' leaves are tracked: {}",
            names.leaf_count()
        );
        let first = discover_with_cache(&app_dirs, &icon_dirs, &mut names);
        assert_eq!(first.icons_resolved, 2, "one hit and one miss");
        assert!(!names.scan_leaves(), "nothing moved yet");
        assert_eq!(
            names.leaf_scans(),
            names.leaf_count().min(LEAVES_PER_SCAN) as u64,
            "one scan is one slice of the tracked leaves"
        );

        // The icon arrives in hicolor, the theme searched first.
        std::fs::write(
            icons.join("hicolor/scalable/apps/late-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><circle cx=\"4\" cy=\"4\" r=\"3\" fill=\"#f0b020\"/></svg>",
        )
        .expect("late icon");
        assert!(
            names.scan_leaves(),
            "a file written into an existing theme is a change the theme stamp cannot see"
        );
        assert_eq!(
            names.leaves_moved(),
            1,
            "one theme moved, not one name per entry"
        );

        // Only the miss (nothing witnesses it) and names witnessed in hicolor are
        // dropped: the hit answered by zzz-other survives.
        let after = reload_with_cache(
            &app_dirs,
            &icon_dirs,
            &first.apps,
            &app_dirs,
            true,
            &mut names,
        );
        assert_eq!(
            after.icons_resolved, 1,
            "the miss resolved once and the hit was kept: {} names resolved",
            after.icons_resolved
        );
        assert!(after.apps.iter().all(|app| app.icon_path.is_some()));
        let late = after
            .apps
            .iter()
            .find(|app| app.name == "Late")
            .expect("the late entry");
        let path = late.icon_path.as_ref().expect("resolved");
        assert!(
            path.ends_with("hicolor/scalable/apps/late-icon.svg"),
            "{}",
            path.display()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// One scan inspects a fixed slice, however many leaves a theme offers.
    #[test]
    fn a_deep_scan_inspects_only_its_budget() {
        let root = scratch("icons-deep-bound");
        let icons = root.join("icons");
        for size in [
            "8x8", "16x16", "22x22", "24x24", "32x32", "48x48", "64x64", "96x96", "256x256",
            "512x512",
        ] {
            std::fs::create_dir_all(icons.join(format!("hicolor/{size}/apps"))).expect("size");
        }
        std::fs::create_dir_all(icons.join("hicolor/scalable/apps")).expect("scalable");
        let icon_dirs = vec![icons.clone()];
        let mut names = IconNames::new(&icon_dirs);
        assert_eq!(
            names.leaf_count(),
            MAX_LEAVES_PER_THEME,
            "a theme is capped at its own budget"
        );
        assert!(LEAVES_PER_SCAN < names.leaf_count(), "the budget must bite");
        let _ = names.scan_leaves();
        assert_eq!(
            names.leaf_scans(),
            LEAVES_PER_SCAN as u64,
            "one scan is one slice"
        );

        // The measured cost, on a leaf holding as many icons as a large theme's
        // `apps` directory does. The number in §59 comes from this run.
        let leaf = icons.join("hicolor/scalable/apps");
        for index in 0..200 {
            std::fs::write(leaf.join(format!("fill-{index}.svg")), b"<svg/>").expect("fill");
        }
        let started = std::time::Instant::now();
        let rounds = 200u32;
        for _ in 0..rounds {
            let _ = names.scan_leaves();
        }
        let per_scan = started.elapsed().as_secs_f64() * 1e6 / f64::from(rounds);
        println!(
            "deep scan: {per_scan:.1} us/scan over {} tracked leaves (one holding 200 files), cache {} bytes, {} entries in the order",
            names.leaf_count(),
            names.bytes(),
            names.len()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A file rewritten in place is noticed by the file scan, and the name keeps
    /// its answer: only the pixels were stale.
    ///
    /// The leaf scan cannot see this. Measured on this machine, rewriting a file
    /// leaves the directory's entry count *and* the file's own mtime
    /// byte-identical; only the length moves, which is what this compares.
    #[cfg(unix)]
    #[test]
    fn a_rewritten_icon_file_is_noticed_by_the_file_scan() {
        let root = scratch("icons-rewrite");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(icons.join("hicolor/scalable/apps")).expect("theme");
        let icon = icons.join("hicolor/scalable/apps/swap-icon.svg");
        std::fs::write(&icon, "<svg/>").expect("icon");
        let prog = program(&root, "run-me");
        std::fs::write(
            apps.join("Swap.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=Swap\nIcon=swap-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("entry");

        let app_dirs = vec![apps.clone()];
        let icon_dirs = vec![icons.clone()];
        let mut names = IconNames::new(&icon_dirs);
        let first = discover_with_cache(&app_dirs, &icon_dirs, &mut names);
        assert_eq!(first.apps.len(), 1);
        assert_eq!(names.slot_count(), 1, "one name to re-stat");
        assert!(!names.scan_files(), "the file has not moved yet");
        assert_eq!(names.file_stats(), 1, "one metadata call, not a read");
        assert!(!names.scan_files(), "and still has not");

        // A package upgrade rewrites the same path with different artwork.
        std::fs::write(
            &icon,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#c02020\"/></svg>",
        )
        .expect("rewrite");
        assert!(
            names.scan_files(),
            "a file rewritten in place must be noticed: the leaf count and the mtimes did not move"
        );
        assert_eq!(names.files_changed(), 1);
        assert!(!names.scan_files(), "the new stamp is the baseline now");

        // The answer is unchanged, so nothing is re-resolved: only the pixels
        // cached against the path are stale.
        let after = reload_with_cache(
            &app_dirs,
            &icon_dirs,
            &first.apps,
            &app_dirs,
            true,
            &mut names,
        );
        assert_eq!(
            after.icons_resolved, 0,
            "the search answer did not move, so no name is looked up again: {}",
            after.icons_resolved
        );
        assert_eq!(
            after.apps.first().and_then(|app| app.icon_path.as_ref()),
            Some(&icon)
        );
        println!(
            "file scan: {} names, {} bytes, cache entry {} bytes",
            names.slot_count(),
            names.bytes(),
            std::mem::size_of::<Cached>()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The file scan inspects a fixed slice per check, whatever the map holds.
    #[cfg(unix)]
    #[test]
    fn a_file_scan_inspects_only_its_budget() {
        let root = scratch("icons-file-bound");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(icons.join("hicolor/scalable/apps")).expect("theme");
        let prog = program(&root, "run-me");
        let wanted = 40usize;
        for index in 0..wanted {
            std::fs::write(
                icons.join(format!("hicolor/scalable/apps/icon-{index}.svg")),
                "<svg/>",
            )
            .expect("icon");
            std::fs::write(
                apps.join(format!("app-{index}.desktop")),
                format!(
                    "[Desktop Entry]\nType=Application\nName=App {index:02}\nIcon=icon-{index}\nExec={}\n",
                    prog.display()
                ),
            )
            .expect("entry");
        }
        let icon_dirs = vec![icons];
        let mut names = IconNames::new(&icon_dirs);
        let found = discover_with_cache(std::slice::from_ref(&apps), &icon_dirs, &mut names);
        assert_eq!(found.apps.len(), wanted);
        assert_eq!(names.slot_count(), wanted, "one slot per resolved name");
        assert!(FILES_PER_SCAN < wanted, "the budget must bite");
        let _ = names.scan_files();
        assert_eq!(
            names.file_stats(),
            FILES_PER_SCAN as u64,
            "one check re-stats one slice"
        );

        // Measured cost: 200 checks over the full map.
        let started = std::time::Instant::now();
        let rounds = 200u32;
        for _ in 0..rounds {
            let _ = names.scan_files();
        }
        let per_scan = started.elapsed().as_secs_f64() * 1e6 / f64::from(rounds);
        println!(
            "file scan: {per_scan:.1} us/check over {} slots ({FILES_PER_SCAN} metadata calls), cache {} bytes",
            names.slot_count(),
            names.bytes()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn an_incremental_reload_reads_only_the_changed_directory() {
        let root = scratch("watch-incremental");
        let first = root.join("home/applications");
        let second = root.join("system/applications");
        std::fs::create_dir_all(&first).expect("home");
        std::fs::create_dir_all(&second).expect("system");
        let prog = program(&root, "run-me");
        let entry = |name: &str| {
            format!(
                "[Desktop Entry]\nType=Application\nName={name}\nExec={}\n",
                prog.display()
            )
        };
        std::fs::write(first.join("one.desktop"), entry("One")).expect("one");
        std::fs::write(second.join("two.desktop"), entry("Two")).expect("two");
        let dirs = [first.clone(), second.clone()];
        let full = discover_tracked(&dirs, &[]);
        assert_eq!(full.apps.len(), 2);
        assert_eq!(full.dirs_read, 2);
        assert_eq!(full.files_read, 2);

        let mut watch = DesktopWatch::new(&dirs, &[], 0);
        assert!(watch.poll(WATCH_INTERVAL_MS).is_none(), "armed as it is");
        // A new entry in the second directory only.
        std::fs::write(second.join("three.desktop"), entry("Three")).expect("three");
        let change = watch
            .poll(WATCH_INTERVAL_MS * 2)
            .expect("the second directory moved");
        assert_eq!(change.app_dirs, vec![1], "only the second directory moved");
        assert!(!change.icons);
        let reread: Vec<PathBuf> = change
            .app_dirs
            .iter()
            .filter_map(|index| dirs.get(*index).cloned())
            .collect();
        let increment = reload_tracked(&dirs, &[], &full.apps, &reread, false);
        assert_eq!(increment.dirs_read, 1, "one directory, not two");
        assert_eq!(
            increment.files_read, 2,
            "only the changed directory's files were parsed"
        );
        let names: Vec<&str> = increment.apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["One", "Three", "Two"]);
        assert!(
            increment
                .apps
                .iter()
                .any(|a| a.name == "One" && a.source.parent() == Some(first.as_path())),
            "the untouched directory kept its own entry"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn an_icon_only_change_reads_no_desktop_file_at_all() {
        let root = scratch("watch-icons");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        std::fs::create_dir_all(&icons).expect("icons");
        let prog = program(&root, "run-me");
        std::fs::write(
            apps.join("one.desktop"),
            format!(
                "[Desktop Entry]\nType=Application\nName=One\nIcon=one-icon\nExec={}\n",
                prog.display()
            ),
        )
        .expect("one");
        let full = discover_tracked(std::slice::from_ref(&apps), std::slice::from_ref(&icons));
        assert_eq!(full.apps.len(), 1);
        assert!(
            full.apps
                .first()
                .and_then(|a| a.icon_path.as_ref())
                .is_none(),
            "the theme does not exist yet"
        );

        // Armed before the theme exists, exactly like a running shell.
        let mut watch =
            DesktopWatch::new(std::slice::from_ref(&apps), std::slice::from_ref(&icons), 0);
        let theme = icons.join("hicolor/scalable/apps");
        std::fs::create_dir_all(&theme).expect("theme");
        std::fs::write(
            theme.join("one-icon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect x=\"0\" y=\"0\" width=\"8\" height=\"8\" fill=\"#2080c0\"/></svg>",
        )
        .expect("icon");

        let change = watch.poll(WATCH_INTERVAL_MS).expect("the icon tree moved");
        assert!(change.icons);
        assert!(
            change.app_dirs.is_empty(),
            "no applications directory moved"
        );
        let increment = reload_tracked(
            std::slice::from_ref(&apps),
            std::slice::from_ref(&icons),
            &full.apps,
            &[],
            true,
        );
        assert_eq!(increment.dirs_read, 0, "no applications directory was read");
        assert_eq!(increment.files_read, 0, "no desktop file was read");
        assert!(
            increment
                .apps
                .first()
                .and_then(|a| a.icon_path.as_ref())
                .is_some(),
            "the new theme resolved for the entry that was already known"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn one_check_costs_a_bounded_number_of_directory_reads() {
        let root = scratch("watch-cost");
        let apps = root.join("applications");
        std::fs::create_dir_all(&apps).expect("apps");
        // A real applications directory: a few hundred entries.
        for index in 0..200 {
            std::fs::write(apps.join(format!("entry-{index:03}.desktop")), b"x").expect("entry");
        }
        let mut watch =
            DesktopWatch::new(std::slice::from_ref(&apps), std::slice::from_ref(&root), 0);
        let start = std::time::Instant::now();
        let runs = 20u64;
        for step in 0..runs {
            assert!(
                watch
                    .poll(WATCH_INTERVAL_MS.saturating_mul(step + 1))
                    .is_none(),
                "nothing changed"
            );
        }
        let per = start.elapsed().as_secs_f64() * 1000.0 / runs as f64;
        println!(
            "one watch check: {per:.3} ms over {} directories ({} stat calls)",
            watch.paths(),
            watch.stats()
        );
        assert!(per < 25.0, "a check took {per:.1} ms");
        assert_eq!(
            watch.stats(),
            u64::try_from(watch.paths())
                .unwrap_or(0)
                .saturating_mul(runs)
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_watched_paths_are_bounded() {
        let root = scratch("watch-bound");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        for index in 0..(MAX_WATCH_DIRS + 40) {
            std::fs::create_dir_all(icons.join(format!("theme-{index:03}"))).expect("theme");
        }
        let watch = DesktopWatch::new(std::slice::from_ref(&apps), std::slice::from_ref(&icons), 0);
        assert_eq!(watch.paths(), MAX_WATCH_DIRS, "the cap holds");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_directory_is_not_a_change_every_second() {
        let root = scratch("watch-missing");
        let apps = root.join("applications");
        let icons = root.join("icons");
        std::fs::create_dir_all(&apps).expect("apps");
        // No icon root at all: `None` on both sides is stable, not churn.
        let mut watch =
            DesktopWatch::new(std::slice::from_ref(&apps), std::slice::from_ref(&icons), 0);
        assert!(watch.poll(WATCH_INTERVAL_MS).is_none());
        assert!(watch.poll(WATCH_INTERVAL_MS * 2).is_none());
        assert_eq!(watch.reloads(), 0, "absence is not a change on its own");
        let _ = std::fs::remove_dir_all(&root);
    }
}
