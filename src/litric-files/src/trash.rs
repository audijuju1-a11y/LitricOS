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
    /// The original location, decoded from the entry's `Path=` line. Today
    /// that value is `file://<logical path>`, which round-trips through
    /// [`unpct_path`] but is **not** what the trash specification asks for:
    /// `Path=` must carry the original *location* (an absolute pathname, or
    /// a relative one measured from the trash directory's parent). Recorded
    /// as BUG-0009; the format is shared with the shell's own trash writer,
    /// so it changes in both places or not at all.
    pub original_path: String,
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
        let original_uri = format!("file://{}", logical.as_str());
        let size = if is_dir {
            let (bytes, _f, _d, _p) = root.measure(logical)?;
            bytes
        } else {
            meta.len()
        };
        let deleted_iso = iso_local(root, now);
        let record = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            pct_path(&original_uri),
            deleted_iso
        );
        let name = self.reserve_info(&logical.file_name().to_owned(), &record)?;
        if let Err(e) = fs::rename(&host, self.files_dir.join(&name)) {
            // Nothing left its place, so the reservation has to go too: an
            // info file with no item behind it would list nowhere and
            // restore nothing.
            let _ = fs::remove_file(self.info_dir.join(format!("{name}.trashinfo")));
            return Err(Error::from_io("trash move", &e));
        }
        Ok(OnDiskItem {
            name,
            original_path: original_uri,
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
        let mut deleted = String::new();
        for line in raw.lines() {
            if let Some(v) = line.strip_prefix("Path=") {
                original = unpct_path(v);
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

/// `file:///Documents/a%20b.txt` — the spec wants `Path=` percent-encoded.
/// The logical path's segments are already legal names, so encoding the
/// whole string is safe: only reserved characters change.
fn pct_path(uri: &str) -> String {
    uri.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'.' | b'_' | b'~') {
                String::from(b as char)
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// The inverse of [`pct_path`], applied when reading back.
///
/// Index invariants: `i` only ever advances by 1 or 3 and never exceeds
/// `bytes.len()`, so every `get` below either succeeds or falls through to
/// the literal-byte path; the arithmetic is bounded by the input length.
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn unpct_path(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i) == Some(&b'%') {
            if let Some(hex) = s.get(i + 1..i + 3) {
                if let Ok(b) = u8::from_str_radix(hex, 16) {
                    out.push(b);
                    i += 3;
                    continue;
                }
            }
        }
        if let Some(b) = bytes.get(i) {
            out.push(*b);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

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
