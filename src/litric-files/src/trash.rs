//! The freedesktop.org home trash **on disk** (ADR-0019 §4).
//!
//! `litric-trash` owns *what deleting means* — retention, capacity, the
//! ledger — and does no I/O. This module is the disk side of the same
//! contract: it maintains `<root>/.local/share/Trash/{files,info}` in the
//! exact layout the freedesktop specification describes, so other tools
//! (`gio trash`, `kioclient`, every Linux file manager) read the same trash
//! Litric writes.
//!
//! The ledger for decisions lives in [`litric_trash::Trash`]; this module
//! reads and writes the two directories and the `.trashinfo` files, and
//! reports entries the ledger can rebuild itself from.

use crate::ops::{unix_time_of, Root};
use crate::{Error, ErrorKind};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// The directory name the freedesktop specification fixes for the home
/// trash. `ops.rs` excludes it from `RECENT` under this exact name.
pub const TRASH_DIR_NAME: &str = ".local/share/Trash";

/// One entry read back off the disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnDiskItem {
    /// The name inside `Trash/files/`.
    pub name: String,
    /// Where the item came from, as a logical path inside the daemon root
    /// (`/Documents/x`). It is mapped back from the entry's `Path=` line by
    /// [`crate::trashinfo::logical_location`], which understands both the
    /// current form — the original location as an absolute host pathname — and
    /// the legacy `file://<logical>` form a build before BUG-0009 wrote. An
    /// entry in either form restores to the same place.
    pub original_path: String,
    /// True when `original_path` is a recorded location this root does not own
    /// (see [`crate::trashinfo::maps_into_root`]). It is kept and listed — the
    /// record is the truth about where the item came from — but it cannot be
    /// restored: the path looks like a logical path, so resolving it here would
    /// write the item to a location the entry never named.
    pub outside_root: bool,
    /// The deletion date, ISO 8601 `YYYY-MM-DDThh:mm:ss` as recorded.
    pub deleted_iso: String,
    /// Whether the `files/` entry is a directory.
    pub is_dir: bool,
    /// Size of the trashed object, bytes (a directory counts its tree,
    /// measured, because the spec says the info file may omit size).
    pub size: u64,
}

/// The on-disk trash rooted at `<root>/.local/share/Trash`.
#[derive(Debug, Clone)]
pub struct DiskTrash {
    files_dir: PathBuf,
    info_dir: PathBuf,
    /// The host directory logical paths are relative to. Kept because the
    /// reader has to turn the absolute location recorded in `Path=` back into a
    /// logical path, and only the root knows where the absolute form begins.
    host_root: PathBuf,
}

impl DiskTrash {
    /// Open (creating if missing) the trash under the daemon root.
    ///
    /// # Errors
    /// Chain errors when creating the directory chain fails, or a component
    /// of the chain exists but is a symlink (`symlink-refused`).
    pub fn open(root: &Root) -> Result<Self, Error> {
        let base = root.host_root().join(TRASH_DIR_NAME);
        // ensure_dir_chain validates every component it walks, so a link
        // planted anywhere under `.local` is refused rather than followed.
        root.ensure_dir_chain(&base)?;
        let files_dir = base.join("files");
        let info_dir = base.join("info");
        for dir in [&files_dir, &info_dir] {
            match fs::symlink_metadata(dir) {
                Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
                Ok(_) => {
                    return Err(Error::new(
                        ErrorKind::NotADirectory,
                        "the trash layout is damaged: files/ or info/ is not a directory",
                    ))
                }
                Err(_) => fs::create_dir(dir).map_err(|e| Error::from_io("create trash", &e))?,
            }
        }
        Ok(Self {
            files_dir,
            info_dir,
            host_root: root.host_root().to_path_buf(),
        })
    }

    /// The `files/` directory, for listing and restores.
    #[must_use]
    pub fn files_dir(&self) -> &std::path::Path {
        &self.files_dir
    }

