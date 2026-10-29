//! Quarantine: moving a detected file out of the way, and putting it back.
//!
//! One owner for both halves, so Defender's window and any future caller move
//! files through the same rules. The moves go through [`FsBridge`], the same
//! validated filesystem the Files window uses; there is no second backend and
//! no direct filesystem call in the shell.
//!
//! # The guarantees
//!
//! * A file is only taken from the folders the Fast scan covers. Deep scan
//!   reports what it finds in system folders, but it must not move the OS out
//!   from under the running machine, so those findings are refused here.
//! * Links and folders are refused. A link is not the file it points at, and a
//!   folder is not a finding.
//! * The quarantined copy never replaces an earlier one: names are numbered
//!   until a free one is found, and the bridge's own create-new and
//!   rename-never-overwrite rules back that up.
//! * Every entry has a record on disk, so the list survives a restart and a
//!   restore still knows where the file came from.
//! * Every new record carries the SHA-256 of the bytes that were contained, and
//!   restore checks it before moving anything. Bytes that no longer match are
//!   refused and the contained copy is left alone.
//! * A failure leaves the source where it was, or says plainly that it could
//!   not. No path in this module reports success it did not observe.

use std::io::Read;

use crate::apps::ScanMode;
use crate::fsbridge::{FsBridge, FsError};
use litric_defender::sha256;

/// Where quarantined files and their records live, relative to the storage
/// root. Deliberately outside every scan root so a scan never re-reports what
/// is already contained.
pub const DIR: &str = "/Quarantine";

/// The suffix of a record. A record is named after the file it describes, so a
/// person looking at the folder can pair them without opening anything.
pub const RECORD_SUFFIX: &str = ".quarantine";

/// The current record format: header, digest, size, time, reason, original.
const HEADER: &str = "LITRIC-QUARANTINE 2";

/// Records written before integrity was recorded. They are still honoured, and
/// the window says so rather than inventing a digest.
const LEGACY_HEADER: &str = "LITRIC-QUARANTINE 1";

/// Read buffer for hashing. Fixed, so peak memory per file is this, never the
/// file size.
const HASH_CHUNK: usize = 8192;

/// Whether a contained file still matches the bytes that were quarantined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Integrity {
    /// A digest is recorded but has not been checked in this session.
    NotChecked,
    /// The bytes hash to the recorded digest.
    Verified,
    /// The record predates integrity records, so there is nothing to compare.
    NotRecorded,
    /// The bytes no longer hash to the recorded digest.
    Mismatch,
}

impl Integrity {
    /// The words the window shows. Plain language, no digest and no code.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotChecked => "Not checked",
            Self::Verified => "Verified",
            Self::NotRecorded => "No integrity record",
            Self::Mismatch => "Does not match",
        }
    }
}

/// One contained file, as the window shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quarantined {
    /// The name it has inside the quarantine folder.
    pub stored: String,
    /// Where it was taken from, as the user knew it.
    pub original: String,
    /// Why it was taken, in the user's words.
    pub reason: String,
    /// When it was taken, Unix seconds.
    pub time: u64,
    /// How large it was.
    pub size: u64,
    /// The recorded SHA-256, lowercase hex. `None` for a legacy record.
    pub hash: Option<String>,
    /// What this session knows about the bytes. Set by [`verify`] and by a
    /// restore; never guessed.
    pub integrity: Integrity,
}

/// A record as it is read from disk.
struct Record {
    hash: Option<String>,
    size: u64,
    time: u64,
    reason: String,
    original: String,
}

