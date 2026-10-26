//! `COMPRESS` and `EXTRACT` — archive creation and extraction under the
//! ADR-0019 limits (§ "Extraction limits").
//!
//! The codecs ([`crate::zip`], [`crate::tar`], [`crate::gzip`]) are pure
//! in-memory transformations with their own bomb checks. This module adds
//! the filesystem half: walking the real tree for compression, streaming
//! validated entries to disk for extraction, and refusing everything the
//! ADR names — absolute names, `..`, drive letters, backslashes, symlinks,
//! hard links, device nodes, entries past the limits.
//!
//! Extraction never follows or creates a symlink: the readers refuse link
//! entries outright (`unsupported-archive`), so the writer below only ever
//! sees regular files and directories.

use crate::ops::{Root, WALK_CAP};
use crate::path::LogicalPath;
use crate::{Error, ErrorKind, Limits};
use std::fs;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};

/// The suffixes `EXTRACT` understands, in the order they are tried.
pub const EXTRACT_SUFFIXES: [&str; 5] = [".zip", ".tar.gz", ".tgz", ".tar", ".gz"];

/// What kind of archive a name declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    /// PKWARE APPNOTE zip.
    Zip,
    /// POSIX ustar, optionally gzip-compressed.
    Tar,
    /// A single gzip member (`.gz` alone).
    Gzip,
}

impl ArchiveKind {
    /// Classify by file-name suffix. `.tar.gz` and `.tgz` are tar.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        if lower.ends_with(".zip") {
            Some(Self::Zip)
        } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") || lower.ends_with(".tar") {
            Some(Self::Tar)
        } else if lower.ends_with(".gz") {
            Some(Self::Gzip)
        } else {
            None
        }
    }
}

/// Read an archive's bytes by its logical path, transparently
/// decompressing the gzip-wrapped forms.
///
/// # Errors
/// `not-found`/chain errors from resolution, `unsupported-archive` for a
/// suffix that is not recognised, codec errors pass through as
/// `corrupt-archive`/`limit` inside `io::Error`.
pub fn load_archive_bytes(
    root: &Root,
    logical: &LogicalPath,
) -> Result<(Vec<u8>, ArchiveKind), Error> {
    let kind = ArchiveKind::from_name(logical.as_str()).ok_or_else(|| {
        Error::new(
            ErrorKind::UnsupportedArchive,
            "not an archive Litric can extract (.zip .tar .tar.gz .tgz .gz)",
        )
    })?;
    let host = root.resolve(logical)?;
    let raw = fs::read(&host).map_err(|e| Error::from_io("read archive", &e))?;
    match kind {
        ArchiveKind::Gzip | ArchiveKind::Tar
            if logical.as_str().to_lowercase().ends_with(".gz")
                || logical.as_str().to_lowercase().ends_with(".tgz") =>
        {
            let decoded = gzip_decode(&raw)?;
            Ok((
                decoded,
                if kind == ArchiveKind::Tar {
                    ArchiveKind::Tar
                } else {
                    ArchiveKind::Gzip
                },
            ))
        }
        _ => Ok((raw, kind)),
    }
}

fn gzip_decode(raw: &[u8]) -> Result<Vec<u8>, Error> {
    let mut reader = BufReader::new(raw);
    let (out, _members) = crate::gzip::decompress(&mut reader, Limits::ADR_0019.max_output)
        .map_err(|e| Error::from_io("gunzip", &e))?;
    Ok(out)
}

