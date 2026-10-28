//! The shell's preference file: one schema, one parser, one atomic write.
//!
//! Before this module the record was parsed and written inline in `main.rs`.
//! That worked for four keys and stopped working the moment the file had to
//! answer questions the code did not: what happens to a key from a newer build,
//! what happens when the file is half-written, which values are in scope for a
//! migration, and where the format is documented. The gate matrix names the gap
//! directly — "no settings store, no schema, no persistence path" — and this is
//! the smallest store that closes it.
//!
//! # The format
//!
//! Line-oriented `key=value`, UTF-8, `#` starts a comment. `version` names the
//! schema. It is deliberately boring: a person can read and repair it with
//! `cat` and an editor while the desktop is broken, which no binary format
//! offers and the recovery story (ADR-0021) asks for.
//!
//! # Why parsing never fails
//!
//! A preference file is read at startup by the process that draws the screen. A
//! parser that can fail there can leave a user with no desktop because one line
//! is wrong, so every failure mode keeps a default instead:
//!
//! * a key this build does not know is ignored (forward compatibility);
//! * a value this build cannot parse keeps the caller's default for that key;
//! * a line without `=` is skipped;
//! * a `version` newer than [`VERSION`] still parses every known key — the file
//!   is a record of values, not a serialized data structure.
//!
//! Reading the file itself can fail, and that error is returned rather than
//! swallowed: "no file" and "cannot read the file" are different states, and
//! only the first one means this account has never been set up.
//!
//! # Why the write is atomic
//!
//! Write to a temporary file in the same directory, `fsync`, `rename`, `fsync`
//! the directory. A power cut leaves either the previous record or the new one,
//! never a truncated file that parses to defaults and silently resets a user's
//! choices. The directory is `0700` and the file `0600`: preferences are not
//! secrets, but they are the user's, and on a multi-account machine the next
//! account has no business reading them.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use litric_design::{Accent, ThemeKind};

use crate::desktop::Wallpaper;
use crate::i18n::Language;
use crate::settings::{Preferences, UiScale};

/// The schema version this build writes.
///
/// Bump it when a key changes meaning, not when one is added: a new key parses
/// to a default in older builds, while a changed meaning needs a migration
/// before the old value is trusted.
pub const VERSION: u8 = 1;

/// Parse a record, starting from `defaults`.
///
/// `defaults` is not a fallback for the whole file: it supplies each value the
/// file does not set, so a two-key file (the 2026-10-15 format) still produces a
/// complete record.
#[must_use]
pub fn parse(text: &str, defaults: Preferences) -> Preferences {
    let mut preferences = defaults;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            // Read and validated, not acted on: the version decides whether a
            // migration runs, and there is nothing to migrate while this is the
            // only version. Parsing it here is what makes a future migration
            // possible without a second parser.
            "version" => {}
            "theme" => {
                if let Some(theme) = ThemeKind::all()
                    .iter()
                    .copied()
                    .find(|theme| theme.token() == value)
                {
                    preferences.theme = theme;
                }
            }
            "accent" => {
                if let Some(accent) = Accent::parse(value) {
                    preferences.accent = accent;
                }
            }
            "wallpaper" => {
                if let Some(wallpaper) = Wallpaper::parse(value) {
                    preferences.wallpaper = wallpaper;
                }
            }
            "language" => {
                if let Some(language) = Language::parse(value) {
                    preferences.language = language;
                }
            }
            "reduce_motion" => {
                if let Some(enabled) = parse_bool(value) {
                    preferences.reduce_motion = enabled;
                }
            }
            "ui_scale" => {
                if let Some(scale) = UiScale::parse(value) {
                    preferences.ui_scale = scale;
                }
            }
            "theme_schedule" => {
                if let Some(schedule) = crate::settings::ThemeSchedule::parse(value) {
                    preferences.theme_schedule = schedule;
                }
            }
            "onboarded" => {
                if let Some(done) = parse_bool(value) {
                    preferences.onboarded = done;
                }
            }
            "search_engine" => {
                if let Some(engine) = crate::apps::SearchEngine::parse(value) {
                    preferences.search_engine = engine;
                }
            }
            _ => {}
        }
    }
    preferences
}

