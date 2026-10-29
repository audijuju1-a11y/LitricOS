//! A bounded watcher for the folder the Files window is showing.
//!
//! # Why this is a check, not an inotify subscription
//!
//! The shell crate is `#![forbid(unsafe_code)]` and the workspace takes no new
//! dependencies, so there is no safe way to call `inotify_init1` here and no
//! watcher crate to lean on. What this module does instead is the honest
//! bounded version: the currently listed folder is read through the same
//! [`crate::fsbridge::FsBridge`] every other Files operation uses, at most
//! once per [`INTERVAL_MS`], and a digest of the listing is compared with the
//! one the last read produced. A different digest marks the listing stale, and
//! the next frame reads the folder once and draws what is actually on disk.
//!
//! # What it costs
//!
//! One `read_dir` a second while a Files window is visible, the session is
//! unlocked and the listing is settled — not one per frame, and nothing at all
//! while no Files window exists. The frame loop already wakes at least once a
//! second for the top-bar clock, so this adds no timer and no thread: the
//! check rides on the wake-up that was already happening.
//!
//! # What it notices
//!
//! Everything the table draws: entries appearing, disappearing, being renamed,
//! and changes to an entry's size or modification second. A file whose bytes
//! change without its size or mtime second changing is not visible in the
//! table either, so the digest does not claim to have noticed it.

use crate::apps::FileKind;
use crate::fsbridge::Entry;

/// The shortest time between two checks of the open folder.
///
/// One second matches the top-bar clock's own resolution and the shell's idle
/// wake cap, so the check never asks the loop to wake for it.
pub const INTERVAL_MS: u64 = 1_000;

/// The digest of one listing as the Files table would show it.
///
/// Names, kinds, sizes, modification seconds and order all feed the hash, so
/// a listing that would draw differently produces a different digest. Two
/// different listings sharing a 64-bit value would only delay a refresh, never
/// draw a wrong table: the refresh itself always comes from a real read.
#[must_use]
pub fn digest(entries: &[Entry]) -> u64 {
    let mut d = Fnv::new();
    d.u64(u64::try_from(entries.len()).unwrap_or(u64::MAX));
    for e in entries {
        d.bytes(e.name.as_bytes());
        // A separator that cannot occur in a name keeps two adjacent fields
        // from running together, so "ab" + 1 and "a" + 1 + "b" cannot meet.
        d.byte(0xFF);
        d.u64(kind_code(e.kind));
        d.u64(e.size);
        d.u64(e.mtime);
    }
    d.0
}

const fn kind_code(kind: FileKind) -> u64 {
    match kind {
        FileKind::Dir => 0,
        FileKind::File => 1,
        FileKind::Link => 2,
        FileKind::Other => 3,
    }
}

/// FNV-1a, 64-bit. Small, allocation-free and deterministic: the digest is a
/// change detector for one process, not a security primitive.
struct Fnv(u64);

impl Fnv {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn byte(&mut self, b: u8) {
        self.0 ^= u64::from(b);
        self.0 = self.0.wrapping_mul(Self::PRIME);
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.byte(b);
        }
    }

    fn u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.byte(b);
        }
    }
}

/// What the watcher remembers between checks.
///
/// It is armed by a real directory read (`refresh_files`), so a listing that
/// was placed in the model by other means is never mistaken for something the
/// watcher has seen on disk.
#[derive(Debug, Clone, Default)]
pub struct Watch {
    /// The logical path whose listing the last recorded read produced.
    path: Option<String>,
    /// Digest of that listing, or `None` when the read failed.
    digest: Option<u64>,
    /// Whether the last recorded read failed.
    failed: bool,
    /// The earliest shell time at which the next check may read.
    next_ms: u64,
    /// Directory reads this watcher has made, for the diagnostics line.
    reads: u64,
}

impl Watch {
    /// Record the listing a real read produced.
    pub fn note_read(&mut self, path: &str, entries: &[Entry], now_ms: u64) {
        self.path = Some(path.to_owned());
        self.digest = Some(digest(entries));
        self.failed = false;
        self.next_ms = now_ms.saturating_add(INTERVAL_MS);
    }

    /// Record that a real read of `path` failed.
    pub fn note_failure(&mut self, path: &str, now_ms: u64) {
        self.path = Some(path.to_owned());
        self.digest = None;
        self.failed = true;
        self.next_ms = now_ms.saturating_add(INTERVAL_MS);
    }

    /// Forget the recorded listing because what the table shows changed for a
    /// reason other than the folder: the user turned hidden entries on or off.
    /// The next real read arms the watcher again with the new visibility.
    pub fn reset(&mut self) {
        self.path = None;
        self.digest = None;
        self.failed = false;
    }

    /// Postpone the next check without reading.
    ///
    /// Used when the folder cannot be watched right now — no visible Files
    /// window, a locked session, a listing still waiting for its read, or no
    /// file service on this platform — so a gated second is one skipped check
    /// rather than an attempt on every frame.
    pub fn defer(&mut self, now_ms: u64) {
        self.next_ms = now_ms.saturating_add(INTERVAL_MS);
    }

    /// Whether this path is the one last read and the interval has elapsed.
    #[must_use]
    pub fn check_due(&self, path: &str, now_ms: u64) -> bool {
        self.path.as_deref() == Some(path) && now_ms >= self.next_ms
    }