/// Whether a logical path is inside a folder Defender protects.
///
/// This is the Fast scan's scope: the user's own documents. It is also what
/// keeps a malformed or tampered record from moving a system file on restore.
#[must_use]
pub fn in_scope(path: &str) -> bool {
    if !path.starts_with('/') || path.contains("/../") || path.ends_with("/..") {
        return false;
    }
    ScanMode::Fast.roots().iter().any(|root| {
        *path == **root
            || path
                .strip_prefix(*root)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// The folder a logical path is in.
fn parent_of(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some(("", _)) | None => "/",
        Some((parent, _)) => parent,
    }
}

/// The final name in a logical path.
fn name_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// A plain sentence for a failed bridge call. No internal path, no error code.
fn sentence(error: &FsError) -> String {
    match error {
        FsError::Unsupported => "the file service is not available".to_owned(),
        FsError::NotFound => "that file could not be found".to_owned(),
        FsError::Exists => "an item with that name already exists".to_owned(),
        FsError::Denied => "permission was refused".to_owned(),
        FsError::ReadOnly => "that folder can only be read".to_owned(),
        FsError::Full => "there is no room left".to_owned(),
        FsError::BadName => "that name cannot be used".to_owned(),
        FsError::NotADirectory => "that folder is not available".to_owned(),
        FsError::Io(_) => "the file could not be moved".to_owned(),
    }
}

/// Parse one record, old format or new. `None` means the record is not one this
/// build understands; the caller drops it rather than guessing.
fn parse_record(text: &str) -> Option<Record> {
    let mut lines = text.split('\n');
    let header = lines.next()?;
    let hash = match header {
        HEADER => {
            let line = lines.next()?;
            let digest = line.trim();
            if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            Some(digest.to_ascii_lowercase())
        }
        LEGACY_HEADER => None,
        _ => return None,
    };
    let size = lines.next()?.parse::<u64>().ok()?;
    let time = lines.next()?.parse::<u64>().ok()?;
    let reason = lines.next()?.to_owned();
    let original = lines.collect::<Vec<&str>>().join("\n");
    if original.is_empty() {
        return None;
    }
    Some(Record {
        hash,
        size,
        time,
        reason,
        original,
    })
}

/// Hash one host path in fixed-size chunks.
///
/// The buffer is on the stack and the digest is streamed, so a very large file
/// costs time but not memory. This runs on the caller's thread; a click that
/// hashes a multi-gigabyte file would occupy the UI thread for the duration,
/// which is why the window only does it on an explicit action and on restore,
/// never while drawing.
fn hash_path(path: &std::path::Path) -> Result<[u8; 32], String> {
    let meta = std::fs::symlink_metadata(path)
        .map_err(|_| "the contained file could not be read".to_owned())?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err("the contained file could not be read".to_owned());
    }
    let mut file =
        std::fs::File::open(path).map_err(|_| "the contained file could not be read".to_owned())?;
    let mut hasher = sha256::Sha256::new();
    let mut buffer = [0u8; HASH_CHUNK];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "the contained file could not be read".to_owned())?;
        if read == 0 {
            break;
        }
        let chunk = buffer
            .get(..read)
            .ok_or_else(|| "the contained file could not be read".to_owned())?;
        hasher.update(chunk);
    }
    Ok(hasher.finish())
}

/// The host path of one contained file.
fn contained_path(bridge: &FsBridge, stored: &str) -> Result<std::path::PathBuf, String> {
    bridge
        .host_path(&format!("{DIR}/{stored}"))
        .map_err(|e| sentence(&e))
}

/// Check one contained file against its record.
///
/// # Errors
/// A sentence when the record cannot be read or the contained file cannot be
/// opened. A legacy record is not an error: it reports [`Integrity::NotRecorded`].
pub fn verify(bridge: &FsBridge, stored: &str) -> Result<Integrity, String> {
    if stored.is_empty() || stored.contains('/') || stored.contains('\\') {
        return Err("that quarantine entry could not be found".to_owned());
    }
    let text = bridge
        .read_text(&format!("{DIR}/{stored}{RECORD_SUFFIX}"))
        .map_err(|_| "that quarantine entry could not be found".to_owned())?;
    let record =
        parse_record(&text).ok_or_else(|| "that quarantine entry could not be read".to_owned())?;
    let Some(expected) = record.hash else {
        return Ok(Integrity::NotRecorded);
    };
    let path = contained_path(bridge, stored)?;
    let Some(meta) = std::fs::symlink_metadata(&path).ok() else {
        return Err("that quarantined file is no longer there".to_owned());
    };
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Ok(Integrity::Mismatch);
    }
    let actual = hash_path(&path)?;
    if sha256::hex(&actual) == expected {
        Ok(Integrity::Verified)
    } else {
        Ok(Integrity::Mismatch)
    }
}