/// `1`/`true`/`yes` and `0`/`false`/`no`, or `None` for anything else.
fn parse_bool(raw: &str) -> Option<bool> {
    match raw {
        "1" | "true" | "yes" => Some(true),
        "0" | "false" | "no" => Some(false),
        _ => None,
    }
}

/// Serialize a record: one `key=value` per line, version first, newline at the
/// end so an appended line by hand still parses.
#[must_use]
pub fn encode(preferences: Preferences) -> String {
    let mut out = String::new();
    out.push_str("version=");
    out.push_str(&VERSION.to_string());
    out.push('\n');
    out.push_str("theme=");
    out.push_str(preferences.theme.token());
    out.push('\n');
    out.push_str("accent=");
    out.push_str(preferences.accent.token());
    out.push('\n');
    out.push_str("wallpaper=");
    out.push_str(preferences.wallpaper.token());
    out.push('\n');
    out.push_str("language=");
    out.push_str(preferences.language.token());
    out.push('\n');
    out.push_str("reduce_motion=");
    out.push_str(if preferences.reduce_motion { "1" } else { "0" });
    out.push('\n');
    out.push_str("ui_scale=");
    out.push_str(preferences.ui_scale.token());
    out.push('\n');
    out.push_str("theme_schedule=");
    out.push_str(preferences.theme_schedule.token());
    out.push('\n');
    out.push_str("onboarded=");
    out.push_str(if preferences.onboarded { "1" } else { "0" });
    out.push('\n');
    out.push_str("search_engine=");
    out.push_str(preferences.search_engine.token());
    out.push('\n');
    out
}

