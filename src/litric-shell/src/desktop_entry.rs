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
}