/// Take one file out of its folder and contain it.
///
/// `now` is passed in rather than read here so the record is deterministic in
/// tests and the shell has one clock.
///
/// # Errors
/// A sentence for the window: the file is out of scope, is gone, is a link or
/// folder, cannot be read for its digest, or the move or its record failed. On
/// failure the source is left in place unless the sentence says otherwise.
pub fn quarantine(
    bridge: &FsBridge,
    path: &str,
    reason: &str,
    now: u64,
) -> Result<Quarantined, String> {
    if !in_scope(path) {
        return Err("that file is outside the folders Defender protects".to_owned());
    }
    let host = bridge.host_path(path).map_err(|e| sentence(&e))?;
    let meta = std::fs::symlink_metadata(&host).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "that file could not be found".to_owned()
        } else {
            "that file could not be read".to_owned()
        }
    })?;
    if meta.file_type().is_symlink() {
        return Err("that is a link, not a file".to_owned());
    }
    if !meta.is_file() {
        return Err("only files can be quarantined".to_owned());
    }
    // A checked path refuses links in every parent, but the confinement is
    // stated once more against the real root so a mount or a Windows-style
    // junction cannot make the move cross out of the share.
    let root = bridge.host_path("/").map_err(|e| sentence(&e))?;
    let real_root =
        std::fs::canonicalize(&root).map_err(|_| sentence(&FsError::Io(String::new())))?;
    let real =
        std::fs::canonicalize(&host).map_err(|_| "that file could not be found".to_owned())?;
    if !real.starts_with(&real_root) {
        return Err("that file is outside the folders Defender protects".to_owned());
    }

    // Hash before the move: a file whose bytes cannot be read is not contained
    // on a promise. The digest describes exactly what the move is about to take.
    let digest = sha256::hex(
        &hash_path(&host)
            .map_err(|_| "the file could not be read, so it was not quarantined".to_owned())?,
    );

    if bridge.mkdir("/", "Quarantine").is_err() && !bridge.exists(DIR).unwrap_or(false) {
        return Err("the quarantine folder could not be created".to_owned());
    }

    // A free name, never an overwrite. The bridge's move would invent a name on
    // a clash, so the check happens here and the result is verified below.
    let name = name_of(path);
    // A hidden name would make its record hidden too, and the list skips
    // hidden entries; contain it under a visible name instead.
    let base = if name.starts_with('.') {
        format!("contained-{}", name.trim_start_matches('.'))
    } else {
        name.to_owned()
    };
    let mut attempt = 0_u32;
    let stored = loop {
        let candidate = if attempt == 0 {
            base.to_owned()
        } else {
            format!("{base}.{attempt}")
        };
        let taken = bridge
            .exists(&format!("{DIR}/{candidate}"))
            .unwrap_or(false)
            || bridge
                .exists(&format!("{DIR}/{candidate}{RECORD_SUFFIX}"))
                .unwrap_or(false);
        if !taken {
            break candidate;
        }
        attempt = attempt.saturating_add(1);
        if attempt > 500 {
            return Err("the quarantine folder has no free name for that file".to_owned());
        }
    };

    let moved = bridge.move_item(path, DIR).map_err(|e| {
        let text = sentence(&e);
        format!("the file was left where it was: {text}")
    })?;
    if moved != stored {
        // The move chose another name (a race with another writer). Put the
        // chosen name right; if even that fails, move it back rather than
        // leaving an entry nobody can account for.
        if bridge.rename(DIR, &moved, &stored).is_err() {
            let rolled_back = bridge
                .move_item(&format!("{DIR}/{moved}"), parent_of(path))
                .is_ok();
            return Err(if rolled_back {
                "the file could not be quarantined, so it was left where it was".to_owned()
            } else {
                "the file could not be quarantined; check the Quarantine folder".to_owned()
            });
        }
    }

    let record = format!(
        "{HEADER}\n{digest}\n{}\n{now}\n{}\n{path}",
        meta.len(),
        reason.replace('\n', " ")
    );
    if bridge
        .write_text(DIR, &format!("{stored}{RECORD_SUFFIX}"), &record)
        .is_err()
    {
        let rolled_back = bridge
            .move_item(&format!("{DIR}/{stored}"), parent_of(path))
            .is_ok();
        return Err(if rolled_back {
            "the file could not be quarantined, so it was left where it was".to_owned()
        } else {
            "the file was moved but could not be recorded; check the Quarantine folder".to_owned()
        });
    }

    Ok(Quarantined {
        stored,
        original: path.to_owned(),
        reason: reason.to_owned(),
        time: now,
        size: meta.len(),
        hash: Some(digest),
        integrity: Integrity::NotChecked,
    })
}