/// A preference file at a known path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    /// A store for `path`. Nothing is read or created until [`Self::load`] or
    /// [`Self::save`] is called.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The file this store owns.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the record. `Ok(None)` means the file does not exist, which is the
    /// one state that means "never set up". Any other read failure is returned.
    ///
    /// A record that exists is treated as finished setup unless it says
    /// otherwise: the flag was added after the file, so a file without it came
    /// from a setup that finished.
    pub fn load(&self) -> Result<Option<Preferences>, String> {
        // Bytes, then a lossy decode: a corrupted file must produce a desktop
        // with defaults, not a startup error. `read_to_string` refuses invalid
        // UTF-8 outright, which would have turned a truncated write into a
        // desktop that never appears.
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!("cannot read {}: {error}", self.path.display()));
            }
        };
        let text = String::from_utf8_lossy(&bytes);
        // The loader's default record is a finished setup, so a file that does
        // not mention the flag keeps it: the flag was added after the file.
        let defaults = Preferences::default();
        Ok(Some(parse(&text, defaults)))
    }

    /// Write the record atomically, creating and securing the directory.
    pub fn save(&self, preferences: Preferences) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "settings path has no parent directory".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create settings directory: {error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("cannot secure settings directory: {error}"))?;
        }

        let temp = parent.join(format!("settings.conf.tmp.{}", std::process::id()));
        // A stale temporary from a build that was killed between write and
        // rename must not make this one fail forever.
        let _ = fs::remove_file(&temp);

        let result = (|| -> Result<(), String> {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temp)
                .map_err(|error| format!("cannot create temporary settings file: {error}"))?;
            file.write_all(encode(preferences).as_bytes())
                .map_err(|error| format!("cannot write settings: {error}"))?;
            file.sync_all()
                .map_err(|error| format!("cannot sync settings: {error}"))?;
            fs::rename(&temp, &self.path)
                .map_err(|error| format!("cannot commit settings: {error}"))?;
            if let Ok(dir) = fs::File::open(parent) {
                let _ = dir.sync_all();
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("litric-prefstore-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root.join("var/lib/litric-shell/settings.conf")
    }

    /// Remove the whole fixture root, not just the record's own directory.
    fn cleanup(path: &Path) {
        if let Some(root) = path.ancestors().nth(4) {
            let _ = fs::remove_dir_all(root);
        }
    }

    #[test]
    fn a_record_round_trips_through_the_file() {
        let path = fixture("roundtrip");
        let store = Store::new(&path);
        let mut preferences = Preferences::new(ThemeKind::Light, Language::English);
        preferences.accent = Accent::Teal;
        preferences.wallpaper = Wallpaper::Glow;
        preferences.reduce_motion = true;
        store.save(preferences).expect("the record is written");
        let loaded = store
            .load()
            .expect("the record is read")
            .expect("it exists");
        assert_eq!(loaded, preferences);
        cleanup(&path);
    }

    #[test]
    fn the_schedule_round_trips_and_an_unknown_one_keeps_the_default() {
        use crate::settings::ThemeSchedule;

        let preferences = Preferences {
            theme_schedule: ThemeSchedule::Auto,
            ..Preferences::default()
        };
        let text = encode(preferences);
        assert!(text.contains("theme_schedule=auto"), "{text}");
        assert_eq!(
            parse(&text, Preferences::default()).theme_schedule,
            ThemeSchedule::Auto
        );
        // A file from a build that knows a third value must not reset the
        // record, and must not silently become the wrong schedule.
        let parsed = parse("theme_schedule=sunrise\n", Preferences::default());
        assert_eq!(parsed.theme_schedule, ThemeSchedule::Manual);
    }

    #[test]
    fn an_absent_file_is_not_an_error_and_a_present_one_counts_as_set_up() {
        let path = fixture("absent");
        let store = Store::new(&path);
        assert_eq!(store.load(), Ok(None));
        store
            .save(Preferences::new(ThemeKind::Dark, Language::Thai))
            .expect("the record is written");
        let loaded = store
            .load()
            .expect("the record is read")
            .expect("it exists");
        assert!(
            loaded.onboarded,
            "a file on disk came from a finished setup"
        );
        cleanup(&path);
    }

    #[test]
    fn a_file_from_before_the_flag_existed_still_counts_as_finished_setup() {
        // The record this store starts from is a finished setup; a file that
        // does not mention the flag therefore keeps it. The flag was added after
        // the format, so the alternative is showing first-run to every account
        // that upgraded.
        assert!(Preferences::default().onboarded);
        let parsed = parse("theme=light\nlanguage=en\n", Preferences::default());
        assert_eq!(parsed.theme, ThemeKind::Light);
        assert!(parsed.onboarded);

        // A setup that is in progress says so, and that is honoured.
        assert!(!parse("onboarded=0\n", Preferences::default()).onboarded);
    }

    #[test]
    fn the_search_engine_round_trips_and_an_unknown_one_keeps_the_default() {
        use crate::apps::SearchEngine;

        let preferences = Preferences {
            search_engine: SearchEngine::DuckDuckGo,
            ..Preferences::default()
        };
        let text = encode(preferences);
        assert!(text.contains("search_engine=duckduckgo"), "{text}");
        assert_eq!(
            parse(&text, Preferences::default()).search_engine,
            SearchEngine::DuckDuckGo
        );

        // A file from a newer build that knows a fourth engine must not reset
        // the whole record, and must not silently become the wrong engine.
        let parsed = parse("version=1\nsearch_engine=brave\n", Preferences::default());
        assert_eq!(parsed.search_engine, SearchEngine::Google);
    }

    #[test]
    fn unknown_keys_and_values_keep_the_defaults() {
        let defaults = Preferences {
            accent: Accent::Rose,
            wallpaper: Wallpaper::Flat,
            ..Preferences::default()
        };
        let parsed = parse(
            "# written by a newer build\nversion=9\ntheme=sepia\naccent=chartreuse\nwallpaper=painting\nlanguage=klingon\nreduce_motion=maybe\nonboarded=sure\nsurface_blur=0.8\n",
            defaults,
        );
        assert_eq!(parsed.theme, defaults.theme);
        assert_eq!(parsed.accent, defaults.accent);
        assert_eq!(parsed.wallpaper, defaults.wallpaper);
        assert_eq!(parsed.language, defaults.language);
        assert_eq!(parsed.reduce_motion, defaults.reduce_motion);
        assert_eq!(parsed.onboarded, defaults.onboarded);
    }

    #[test]
    fn malformed_lines_are_skipped_rather_than_failing_the_whole_record() {
        let defaults = Preferences {
            accent: Accent::Amber,
            ..Preferences::default()
        };
        let parsed = parse(
            "not a setting\ntheme=light\n\n=no key\nonboarded\naccent=sky\n",
            defaults,
        );
        assert_eq!(parsed.theme, ThemeKind::Light);
        assert_eq!(parsed.accent, Accent::Sky);
    }

    #[test]
    fn bool_tokens_accept_the_forms_a_person_writes() {
        for (raw, expected) in [
            ("1", true),
            ("true", true),
            ("yes", true),
            ("0", false),
            ("false", false),
            ("no", false),
        ] {
            let parsed = parse(&format!("reduce_motion={raw}\n"), Preferences::default());
            assert_eq!(parsed.reduce_motion, expected, "reduce_motion={raw}");
        }
    }

    #[test]
    fn the_encoded_record_names_its_version_first() {
        let text = encode(Preferences::default());
        assert!(text.starts_with("version=1\n"), "{text}");
        assert!(text.ends_with('\n'), "an appended line must still parse");
        for key in [
            "theme=",
            "accent=",
            "wallpaper=",
            "language=",
            "reduce_motion=",
            "ui_scale=",
            "onboarded=",
            "theme_schedule=",
        ] {
            assert!(text.contains(key), "the record omits {key}: {text}");
        }
    }

    #[test]
    fn the_ui_scale_round_trips_and_a_size_this_build_does_not_draw_is_ignored() {
        let preferences = Preferences {
            ui_scale: UiScale::Large,
            ..Preferences::default()
        };
        let text = encode(preferences);
        assert!(text.contains("ui_scale=1.25"), "{text}");
        assert_eq!(
            parse(&text, Preferences::default()).ui_scale,
            UiScale::Large
        );

        // A hand-edited file may write the multiplier, and an exact step is
        // still understood; a size between steps is not a step.
        assert_eq!(
            parse("ui_scale=1.5\n", Preferences::default()).ui_scale,
            UiScale::Larger
        );
        assert_eq!(
            parse("ui_scale=1.75\n", Preferences::default()).ui_scale,
            UiScale::Normal
        );
        assert_eq!(
            parse("ui_scale=huge\n", Preferences::default()).ui_scale,
            UiScale::Normal
        );
    }

    #[test]
    fn the_file_is_private_to_the_account_and_leaves_no_temporary_behind() {
        let path = fixture("private");
        let store = Store::new(&path);
        store
            .save(Preferences::default())
            .expect("the record is written");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let file = fs::metadata(&path).expect("the record exists");
            assert_eq!(file.permissions().mode() & 0o777, 0o600);
            let dir = fs::metadata(path.parent().expect("parent")).expect("the directory exists");
            assert_eq!(dir.permissions().mode() & 0o777, 0o700);
        }
        let names: Vec<String> = fs::read_dir(path.parent().expect("parent"))
            .expect("the directory is readable")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["settings.conf".to_owned()], "{names:?}");
        cleanup(&path);
    }

    #[test]
    fn a_garbage_file_parses_to_defaults_instead_of_stopping_the_desktop() {
        // A settings file is not trusted input. Binary noise, a truncated write
        // and a text file with the wrong contents all have to produce a desktop.
        let path = fixture("garbage");
        let store = Store::new(&path);
        fs::create_dir_all(path.parent().expect("parent")).expect("the directory is made");
        fs::write(&path, [0x00u8, 0xFF, 0x2E, 0x0A, 0x3D]).expect("garbage is written");
        let loaded = store.load().expect("reading succeeds").expect("it exists");
        let defaults = Preferences::default();
        assert_eq!(loaded.theme, defaults.theme);
        assert_eq!(loaded.accent, defaults.accent);
        assert_eq!(loaded.wallpaper, defaults.wallpaper);
        assert!(loaded.onboarded, "an existing file still means set up");
        cleanup(&path);
    }

    #[test]
    fn saving_over_a_stale_temporary_still_succeeds() {
        // The rename is what publishes a record; a left-over temp file from a
        // killed process must not be the reason the next save fails.
        let path = fixture("stale");
        let store = Store::new(&path);
        fs::create_dir_all(path.parent().expect("parent")).expect("the directory is made");
        let stale = path
            .parent()
            .expect("parent")
            .join(format!("settings.conf.tmp.{}", std::process::id()));
        fs::write(&stale, "theme=light\n").expect("stale temp is written");
        store
            .save(Preferences::new(ThemeKind::Dark, Language::Thai))
            .expect("the save succeeds");
        assert!(!stale.exists(), "the stale temporary is cleaned up");
        cleanup(&path);
    }
}