/// `COMPRESS <dest-zip> <path>…`: write a new zip holding every source.
/// The destination must not exist (`exists` otherwise). Progress is
/// reported per source item through `on_progress(done_bytes, total_bytes)`.
///
/// # Errors
/// `exists` when the destination name is taken, chain errors for sources,
/// `too-large` from the zip writer for a file past the 32-bit format,
/// I/O errors reading sources.
pub fn compress_zip(
    root: &Root,
    dest: &LogicalPath,
    sources: &[LogicalPath],
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PathBuf, Error> {
    let (parent, name) = root.resolve_parent(dest)?;
    let dest_host = parent.join(name);
    if fs::symlink_metadata(&dest_host).is_ok() {
        return Err(Error::new(
            ErrorKind::Exists,
            "an archive with that name already exists",
        ));
    }
    // Plan first: every source must resolve before any byte is written,
    // so a bad source does not leave a half archive behind.
    let mut planned: Vec<(PathBuf, String, u64)> = Vec::new();
    let mut total = 0u64;
    for src in sources {
        let host = root.resolve(src)?;
        let meta = fs::symlink_metadata(&host).map_err(|e| Error::from_io("compress", &e))?;
        if meta.is_dir() && !meta.file_type().is_symlink() {
            let mut size = 0u64;
            collect_tree(&host, &mut planned_size(&mut size));
            total = total.saturating_add(size);
        } else {
            total = total.saturating_add(meta.len());
        }
        planned.push((host, archive_name_of(src), meta.len()));
    }
    let file = fs::File::create(&dest_host).map_err(|e| Error::from_io("create archive", &e))?;
    let mut out = std::io::BufWriter::new(file);
    let mut done = 0u64;
    let mut specs: Vec<(crate::zip::ZipSpec, Vec<u8>)> = Vec::new();
    for (host, arc_name, size) in &planned {
        let meta = fs::symlink_metadata(host).map_err(|e| Error::from_io("compress", &e))?;
        if meta.is_dir() && !meta.file_type().is_symlink() {
            // Directories and their contents, relative to the source root.
            let base_dir = PathBuf::from(host);
            let mut entries: Vec<(String, PathBuf)> = Vec::new();
            collect_tree_named(&base_dir, "", &mut entries);
            for (rel, path) in &entries {
                let name_in_zip = format!("{arc_name}/{rel}");
                let m = fs::symlink_metadata(path).map_err(|e| Error::from_io("compress", &e))?;
                if m.is_dir() && !m.file_type().is_symlink() {
                    specs.push((
                        crate::zip::ZipSpec {
                            name: format!("{name_in_zip}/"),
                            store: true,
                        },
                        Vec::new(),
                    ));
                } else if m.is_file() {
                    let bytes = fs::read(path).map_err(|e| Error::from_io("compress", &e))?;
                    done = done.saturating_add(bytes.len() as u64);
                    on_progress(done, total);
                    specs.push((
                        crate::zip::ZipSpec {
                            name: name_in_zip,
                            store: false,
                        },
                        bytes,
                    ));
                }
                // Links inside a tree are skipped on compression: a zip
                // entry cannot faithfully carry one, and inventing a copy
                // would lie about what was archived.
            }
        } else if meta.file_type().is_symlink() {
            // A link as an explicit source: archive the *target text* as a
            // small file, which is honest about what a zip can carry and
            // never turns into a traversal on extract.
            let target = fs::read_link(host).map_err(|e| Error::from_io("read link", &e))?;
            let text = target.to_string_lossy().into_owned();
            done = done.saturating_add(size.saturating_add(text.len() as u64));
            on_progress(done, total);
            specs.push((
                crate::zip::ZipSpec {
                    name: format!("{arc_name}.lnk.txt"),
                    store: true,
                },
                text.into_bytes(),
            ));
        } else {
            let bytes = fs::read(host).map_err(|e| Error::from_io("compress", &e))?;
            done = done.saturating_add(bytes.len() as u64);
            on_progress(done, total);
            specs.push((
                crate::zip::ZipSpec {
                    name: arc_name.clone(),
                    store: false,
                },
                bytes,
            ));
        }
    }
    crate::zip::write_zip(&mut out, &specs).map_err(|e| Error::from_io("write archive", &e))?;
    out.flush()
        .map_err(|e| Error::from_io("flush archive", &e))?;
    Ok(dest_host)
}

fn planned_size(size: &mut u64) -> impl FnMut(u64) + '_ {
    move |n: u64| *size = size.saturating_add(n)
}