    /// Move an item into the trash, writing its `.trashinfo` **first**.
    ///
    /// The specification is explicit about the order: the information file
    /// is created before the item moves, and it is created exclusively
    /// (`O_EXCL`) so that two processes trashing the same name cannot claim
    /// the same entry. Following that order is what makes an interrupted
    /// trash recoverable: the record exists before the item leaves its
    /// place, and a failed move takes the record away again.
    ///
    /// `now` supplies the deletion timestamp (Unix seconds) rendered in the
    /// root's local time, which is what the spec's `DeletionDate=` wants.
    ///
    /// # Errors
    /// `not-found` when the item is gone, chain errors from resolution,
    /// `exists` when no free trash name is left, I/O errors moving or
    /// writing.
    pub fn put(
        &self,
        root: &Root,
        logical: &crate::path::LogicalPath,
        now: u64,
    ) -> Result<OnDiskItem, Error> {
        let host = root.resolve(logical)?;
        let meta = fs::symlink_metadata(&host).map_err(|e| Error::from_io("trash", &e))?;
        let is_dir = meta.is_dir() && !meta.file_type().is_symlink();
        let size = if is_dir {
            let (bytes, _f, _d, _p) = root.measure(logical)?;
            bytes
        } else {
            meta.len()
        };
        let deleted_iso = iso_local(root, now);
        // `Path=` is the original *location*: the absolute host pathname, which
        // is what the Freedesktop specification asks for and what every other
        // tool reads back. It used to be `file://<logical>`, which a conforming
        // reader takes for a *relative* path and restores to the wrong place
        // (BUG-0009). The formatting itself lives in one place now —
        // [`crate::trashinfo`] — so this writer and the shell's cannot drift.
        let record = crate::trashinfo::record(&host, &deleted_iso);
        let name = self.reserve_info(logical.file_name(), &record)?;
        if let Err(e) = fs::rename(&host, self.files_dir.join(&name)) {
            // Nothing left its place, so the reservation has to go too: an
            // info file with no item behind it would list nowhere and
            // restore nothing.
            let _ = fs::remove_file(self.info_dir.join(format!("{name}.trashinfo")));
            return Err(Error::from_io("trash move", &e));
        }
        Ok(OnDiskItem {
            name,
            // The logical path, not the absolute location that was written:
            // every caller of this field wants something it can hand back to
            // `Root::resolve`, and the mapping is lossless (the record holds the
            // absolute form, and `logical_location` inverts it).
            original_path: logical.as_str().to_owned(),
            // An item we just trashed came out of this root by construction.
            outside_root: false,
            deleted_iso,
            is_dir,
            size,
        })
    }