/// Read the records on disk.
///
/// A record whose file has gone is left out: the list is what is actually
/// contained now, not what once was. An unreadable folder is an error, not an
/// empty list — "nothing quarantined" and "the list could not be read" are
/// different statements.
///
/// # Errors
/// A sentence when the folder exists but cannot be read.
pub fn list(bridge: &FsBridge) -> Result<Vec<Quarantined>, String> {
    if !bridge.exists(DIR).unwrap_or(false) {
        return Ok(Vec::new());
    }
    let entries = bridge
        .list_dir(DIR)
        .map_err(|_| "the quarantine folder could not be read".to_owned())?;
    let mut out = Vec::new();
    for entry in entries {
        let Some(stored) = entry.name.strip_suffix(RECORD_SUFFIX) else {
            continue;
        };
        if stored.is_empty() {
            continue;
        }
        let Ok(text) = bridge.read_text(&format!("{DIR}/{}", entry.name)) else {
            continue;
        };
        let Some(record) = parse_record(&text) else {
            continue;
        };
        if !record.original.starts_with('/')
            || !in_scope(&record.original)
            || !bridge.exists(&format!("{DIR}/{stored}")).unwrap_or(false)
        {
            continue;
        }
        let integrity = if record.hash.is_some() {
            Integrity::NotChecked
        } else {
            Integrity::NotRecorded
        };
        out.push(Quarantined {
            stored: stored.to_owned(),
            original: record.original,
            reason: record.reason,
            time: record.time,
            size: record.size,
            hash: record.hash,
            integrity,
        });
    }
    out.sort_by(|a, b| b.time.cmp(&a.time).then_with(|| a.stored.cmp(&b.stored)));
    Ok(out)
}

/// The result of putting a contained file back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Restored {
    /// Where it went.
    pub original: String,
    /// Whether the bytes were checked against the record after the move.
    pub integrity: Integrity,
}