fn collect_tree(dir: &Path, visit: &mut impl FnMut(u64)) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read.filter_map(|e| e.ok()) {
        let Ok(meta) = fs::symlink_metadata(entry.path()) else {
            continue;
        };
        if meta.is_dir() && !meta.file_type().is_symlink() {
            collect_tree(&entry.path(), visit);
        } else {
            visit(meta.len());
        }
    }
}

fn collect_tree_named(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read.filter_map(|e| e.ok()) {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let child_rel = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        out.push((child_rel.clone(), entry.path()));
        if let Ok(meta) = fs::symlink_metadata(entry.path()) {
            if meta.is_dir() && !meta.file_type().is_symlink() && out.len() < WALK_CAP {
                collect_tree_named(&entry.path(), &child_rel, out);
            }
        }
    }
}

/// The name an archived item carries: its final segment.
fn archive_name_of(logical: &LogicalPath) -> String {
    logical.file_name().to_owned()
}

/// `EXTRACT <archive> <dest-dir>`: validate the whole archive first (the
/// readers enforce entries, ratios, output caps), then write it out.
/// Returns the destination directory.
///
/// # Errors
/// Every rule from ADR-0019: `unsafe-archive-entry` for bad names,
/// `unsupported-archive` for links and special entries, `limit` and
/// `corrupt-archive` from the codecs, `exists` when the destination exists.
pub fn extract_archive(
    root: &Root,
    archive: &LogicalPath,
    dest_dir: &LogicalPath,
    on_progress: impl FnMut(u64, u64),
) -> Result<PathBuf, Error> {
    let (raw, kind) = load_archive_bytes(root, archive)?;
    let dest_parent = root.resolve(dest_dir)?;
    if fs::symlink_metadata(&dest_parent).is_err() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "destination directory does not exist",
        ));
    }
    match kind {
        ArchiveKind::Zip => extract_zip(root, &raw, dest_dir, on_progress),
        ArchiveKind::Tar => extract_tar(root, &raw, dest_dir, on_progress),
        ArchiveKind::Gzip => extract_single_gzip(root, &raw, archive, dest_dir, on_progress),
    }
}