    /// Take a trash name by creating its `.trashinfo` exclusively, which is
    /// how the specification makes the choice atomic: `create_new` is
    /// `O_EXCL`, so a name already claimed fails and the next candidate is
    /// tried instead of overwriting the entry that holds it.
    ///
    /// A name counts as taken when **either** half of the entry is present —
    /// the record in `info/` or the item in `files/`. Reading both is what
    /// keeps a half-written entry (an older build moved the item first and
    /// could die before writing the record) from being overwritten by the
    /// next trashing of the same name.
    ///
    /// # Errors
    /// `exists` when no candidate is free; I/O errors creating the record.
    fn reserve_info(&self, base: &str, record: &str) -> Result<String, Error> {
        let taken = |n: &str| {
            fs::symlink_metadata(self.info_dir.join(format!("{n}.trashinfo"))).is_ok()
                || fs::symlink_metadata(self.files_dir.join(n)).is_ok()
        };
        let mut candidates: Vec<String> = vec![base.to_owned()];
        candidates.extend(numbered_names(base));
        // The last resort the previous implementation used, kept so a
        // directory full of `name.N.ext` still trashes rather than failing.
        candidates.push(format!("{base}.{}", now_nanos_fallback()));
        for candidate in candidates {
            if taken(&candidate) {
                continue;
            }
            let path = self.info_dir.join(format!("{candidate}.trashinfo"));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    // `write_all` is where a full disk or a permission
                    // error surfaces; a `File` drop cannot report one, so
                    // nothing is left to a silent close.
                    if let Err(e) = file.write_all(record.as_bytes()) {
                        let _ = fs::remove_file(&path);
                        return Err(Error::from_io("write trashinfo", &e));
                    }
                    return Ok(candidate);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(Error::from_io("create trashinfo", &e)),
            }
        }
        Err(Error::new(
            ErrorKind::Exists,
            "no free trash name for that item",
        ))
    }

    /// Every trash entry, oldest deletion first — the `TRASHLIST` order.
    ///
    /// An entry whose `.trashinfo` is missing or unreadable is skipped:
    /// a half-written trash entry is shown as absent rather than failing
    /// the whole listing.
    ///
    /// # Errors
    /// Only when `info/` itself is unreadable.
    pub fn list(&self) -> Result<Vec<OnDiskItem>, Error> {
        let mut out = Vec::new();
        let mut names: Vec<String> = fs::read_dir(&self.info_dir)
            .map_err(|e| Error::from_io("list trash", &e))?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|n| n.ends_with(".trashinfo"))
            .map(|n| n.trim_end_matches(".trashinfo").to_owned())
            .collect();
        names.sort();
        for name in names {
            let Some(item) = self.read_one(&name) else {
                continue;
            };
            out.push(item);
        }
        out.sort_by(|a, b| a.deleted_iso.cmp(&b.deleted_iso));
        Ok(out)
    }

    fn read_one(&self, name: &str) -> Option<OnDiskItem> {
        let raw = fs::read_to_string(self.info_dir.join(format!("{name}.trashinfo"))).ok()?;
        let mut original = String::new();
        let mut outside_root = false;
        let mut deleted = String::new();
        for line in raw.lines() {
            if let Some(v) = line.strip_prefix("Path=") {
                // Decode, then map back to a logical path. The current absolute
                // form and the legacy `file://` form both resolve here; anything
                // else is kept verbatim AND flagged, because a recorded location
                // from another root is often a syntactically valid logical path
                // (`/somewhere/else/stray.txt`) and syntax alone cannot refuse
                // it. The flag is what stops a restore from landing somewhere
                // guessed.
                let decoded = crate::trashinfo::pct_decode(v);
                outside_root = !crate::trashinfo::maps_into_root(&self.host_root, &decoded);
                original = crate::trashinfo::logical_location(&self.host_root, &decoded);
            } else if let Some(v) = line.strip_prefix("DeletionDate=") {
                deleted = v.to_owned();
            }
        }
        if original.is_empty() {
            return None; // no Path=, nothing to restore to — not an entry
        }
        let host = self.files_dir.join(name);
        let meta = fs::symlink_metadata(&host).ok()?;
        Some(OnDiskItem {
            name: name.to_owned(),
            original_path: original,
            outside_root,
            deleted_iso: deleted,
            is_dir: meta.is_dir() && !meta.file_type().is_symlink(),
            size: meta.len(),
        })
    }

    /// Look up one entry by trash name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<OnDiskItem> {
        if name.contains('/') || name.starts_with('.') {
            return None;
        }
        self.read_one(name)
    }

    /// Move an entry out of the trash back to `dest_parent/name` as chosen
    /// by the ledger's `plan_restore`. Returns the host path restored to.
    ///
    /// # Errors
    /// `restore-would-overwrite` when something stands at the destination
    /// (the ledger's rule, enforced again here against the live disk),
    /// `not-found` when the entry vanished, I/O errors.
    pub fn restore(&self, name: &str, dest: &std::path::Path) -> Result<std::path::PathBuf, Error> {
        // The same name rule `get` and `purge` apply, enforced here too: a
        // trash name is one segment inside `files/`, never a path. The
        // server resolves the name through `get` first, so this is the
        // second of two independent checks, not the only one.
        if name.contains('/') || name.starts_with('.') {
            return Err(Error::new(ErrorKind::InvalidName, "bad trash name"));
        }
        if fs::symlink_metadata(dest).is_ok() {
            return Err(Error::new(
                ErrorKind::RestoreWouldOverwrite,
                "the original location is occupied",
            ));
        }
        let src = self.files_dir.join(name);
        if fs::symlink_metadata(&src).is_err() {
            return Err(Error::new(ErrorKind::NotFound, "no such trash entry"));
        }
        fs::rename(&src, dest).map_err(|e| Error::from_io("restore", &e))?;
        let _ = fs::remove_file(self.info_dir.join(format!("{name}.trashinfo")));
        Ok(dest.to_path_buf())
    }

    /// Permanently remove one entry and its info file. Returns the size
    /// freed.
    ///
    /// # Errors
    /// `not-found` when the entry is gone; I/O errors otherwise.
    pub fn purge(&self, name: &str) -> Result<u64, Error> {
        if name.contains('/') || name.starts_with('.') {
            return Err(Error::new(ErrorKind::InvalidName, "bad trash name"));
        }
        let host = self.files_dir.join(name);
        let meta = fs::symlink_metadata(&host)
            .map_err(|_| Error::new(ErrorKind::NotFound, "no such trash entry"))?;
        let size = if meta.is_dir() && !meta.file_type().is_symlink() {
            dir_size(&host)
        } else {
            meta.len()
        };
        if meta.is_dir() && !meta.file_type().is_symlink() {
            fs::remove_dir_all(&host).map_err(|e| Error::from_io("purge", &e))?;
        } else {
            fs::remove_file(&host).map_err(|e| Error::from_io("purge", &e))?;
        }
        let _ = fs::remove_file(self.info_dir.join(format!("{name}.trashinfo")));
        Ok(size)
    }

    /// Empty the whole trash. Returns `(entries removed, bytes freed)`.
    ///
    /// # Errors
    /// Only when `info/` is unreadable; per-entry failures skip the entry.
    pub fn empty(&self) -> Result<(u64, u64), Error> {
        let mut removed = 0u64;
        let mut freed = 0u64;
        for item in self.list()? {
            if self.purge(&item.name).is_ok() {
                removed = removed.saturating_add(1);
                freed = freed.saturating_add(item.size);
            }
        }
        Ok((removed, freed))
    }
}

