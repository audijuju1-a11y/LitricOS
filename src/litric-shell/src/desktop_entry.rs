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

/// Discover the applications in `dirs` and resolve their icons in `icon_dirs`.
///
/// Resolution is a handful of `stat` calls per entry, done once here rather
/// than in a frame; a name that does not resolve leaves `icon_path` empty and
/// the launcher draws its generated glyph.
#[must_use]
pub fn discover_in_with(dirs: &[PathBuf], icon_dirs: &[PathBuf]) -> Vec<DesktopApp> {
    let mut claimed: Vec<String> = Vec::new();
    let mut apps: Vec<DesktopApp> = Vec::new();
    'dirs: for dir in dirs {
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
            let text = String::from_utf8_lossy(&bytes);
            if let Some(mut app) = parse_entry(&text, path, id) {
                if let Some(icon) = app.icon.as_deref() {
                    app.icon_path = crate::themeicons::find_in(icon_dirs, icon, 48);
                }
                apps.push(app);
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
    apps
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
        let seen: Vec<Stamp> = paths.iter().map(|path| stamp_of(path)).collect();
        Self {
            app_dirs: app_dirs.to_vec(),
            icon_dirs: icon_dirs.to_vec(),
            paths,
            seen,
            next_ms: now_ms.saturating_add(WATCH_INTERVAL_MS),
            checks: 0,
            stats: 0,
            reloads: 0,
        }
    }

    /// Whether the tree changed since the last look.
    ///
    /// `false` before the interval has elapsed, without touching the disk: the
    /// frame loop may call this every frame and pay one comparison.
    pub fn poll(&mut self, now_ms: u64) -> bool {
        if now_ms < self.next_ms {
            return false;
        }
        self.next_ms = now_ms.saturating_add(WATCH_INTERVAL_MS);
        self.checks = self.checks.saturating_add(1);
        let mut changed = false;
        for (index, path) in self.paths.iter().enumerate() {
            self.stats = self.stats.saturating_add(1);
            let fresh = stamp_of(path);
            if self.seen.get(index).copied() != Some(fresh) {
                changed = true;
                if let Some(slot) = self.seen.get_mut(index) {
                    *slot = fresh;
                }
            }
        }
        if changed {
            self.reloads = self.reloads.saturating_add(1);
        }
        changed
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
            !watch.poll(10 + WATCH_INTERVAL_MS - 1),
            "inside the interval"
        );
        assert_eq!(watch.stats(), 0, "an early poll must not touch the disk");
        assert_eq!(watch.checks(), 0);
        assert!(!watch.poll(10 + WATCH_INTERVAL_MS), "nothing changed");
        assert_eq!(
            watch.stats(),
            u64::try_from(watch.paths()).unwrap_or(u64::MAX)
        );
        assert!(
            !watch.poll(10 + WATCH_INTERVAL_MS + 1),
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
            !watch.poll(WATCH_INTERVAL_MS),
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
        assert!(
            watch.poll(WATCH_INTERVAL_MS * 2),
            "a new .desktop must be noticed"
        );
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
        assert!(
            watch.poll(WATCH_INTERVAL_MS),
            "a new theme directory must be noticed"
        );

        // And removing the entry is a change too.
        std::fs::remove_file(apps.join("late.desktop")).expect("remove");
        assert!(watch.poll(WATCH_INTERVAL_MS * 2), "a removed entry");
        assert!(discover_in(std::slice::from_ref(&apps)).is_empty());
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
                !watch.poll(WATCH_INTERVAL_MS.saturating_mul(step + 1)),
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
        assert!(!watch.poll(WATCH_INTERVAL_MS));
        assert!(!watch.poll(WATCH_INTERVAL_MS * 2));
        assert_eq!(watch.reloads(), 0, "absence is not a change on its own");
        let _ = std::fs::remove_dir_all(&root);
    }
}