fn extract_zip(
    root: &Root,
    raw: &[u8],
    dest_dir: &LogicalPath,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PathBuf, Error> {
    let entries = crate::zip::read_zip_slice(raw, &Limits::ADR_0019)
        .map_err(|e| Error::from_io("validate zip", &e))?;
    let out_dir_host = make_extract_root(root, dest_dir)?;
    let total: u64 = entries.iter().map(|e| e.content.len() as u64).sum();
    let mut done = 0u64;
    for entry in &entries {
        let logical = extract_path(dest_dir, &entry.name, entry.is_dir)?;
        let host = write_entry(root, &logical, &entry.content, entry.is_dir)?;
        done = done.saturating_add(entry.content.len() as u64);
        on_progress(done, total);
        let _ = host;
    }
    Ok(out_dir_host)
}

fn extract_tar(
    root: &Root,
    raw: &[u8],
    dest_dir: &LogicalPath,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PathBuf, Error> {
    let out_dir_host = make_extract_root(root, dest_dir)?;
    let entries = crate::tar::read_archive(&mut &raw[..], &Limits::ADR_0019)
        .map_err(|e| Error::from_io("validate tar", &e))?;
    let total: u64 = entries
        .iter()
        .map(|(n, _d, c)| if n.ends_with('/') { 0 } else { c.len() as u64 })
        .sum();
    let mut done = 0u64;
    for (name, is_dir, content) in &entries {
        let is_dir = *is_dir || name.ends_with('/');
        let clean = name.trim_end_matches('/');
        let logical = extract_path(dest_dir, clean, is_dir)?;
        write_entry(root, &logical, content, is_dir)?;
        done = done.saturating_add(content.len() as u64);
        on_progress(done, total);
    }
    Ok(out_dir_host)
}

fn extract_single_gzip(
    root: &Root,
    raw: &[u8],
    archive: &LogicalPath,
    dest_dir: &LogicalPath,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PathBuf, Error> {
    let out_dir_host = make_extract_root(root, dest_dir)?;
    let decoded = gzip_decode(raw)?;
    // `notes.txt.gz` extracts as `notes.txt`, and `x.tgz` as `x.tar`. The
    // suffix match is case-insensitive because the classification that chose
    // this path is: `NOTES.TXT.GZ` is a gzip archive, and without the fold
    // its output would be named `NOTES.TXT.GZ` — the archive itself, so the
    // extraction failed with a bare `exists` instead of writing anything.
    let base = archive.file_name();
    let stem: String = if let Some(s) = strip_suffix_ci(base, ".tgz") {
        format!("{s}.tar")
    } else if let Some(s) = strip_suffix_ci(base, ".gz") {
        s.to_owned()
    } else {
        base.to_owned()
    };
    crate::ops::Root::validate_segment(&stem)?;
    let logical = dest_dir
        .join(&stem)
        .map_err(|e| Error::new(ErrorKind::InvalidName, format!("bad extract name: {e}")))?;
    let total = decoded.len() as u64;
    write_entry(root, &logical, &decoded, false)?;
    on_progress(total, total);
    Ok(out_dir_host)
}

/// Create the extraction root if missing and prove it is a real directory.
fn make_extract_root(root: &Root, dest_dir: &LogicalPath) -> Result<PathBuf, Error> {
    let host = root.resolve(dest_dir)?;
    let meta = fs::symlink_metadata(&host).map_err(|e| Error::from_io("extract dest", &e))?;
    if meta.file_type().is_symlink() {
        return Err(Error::new(
            ErrorKind::SymlinkRefused,
            "extraction never writes through a link",
        ));
    }
    if !meta.is_dir() {
        return Err(Error::new(
            ErrorKind::NotADirectory,
            "the destination is not a directory",
        ));
    }
    Ok(host)
}

/// Turn an archive entry name into a [`LogicalPath`] under `dest_dir`,
/// enforcing every ADR-0019 name rule before anything is written.
fn extract_path(dest_dir: &LogicalPath, name: &str, is_dir: bool) -> Result<LogicalPath, Error> {
    let trimmed = name.trim_start_matches('/');
    let unsafe_name =
        |why: &str| Error::new(ErrorKind::UnsafeArchiveEntry, format!("{why}: {name:?}"));
    if trimmed.is_empty() {
        return Err(unsafe_name("empty entry name"));
    }
    if trimmed.contains('\\') {
        return Err(unsafe_name("backslash in name (Windows-style path)"));
    }
    if trimmed.as_bytes().get(1) == Some(&b':') {
        return Err(unsafe_name("drive letter in name"));
    }
    let mut logical = dest_dir.clone();
    for segment in trimmed.split('/') {
        if segment.is_empty() {
            continue; // `a//b` collapses; only `..` and `.` are meaningful
        }
        if segment == ".." {
            return Err(unsafe_name("parent traversal in name"));
        }
        if segment == "." {
            continue;
        }
        LogicalPath::root()
            .join(segment)
            .map_err(|e| unsafe_name(&format!("bad segment ({e})")))?;
        logical = logical.join(segment).map_err(|e| {
            Error::new(
                ErrorKind::UnsafeArchiveEntry,
                format!("bad entry name: {e}"),
            )
        })?;
    }
    if is_dir && logical.as_str() == dest_dir.as_str() {
        return Err(unsafe_name("entry is the destination itself"));
    }
    Ok(logical)
}

/// Write one validated entry: `create_dir` or `create_new` (O_EXCL), which
/// refuse an existing entry including a symlink — the no-follow creation
/// guarantee comes from the syscall.
///
/// The one existing entry a directory entry accepts is a real directory
/// this same extraction already created: archives legitimately carry the
/// same directory twice (ustar writers store `docs/` and some tools also
/// store `docs`), and treating the repeat as an error aborted the whole
/// extraction part-way through for a request that was not wrong.
fn write_entry(
    root: &Root,
    logical: &LogicalPath,
    content: &[u8],
    is_dir: bool,
) -> Result<PathBuf, Error> {
    // The entry's own folder may be missing: a zip is allowed to carry
    // `docs/report.txt` and no `docs/` entry at all, and `COMPRESS` writes
    // exactly that shape. `resolve_parent` would refuse it with `not-found`, so
    // the parent is resolved through the creating form. The link checks are the
    // same ones every other write performs.
    let parent_logical = logical
        .parent()
        .ok_or_else(|| Error::new(ErrorKind::NotADirectory, "the root has no parent"))?;
    let parent_host = root.resolve_dir_creating(&parent_logical)?;
    let host = parent_host.join(logical.file_name());
    if is_dir {
        match fs::create_dir(&host) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                let meta =
                    fs::symlink_metadata(&host).map_err(|e| Error::from_io("extract dir", &e))?;
                if meta.file_type().is_symlink() || !meta.is_dir() {
                    return Err(Error::new(
                        ErrorKind::UnsafeArchiveEntry,
                        "a non-directory stands where the archive wants a directory",
                    ));
                }
            }
            Err(e) => return Err(Error::from_io("extract dir", &e)),
        }
    } else {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&host)
            .map_err(|e| Error::from_io("extract file", &e))?;
        file.write_all(content)
            .map_err(|e| Error::from_io("extract write", &e))?;
    }
    Ok(host)
}