// Percent-encoding for the `Path=` field lives in [`crate::trashinfo`], with the
// rest of the record's format. It used to be duplicated here and in
// `litric-shell`'s bridge, which is how one defect came to be fixed twice
// (BUG-0011) and why the format itself had two chances to be wrong (BUG-0009).

/// Directory size by walking; used where a `.trashinfo`-recorded size is
/// absent and the ledger needs a number.
fn dir_size(host: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(read) = fs::read_dir(host) {
        for entry in read.filter_map(|e| e.ok()) {
            if let Ok(meta) = fs::symlink_metadata(entry.path()) {
                if meta.is_dir() && !meta.file_type().is_symlink() {
                    total = total.saturating_add(dir_size(&entry.path()));
                } else {
                    total = total.saturating_add(meta.len());
                }
            }
        }
    }
    total
}

/// ISO 8601 local time, the `DeletionDate=` format. Composed from the
/// root's zone rather than reaching for a clock, so tests can pin it.
#[must_use]
pub fn iso_local(root: &Root, unix: u64) -> String {
    let dt = root.zone().local(i64::try_from(unix).unwrap_or(i64::MAX));
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
    )
}

/// `name.ext` becomes `name.2.ext`, `name.3.ext`, … — the collision order
/// this module has always used, so an entry's name keeps the meaning it
/// already had under the previous implementation.
fn numbered_names(base: &str) -> impl Iterator<Item = String> + '_ {
    let (stem, ext) = match base.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s, Some(e)),
        _ => (base, None),
    };
    (2..10_000u32).map(move |n| match ext {
        Some(e) => format!("{stem}.{n}.{e}"),
        None => format!("{stem}.{n}"),
    })
}

fn now_nanos_fallback() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0)
}

/// Size of a metadata-modified entry, shared with the server layer.
#[must_use]
pub fn entry_size(meta: &fs::Metadata) -> u64 {
    if meta.is_dir() && !meta.file_type().is_symlink() {
        0
    } else {
        meta.len()
    }
}