    /// Compare a fresh read with the recorded listing.
    ///
    /// Returns whether the table would change. The recorded state always moves
    /// to the fresh read, so an unchanged listing costs nothing until the next
    /// interval and a changed one refreshes once, not on every following
    /// check.
    pub fn observe(&mut self, path: &str, entries: &[Entry], now_ms: u64) -> bool {
        self.reads = self.reads.saturating_add(1);
        let fresh = digest(entries);
        let changed =
            self.path.as_deref() == Some(path) && (self.failed || self.digest != Some(fresh));
        self.path = Some(path.to_owned());
        self.digest = Some(fresh);
        self.failed = false;
        self.next_ms = now_ms.saturating_add(INTERVAL_MS);
        changed
    }

    /// Compare a failed read with the recorded listing.
    ///
    /// A folder that was readable and is not any more has changed: the table
    /// must stop showing rows it can no longer stand behind. A failure that
    /// repeats is not a new change, so it is reported once.
    pub fn observe_failure(&mut self, path: &str, now_ms: u64) -> bool {
        self.reads = self.reads.saturating_add(1);
        let changed = self.path.as_deref() == Some(path) && !self.failed;
        self.path = Some(path.to_owned());
        self.digest = None;
        self.failed = true;
        self.next_ms = now_ms.saturating_add(INTERVAL_MS);
        changed
    }

    /// The path the watcher is armed for, if any.
    #[must_use]
    pub fn watched(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Directory reads the watcher has made since the shell started.
    #[must_use]
    pub fn reads(&self) -> u64 {
        self.reads
    }
}

#[cfg(test)]
mod tests {
    use super::{digest, Watch, INTERVAL_MS};
    use crate::apps::FileKind;
    use crate::fsbridge::Entry;

    fn entry(name: &str, kind: FileKind, size: u64, mtime: u64) -> Entry {
        Entry {
            name: name.to_owned(),
            kind,
            size,
            mtime,
        }
    }

    #[test]
    fn the_digest_changes_when_the_table_would_draw_differently() {
        let base = vec![
            entry("a.txt", FileKind::File, 3, 100),
            entry("b", FileKind::Dir, 0, 90),
        ];
        let d = digest(&base);
        assert_eq!(d, digest(&base), "the digest is deterministic");

        let mut renamed = base.clone();
        renamed
            .first_mut()
            .expect("the fixture has a first entry")
            .name = "c.txt".to_owned();
        assert_ne!(d, digest(&renamed), "a rename is a change");

        let mut resized = base.clone();
        resized
            .first_mut()
            .expect("the fixture has a first entry")
            .size = 4;
        assert_ne!(d, digest(&resized), "a new size is a change");

        let mut touched = base.clone();
        touched
            .get_mut(1)
            .expect("the fixture has a second entry")
            .mtime = 91;
        assert_ne!(d, digest(&touched), "a new modification second is a change");

        let mut retyped = base.clone();
        retyped
            .get_mut(1)
            .expect("the fixture has a second entry")
            .kind = FileKind::Link;
        assert_ne!(d, digest(&retyped), "a new kind is a change");

        let mut reordered = base.clone();
        reordered.reverse();
        assert_ne!(d, digest(&reordered), "the table's order is part of it");

        let mut extra = base.clone();
        extra.push(entry("n", FileKind::Other, 1, 1));
        assert_ne!(d, digest(&extra), "a new entry is a change");
        assert_ne!(d, digest(&[]), "an empty folder has its own digest");
    }

    #[test]
    fn the_watcher_reads_at_most_once_per_interval_and_only_after_a_real_read() {
        let mut w = Watch::default();
        let a = vec![entry("a.txt", FileKind::File, 1, 10)];
        let b = vec![entry("b.txt", FileKind::File, 1, 10)];

        assert!(
            !w.check_due("/Documents", 10_000),
            "an unarmed watcher never reads"
        );
        w.note_read("/Documents", &a, 100);
        assert!(!w.check_due("/Documents", 1_099), "inside the interval");
        assert!(
            !w.check_due("/Downloads", 5_000),
            "another folder is not the one that was read"
        );
        assert!(w.check_due("/Documents", 100 + INTERVAL_MS));

        assert!(
            !w.observe("/Documents", &a, 1_100),
            "the same listing is not a change"
        );
        assert_eq!(w.reads(), 1);
        assert!(
            !w.check_due("/Documents", 1_200),
            "observe opened a new interval"
        );

        assert!(
            w.observe("/Documents", &b, 2_100),
            "a different listing is a change"
        );
        assert_eq!(w.reads(), 2);
        assert!(!w.observe("/Documents", &b, 3_100), "and is reported once");
    }

    #[test]
    fn losing_and_regaining_the_folder_are_each_one_change() {
        let mut w = Watch::default();
        let entries = vec![entry("a.txt", FileKind::File, 1, 10)];
        w.note_read("/Documents", &entries, 0);

        assert!(
            w.observe_failure("/Documents", 1_000),
            "a readable folder that stops being readable is a change"
        );
        assert!(
            !w.observe_failure("/Documents", 2_000),
            "a repeat failure is not a second change"
        );
        assert!(
            w.observe("/Documents", &entries, 3_000),
            "the folder coming back is a change"
        );
        assert!(!w.observe("/Documents", &entries, 4_000));
        assert_eq!(w.reads(), 4);
    }

    #[test]
    fn a_deferred_check_is_not_retried_until_the_next_interval() {
        let mut w = Watch::default();
        w.note_read("/Documents", &[], 0);
        assert!(w.check_due("/Documents", 1_000));
        w.defer(1_000);
        assert!(!w.check_due("/Documents", 1_500));
        assert!(w.check_due("/Documents", 2_000));
        assert_eq!(w.reads(), 0, "deferring reads nothing");
    }
}