/// Put one contained file back where it came from.
///
/// The record is the only source of the original path, and it is checked again
/// before anything moves. Nothing is overwritten: an item already at the
/// original path, or a missing folder, is refused and the quarantined copy
/// stays intact. When the record carries a digest, the contained bytes are
/// hashed first; a mismatch refuses the restore and keeps the copy.
///
/// # Errors
/// A sentence for the window when the name is unknown, the original path is
/// taken or its folder is gone, the contained bytes no longer match, or the
/// move fails.
pub fn restore(bridge: &FsBridge, stored: &str) -> Result<Restored, String> {
    if stored.is_empty() || stored.contains('/') || stored.contains('\\') {
        return Err("that quarantine entry could not be found".to_owned());
    }
    let record = format!("{DIR}/{stored}{RECORD_SUFFIX}");
    let text = bridge
        .read_text(&record)
        .map_err(|_| "that quarantine entry could not be found".to_owned())?;
    let record =
        parse_record(&text).ok_or_else(|| "that quarantine entry could not be read".to_owned())?;
    if !record.original.starts_with('/') || !in_scope(&record.original) {
        return Err("that quarantine entry does not name a safe place".to_owned());
    }
    if !bridge.exists(&format!("{DIR}/{stored}")).unwrap_or(false) {
        return Err("that quarantined file is no longer there".to_owned());
    }
    if bridge.exists(&record.original).unwrap_or(false) {
        return Err("something already exists at the original place".to_owned());
    }
    let parent = parent_of(&record.original);
    if !bridge.exists(parent).unwrap_or(false) {
        return Err("the folder it came from is gone".to_owned());
    }

    // Check the bytes before moving them. A digest that no longer matches is
    // not restored over the user's folder, and the contained copy is kept so
    // the user can inspect it or an administrator can.
    let expected = record.hash.as_deref();
    if let Some(expected) = expected {
        let path = contained_path(bridge, stored)?;
        let Some(meta) = std::fs::symlink_metadata(&path).ok() else {
            return Err("that quarantined file is no longer there".to_owned());
        };
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(
                "the contained file no longer matches what was quarantined, so it was not restored"
                    .to_owned(),
            );
        }
        let actual = hash_path(&path)?;
        if sha256::hex(&actual) != expected {
            return Err(
                "the contained file no longer matches what was quarantined, so it was not restored"
                    .to_owned(),
            );
        }
    }

    let wanted = name_of(&record.original);
    let moved = bridge
        .move_item(&format!("{DIR}/{stored}"), parent)
        .map_err(|e| format!("the file could not be put back: {}", sentence(&e)))?;
    if moved != wanted {
        // The bridge invented a name because something took the original place
        // between the check and the move. Undo it rather than restoring onto a
        // different name and telling the user it went back.
        let _ = bridge.rename(parent, &moved, &format!("{wanted}.restored"));
        let _ = bridge.move_item(&format!("{parent}/{wanted}.restored"), DIR);
        return Err("something already exists at the original place".to_owned());
    }

    // Re-check the bytes where they landed. A mismatch here means something
    // changed them during the move; the file is back, but the window must not
    // call that confirmed.
    let integrity = if let Some(expected) = expected {
        let restored = bridge
            .host_path(&record.original)
            .map_err(|e| sentence(&e))?;
        match hash_path(&restored) {
            Ok(actual) if sha256::hex(&actual) == expected => Integrity::Verified,
            _ => Integrity::Mismatch,
        }
    } else {
        Integrity::NotRecorded
    };

    // The record is no longer needed. If it cannot be removed the file is still
    // back, and a later listing drops the record because its file has gone.
    let _ = bridge.delete(&format!("{DIR}/{stored}{RECORD_SUFFIX}"));
    Ok(Restored {
        original: record.original,
        integrity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bridge(tag: &str) -> (FsBridge, std::path::PathBuf) {
        let root =
            std::env::temp_dir().join(format!("litric-quarantine-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("Documents")).unwrap();
        let zone = litric_locale::Regional::fallback("test").zone;
        let b = FsBridge::open(&root, zone).unwrap();
        (b, root)
    }

    #[test]
    fn quarantine_contains_the_file_and_restore_puts_it_back() {
        let (b, root) = bridge("roundtrip");
        std::fs::write(root.join("Documents/eicar.com"), b"test threat").unwrap();
        let entry = quarantine(&b, "/Documents/eicar.com", "Threat found", 42).unwrap();
        assert_eq!(entry.original, "/Documents/eicar.com");
        let expected_hash = sha256::hex(&sha256::digest(b"test threat"));
        assert_eq!(
            entry.hash.as_deref(),
            Some(expected_hash.as_str()),
            "the record carries the digest of the contained bytes"
        );
        assert!(!root.join("Documents/eicar.com").exists(), "source is gone");
        assert!(root.join("Quarantine/eicar.com").exists(), "contained");
        assert!(root.join("Quarantine/eicar.com.quarantine").exists());
        let listed = list(&b).unwrap();
        assert_eq!(listed.len(), 1);
        let found = listed.first().expect("one contained entry");
        assert_eq!(found.size, 11);
        assert_eq!(found.reason, "Threat found");
        assert_eq!(found.integrity, Integrity::NotChecked);
        assert_eq!(verify(&b, "eicar.com").unwrap(), Integrity::Verified);
        let back = restore(&b, "eicar.com").unwrap();
        assert_eq!(back.original, "/Documents/eicar.com");
        assert_eq!(back.integrity, Integrity::Verified);
        assert!(root.join("Documents/eicar.com").exists(), "back in place");
        assert!(!root.join("Quarantine/eicar.com").exists());
        assert!(list(&b).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_second_quarantine_never_replaces_the_first() {
        let (b, root) = bridge("duplicate");
        std::fs::write(root.join("Documents/note.txt"), b"one").unwrap();
        let first = quarantine(&b, "/Documents/note.txt", "Threat found", 1).unwrap();
        std::fs::write(root.join("Documents/note.txt"), b"two").unwrap();
        let second = quarantine(&b, "/Documents/note.txt", "Threat found", 2).unwrap();
        assert_ne!(first.stored, second.stored, "two separate entries");
        assert!(root.join("Quarantine/note.txt").exists());
        assert!(root.join("Quarantine/note.txt.1").exists());
        assert_eq!(list(&b).unwrap().len(), 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn out_of_scope_and_missing_files_are_refused_without_moving_anything() {
        let (b, root) = bridge("refusals");
        std::fs::create_dir_all(root.join("System")).unwrap();
        std::fs::write(root.join("System/os.bin"), b"system").unwrap();
        let outside = quarantine(&b, "/System/os.bin", "Threat found", 1).unwrap_err();
        assert!(outside.contains("outside the folders"), "{outside}");
        assert!(root.join("System/os.bin").exists(), "source untouched");

        let missing = quarantine(&b, "/Downloads/gone.bin", "Threat found", 1).unwrap_err();
        assert!(missing.contains("could not be found"), "{missing}");
        assert!(!root.join("Quarantine/gone.bin").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn restore_refuses_to_overwrite_or_to_use_a_tampered_record() {
        let (b, root) = bridge("restore-refusals");
        std::fs::write(root.join("Documents/evil.bin"), b"bad").unwrap();
        quarantine(&b, "/Documents/evil.bin", "Threat found", 1).unwrap();
        std::fs::write(root.join("Documents/evil.bin"), b"new file").unwrap();
        let taken = restore(&b, "evil.bin").unwrap_err();
        assert!(taken.contains("already exists"), "{taken}");
        assert!(root.join("Quarantine/evil.bin").exists(), "copy kept");

        // A legacy record that names a system folder is not followed either.
        std::fs::write(root.join("Quarantine/system.bin"), b"payload").unwrap();
        std::fs::write(
            root.join("Quarantine/system.bin.quarantine"),
            format!("{LEGACY_HEADER}\n7\n1\nThreat found\n/System/passwd"),
        )
        .unwrap();
        let tampered = restore(&b, "system.bin").unwrap_err();
        assert!(
            tampered.contains("does not name a safe place"),
            "{tampered}"
        );
        assert!(root.join("Quarantine/system.bin").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn tampered_bytes_are_refused_and_kept() {
        let (b, root) = bridge("tampered");
        std::fs::write(root.join("Documents/evil.bin"), b"the original threat").unwrap();
        quarantine(&b, "/Documents/evil.bin", "Threat found", 1).unwrap();
        std::fs::write(root.join("Quarantine/evil.bin"), b"replaced bytes").unwrap();

        assert_eq!(verify(&b, "evil.bin").unwrap(), Integrity::Mismatch);
        let refused = restore(&b, "evil.bin").unwrap_err();
        assert!(
            refused.contains("no longer matches what was quarantined"),
            "{refused}"
        );
        assert!(
            !root.join("Documents/evil.bin").exists(),
            "nothing was put back"
        );
        assert_eq!(
            std::fs::read(root.join("Quarantine/evil.bin")).unwrap(),
            b"replaced bytes",
            "the tampered copy is intact for inspection"
        );
        let listed = list(&b).unwrap();
        assert_eq!(listed.len(), 1, "the entry is still listed");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_legacy_record_restores_unverified() {
        let (b, root) = bridge("legacy");
        std::fs::create_dir_all(root.join("Quarantine")).unwrap();
        std::fs::write(root.join("Quarantine/old.bin"), b"old bytes").unwrap();
        std::fs::write(
            root.join("Quarantine/old.bin.quarantine"),
            format!("{LEGACY_HEADER}\n9\n1\nThreat found\n/Documents/old.bin"),
        )
        .unwrap();
        let listed = list(&b).unwrap();
        assert_eq!(
            listed.first().map(|entry| entry.integrity),
            Some(Integrity::NotRecorded)
        );
        let back = restore(&b, "old.bin").unwrap();
        assert_eq!(back.original, "/Documents/old.bin");
        assert_eq!(back.integrity, Integrity::NotRecorded);
        assert_eq!(
            std::fs::read(root.join("Documents/old.bin")).unwrap(),
            b"old bytes"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn a_link_is_refused_and_its_target_is_left_alone() {
        let (b, root) = bridge("links");
        std::fs::write(root.join("outside.txt"), b"outside").unwrap();
        std::os::unix::fs::symlink(root.join("outside.txt"), root.join("Documents/link")).unwrap();
        let refused = quarantine(&b, "/Documents/link", "Threat found", 1).unwrap_err();
        assert!(refused.contains("link"), "{refused}");
        assert!(root.join("outside.txt").exists(), "target untouched");
        assert!(root.join("Documents/link").exists(), "link untouched");
        let _ = std::fs::remove_dir_all(&root);
    }
}