/// mtime of a metadata record, in Unix seconds — the shared stat rule.
#[must_use]
pub fn entry_mtime(meta: &fs::Metadata) -> u64 {
    unix_time_of(meta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::LogicalPath;
    use litric_locale::TimeZone;

    fn open_root(tag: &str) -> (PathBuf, crate::ops::Root) {
        let dir = std::env::temp_dir().join(format!("litric-files-trash-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("fixture root");
        let root = crate::ops::Root::open(&dir, TimeZone::utc("test")).expect("root opens");
        (dir, root)
    }

    fn logical(path: &str) -> LogicalPath {
        LogicalPath::parse(path).expect("valid logical path")
    }

    /// The `Path=` value of a record as it sits on disk, decoded.
    fn recorded_path(dir: &std::path::Path, name: &str) -> String {
        let raw = fs::read_to_string(
            dir.join(TRASH_DIR_NAME)
                .join("info")
                .join(format!("{name}.trashinfo")),
        )
        .expect("the record was written");
        let value = raw
            .lines()
            .find_map(|l| l.strip_prefix("Path="))
            .expect("a Path= line");
        crate::trashinfo::pct_decode(value)
    }

    #[test]
    fn the_recorded_path_is_the_absolute_location_not_a_uri() {
        // BUG-0009 regression. The reader must be handed the item's real host
        // location, and no `file://` may appear anywhere in the stored value:
        // a conforming tool would take that for a relative path and restore the
        // item under `<trash parent>/file:/...`.
        let (dir, root) = open_root("path-format");
        fs::create_dir_all(dir.join("Documents")).expect("Documents");
        fs::write(dir.join("Documents/memo.txt"), "x").expect("write");
        let disk = DiskTrash::open(&root).expect("trash");

        let item = disk
            .put(&root, &logical("/Documents/memo.txt"), 1_900_000_000)
            .expect("trash");

        let absolute = dir.join("Documents/memo.txt");
        assert_eq!(recorded_path(&dir, &item.name), absolute.to_string_lossy());
        assert!(
            crate::trashinfo::is_not_a_uri(&recorded_path(&dir, &item.name)),
            "Path= must be a location: {:?}",
            recorded_path(&dir, &item.name)
        );
        // The struct reports the logical path, and the reader agrees with it.
        assert_eq!(item.original_path, "/Documents/memo.txt");
        assert_eq!(
            disk.get(&item.name).expect("listed").original_path,
            "/Documents/memo.txt"
        );
    }

    #[test]
    fn an_entry_written_in_the_legacy_uri_form_still_lists_and_restores() {
        // A `.trashinfo` as an older LitricOS build wrote it: the whole
        // `file://<logical>` URI, percent-encoded. The change of format must not
        // strand the user's existing trash.
        let (dir, root) = open_root("legacy-uri");
        let base = dir.join(TRASH_DIR_NAME);
        fs::create_dir_all(base.join("files")).expect("files");
        fs::create_dir_all(base.join("info")).expect("info");
        fs::write(base.join("files/note.txt"), "old").expect("item");
        let legacy = crate::trashinfo::pct_encode("file:///Documents/note.txt");
        fs::write(
            base.join("info/note.txt.trashinfo"),
            format!("[Trash Info]\nPath={legacy}\nDeletionDate=2026-01-01T00:00:00\n"),
        )
        .expect("record");

        let disk = DiskTrash::open(&root).expect("trash");
        let item = disk
            .get("note.txt")
            .expect("the legacy entry is still listed");
        assert_eq!(item.original_path, "/Documents/note.txt");

        fs::create_dir_all(dir.join("Documents")).expect("Documents");
        let dest = dir.join("Documents/note.txt");
        disk.restore("note.txt", &dest).expect("restores");
        assert!(dest.exists(), "back where it came from");
        assert!(
            !base.join("files/note.txt").exists(),
            "and gone from the trash"
        );
    }

    #[test]
    fn an_entry_whose_path_names_nothing_inside_the_root_is_refused_not_guessed() {
        // A `Path=` outside the root cannot be mapped to a logical path, so the
        // reader keeps it verbatim and FLAGS it. The flag is the part that
        // matters: this recorded location is a syntactically valid logical path
        // (`/somewhere/else/stray.txt` parses fine), so a caller that trusted the
        // parse alone would restore the item into this root, at a location the
        // entry never named. This test originally asserted that parsing fails,
        // which cannot be true for a rooted path and so never exercised the
        // refusal at all -- see the RESTORE and PARSE-not-flag coverage in
        // server.rs and in the `outside_root` assertion below.
        let (dir, root) = open_root("outside-root");
        let base = dir.join(TRASH_DIR_NAME);
        fs::create_dir_all(base.join("files")).expect("files");
        fs::create_dir_all(base.join("info")).expect("info");
        fs::write(base.join("files/stray.txt"), "x").expect("item");
        fs::write(
            base.join("info/stray.txt.trashinfo"),
            "[Trash Info]\nPath=/somewhere/else/stray.txt\n\
             DeletionDate=2026-01-01T00:00:00\n",
        )
        .expect("record");

        let disk = DiskTrash::open(&root).expect("trash");
        let item = disk.get("stray.txt").expect("listed");
        assert_eq!(item.original_path, "/somewhere/else/stray.txt");
        assert!(item.outside_root, "the recorded location is not ours");
        // And the listing still parses as a logical path, which is exactly why
        // the flag has to exist.
        assert!(LogicalPath::parse(&item.original_path).is_ok());
        // The flag has to mean something, so the honest case is asserted next
        // to the dishonest one: an item this root trashed itself is NOT flagged.
        let (dir2, root2) = open_root("inside-root");
        fs::create_dir_all(dir2.join("Documents")).expect("Documents");
        fs::write(dir2.join("Documents/kept.txt"), "x").expect("file");
        let disk2 = DiskTrash::open(&root2).expect("trash");
        disk2
            .put(&root2, &logical("/Documents/kept.txt"), 0)
            .expect("trashed");
        let kept = disk2.get("kept.txt").expect("listed");
        assert_eq!(kept.original_path, "/Documents/kept.txt");
        assert!(
            !kept.outside_root,
            "an entry from this root stays restorable"
        );
    }
}