/// `text` without a trailing `suffix`, compared case-insensitively.
///
/// Byte arithmetic, not character arithmetic: `get` refuses a cut that is
/// not a character boundary, so no input can make this panic, and the
/// comparison is `eq_ignore_ascii_case` on the tail rather than a lowercasing
/// of the whole string (which can change its length).
#[must_use]
fn strip_suffix_ci<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    let cut = text.len().checked_sub(suffix.len())?;
    let tail = text.get(cut..)?;
    if tail.eq_ignore_ascii_case(suffix) {
        text.get(..cut)
    } else {
        None
    }
}

/// `COMPRESS`-helper: strip a trailing slash only.
#[must_use]
pub fn trim_slash(name: &str) -> &str {
    name.trim_end_matches('/')
}

/// One piece the server layer needs: how many bytes an archive's
/// *validated* extraction will write, without touching the disk. Used for
/// the progress denominator before extraction starts.
///
/// # Errors
/// As [`load_archive_bytes`] plus the codec validation errors.
pub fn planned_extract_total(root: &Root, archive: &LogicalPath) -> Result<u64, Error> {
    let (raw, kind) = load_archive_bytes(root, archive)?;
    match kind {
        ArchiveKind::Zip => {
            let entries = crate::zip::read_zip_slice(&raw, &Limits::ADR_0019)
                .map_err(|e| Error::from_io("validate zip", &e))?;
            Ok(entries.iter().map(|e| e.content.len() as u64).sum())
        }
        ArchiveKind::Tar => {
            let entries = crate::tar::read_archive(&mut &raw[..], &Limits::ADR_0019)
                .map_err(|e| Error::from_io("validate tar", &e))?;
            Ok(entries
                .iter()
                .filter(|(n, _, _)| !n.ends_with('/'))
                .map(|(_, _, c)| c.len() as u64)
                .sum())
        }
        ArchiveKind::Gzip => {
            let decoded = gzip_decode(&raw)?;
            Ok(decoded.len() as u64)
        }
    }
}

/// Read helper shared with the server: an archive entry whose declared size
/// outruns the bytes actually present stops extraction (`corrupt-archive`),
/// per the ADR's "output that exceeds its declared size" rule.
///
/// # Errors
/// `corrupt-archive` when the read comes up short.
pub fn read_exact_sized(r: &mut impl Read, size: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0u8; size];
    r.read_exact(&mut buf).map_err(|_| {
        Error::new(
            ErrorKind::CorruptArchive,
            "entry content is shorter than declared",
        )
    })?;
    Ok(buf)
}
