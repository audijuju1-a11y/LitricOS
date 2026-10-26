//! The queue itself: a bounded list, and the file that outlives the process.
//!
//! # What the file is for
//!
//! The whole reason this daemon exists is that the queue must survive the
//! desktop being down. So the queue is not "in memory, written out on shutdown":
//! a `kill -9` is the failure a queue of this kind actually meets. Every mutation
//! writes the whole record atomically — temporary file in the same directory,
//! `fsync`, `rename`, `fsync` the directory — so the file on disk is always a
//! complete queue, either the one before the change or the one after.
//!
//! A queue is at most [`Queue::CAPACITY`] notifications of a few hundred bytes
//! each, so rewriting it whole costs less than the `fsync` that makes it durable.
//! An append-only log would avoid the rewrite and need compaction, a second file
//! format and a recovery path for a half-written record: more machinery to keep a
//! few kilobytes.
//!
//! # Bounded on purpose
//!
//! Anything that accepts text from any program on the machine is a way to fill
//! memory. [`Queue::CAPACITY`] is the ceiling, and it evicts the oldest
//! *non-critical* notification rather than simply the oldest — the same rule the
//! shell's own centre uses, because a critical notification that can be pushed
//! out by a hundred low ones is not critical.
//!
//! # Failure to write
//!
//! [`Queue::post`] never fails. The notification is in memory either way, and
//! losing the message because the disk is full would be worse than losing the
//! promise that it survives a reboot. What the caller gets is a separate answer:
//! the daemon reports `persisted=0` and logs the reason, so the failure is
//! visible instead of implied.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::proto::{note_lines, Note, Pending, Priority};

/// The schema version written to the queue file.
///
/// Bumped when a field changes meaning, not when one is added: a file from a
/// newer build still parses every key this build knows, and an unknown key keeps
/// the default, exactly as `settings.conf` does.
pub const VERSION: u8 = 1;

/// The queue.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Queue {
    items: Vec<Note>,
    next_id: u32,
}

impl Queue {
    /// The most a queue holds before the oldest non-critical note is dropped.
    pub const CAPACITY: usize = 64;

    /// Accept a notification. Returns its id; never fails.
    pub fn post(&mut self, pending: &Pending, now_ms: u64) -> u32 {
        let id = self.next_id.saturating_add(1);
        self.next_id = id;
        self.items.insert(
            0,
            Note {
                id,
                app: pending.app.clone(),
                priority: pending.priority,
                title: pending.title.clone(),
                body: pending.body.clone(),
                created_ms: now_ms,
                expires_ms: pending.expires_at(now_ms),
            },
        );
        while self.items.len() > Self::CAPACITY {
            let victim = self
                .items
                .iter()
                .rposition(|n| n.priority != Priority::Critical)
                .unwrap_or_else(|| self.items.len().saturating_sub(1));
            self.items.remove(victim);
        }
        id
    }

    /// Everything queued, newest first.
    #[must_use]
    pub fn list(&self) -> &[Note] {
        &self.items
    }

    /// How many are queued.
    #[must_use]
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Remove one by id. `true` when something was removed.
    pub fn dismiss(&mut self, id: u32) -> bool {
        let before = self.items.len();
        self.items.retain(|n| n.id != id);
        before != self.items.len()
    }

    /// Remove everything. Returns how many went.
    pub fn clear(&mut self) -> usize {
        let gone = self.items.len();
        self.items.clear();
        gone
    }

    /// Hand the queue over and empty it.
    ///
    /// This is the delivery primitive: `TAKE` hands a note to the desktop
    /// exactly once. Returning the notes and clearing in one call means there is
    /// no window in which a note has been read but not removed, which would
    /// deliver it twice after a crash.
    pub fn take(&mut self) -> Vec<Note> {
        core::mem::take(&mut self.items)
    }

    /// Drop anything past its expiry. Returns how many went.
    ///
    /// A notification with a five-second lifetime that was posted before a
    /// reboot must not appear afterwards as though it had just arrived.
    pub fn expire(&mut self, now_ms: u64) -> usize {
        let before = self.items.len();
        self.items
            .retain(|n| n.expires_ms.is_none_or(|at| at > now_ms));
        before.saturating_sub(self.items.len())
    }

    /// The id the next post will get.
    #[must_use]
    pub fn next_id(&self) -> u32 {
        self.next_id
    }
}

/// Serialize the whole queue.
#[must_use]
pub fn encode(queue: &Queue) -> String {
    let mut out = String::new();
    out.push_str(&format!("version={VERSION}\n"));
    out.push_str(&format!("next_id={}\n", queue.next_id));
    for note in queue.list() {
        out.push_str("note\n");
        out.push_str(&format!("id={}\n", note.id));
        for line in note_lines(note) {
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("end\n");
    }
    out
}

/// Parse a queue file.
///
/// Never fails: a corrupt or truncated record produces the notifications it can
/// still read, because a desktop that shows three of five notifications is
/// better than one that shows none and reports an error nobody can act on. The
/// caller is told how many lines were unreadable so the daemon can log it.
#[must_use]
pub fn parse(text: &str) -> (Queue, usize) {
    let mut queue = Queue::default();
    let mut skipped = 0usize;
    let mut current: Option<(Option<u32>, Vec<String>)> = None;

    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line {
            "note" => {
                if current.is_some() {
                    skipped = skipped.saturating_add(1);
                }
                current = Some((None, Vec::new()));
            }
            "end" => {
                if let Some((id, lines)) = current.take() {
                    match id.map(|id| crate::proto::note_from_lines(id, &lines)) {
                        Some(Some(note)) => {
                            queue.next_id = queue.next_id.max(note.id);
                            queue.items.push(note);
                        }
                        _ => skipped = skipped.saturating_add(1),
                    }
                } else {
                    skipped = skipped.saturating_add(1);
                }
            }
            other => {
                if let Some((id, lines)) = current.as_mut() {
                    if let Some(raw) = other.strip_prefix("id=") {
                        match raw.trim().parse::<u32>() {
                            Ok(parsed) => *id = Some(parsed),
                            Err(_) => skipped = skipped.saturating_add(1),
                        }
                    } else {
                        lines.push(other.to_owned());
                    }
                } else if let Some(raw) = other.strip_prefix("next_id=") {
                    // Only a floor: a record whose highest note id is larger
                    // than the stored counter would otherwise reissue an id the
                    // desktop has already seen.
                    if let Ok(hint) = raw.trim().parse::<u32>() {
                        queue.next_id = queue.next_id.max(hint);
                    }
                } else if other.starts_with("version=") {
                    // Read and ignored: this build parses every key it knows
                    // whatever the version says, so a file from a newer build
                    // still produces a usable queue.
                } else {
                    skipped = skipped.saturating_add(1);
                }
            }
        }
    }
    if current.is_some() {
        // A record that never reached `end` is not a notification: it may have
        // been half written, and half a message shown as a whole one is a lie.
        skipped = skipped.saturating_add(1);
    }
    (queue, skipped)
}

/// The queue's file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    /// A store for `path`. Nothing is read or written until asked.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The file this store owns.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the queue. `Ok((queue, skipped))`; an absent file is an empty
    /// queue, because "nothing has been posted yet" is not an error.
    ///
    /// # Errors
    /// A file that exists and cannot be read. That is different from an absent
    /// one: reporting it as empty would silently discard every queued
    /// notification and call it a fresh start.
    pub fn load(&self) -> Result<(Queue, usize), String> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok((Queue::default(), 0))
            }
            Err(error) => {
                return Err(format!("cannot read {}: {error}", self.path.display()));
            }
        };
        Ok(parse(&String::from_utf8_lossy(&bytes)))
    }

    /// Write the queue atomically.
    ///
    /// # Errors
    /// Any failure to create the directory, write, sync or rename.
    pub fn save(&self, queue: &Queue) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "the queue path has no parent directory".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create the queue directory: {error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("cannot secure the queue directory: {error}"))?;
        }

        let temp = parent.join(format!("queue.conf.tmp.{}", std::process::id()));
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
                .map_err(|error| format!("cannot create the temporary queue file: {error}"))?;
            file.write_all(encode(queue).as_bytes())
                .map_err(|error| format!("cannot write the queue: {error}"))?;
            file.sync_all()
                .map_err(|error| format!("cannot sync the queue: {error}"))?;
            fs::rename(&temp, &self.path)
                .map_err(|error| format!("cannot commit the queue: {error}"))?;
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
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn pending(title: &str, priority: Priority, ttl_ms: Option<u64>) -> Pending {
        Pending {
            app: "com.litric.test".to_owned(),
            priority,
            title: title.to_owned(),
            body: "body".to_owned(),
            ttl_ms,
        }
    }

    fn fixture(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("litric-notifyd-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root.join("var/lib/litric-notifyd/queue.conf")
    }

    fn cleanup(path: &Path) {
        if let Some(root) = path.ancestors().nth(4) {
            let _ = fs::remove_dir_all(root);
        }
    }

    #[test]
    fn the_newest_notification_is_first_and_ids_are_never_reused() {
        let mut queue = Queue::default();
        let a = queue.post(&pending("a", Priority::Normal, None), 1_000);
        let b = queue.post(&pending("b", Priority::Normal, None), 2_000);
        assert_eq!(queue.list()[0].id, b, "newest first, like the centre");
        assert_eq!(queue.list()[1].id, a);
        assert!(b > a);
    }

    #[test]
    fn a_taken_queue_is_empty_and_hands_over_everything() {
        let mut queue = Queue::default();
        queue.post(&pending("a", Priority::Normal, None), 1_000);
        queue.post(&pending("b", Priority::High, None), 2_000);
        let taken = queue.take();
        assert_eq!(taken.len(), 2);
        assert_eq!(queue.count(), 0, "a delivered note is not delivered again");
        assert!(queue.take().is_empty());
    }

    #[test]
    fn a_critical_note_survives_a_flood_of_low_ones() {
        let mut queue = Queue::default();
        let critical = queue.post(&pending("disk", Priority::Critical, None), 1_000);
        for i in 0..Queue::CAPACITY.saturating_add(20) {
            queue.post(&pending(&format!("n{i}"), Priority::Low, None), 2_000);
        }
        assert_eq!(queue.count(), Queue::CAPACITY);
        assert!(
            queue.list().iter().any(|n| n.id == critical),
            "a critical notification must not be evicted by low ones"
        );
    }

    #[test]
    fn expiry_drops_what_the_sender_timed_and_keeps_what_it_did_not() {
        let mut queue = Queue::default();
        queue.post(&pending("short", Priority::Normal, Some(1_000)), 5_000);
        queue.post(&pending("forever", Priority::Normal, None), 5_000);
        queue.post(&pending("critical", Priority::Critical, Some(0)), 5_000);
        assert_eq!(queue.expire(5_500), 0, "nothing has expired yet");
        assert_eq!(queue.expire(6_001), 1);
        let titles: Vec<&str> = queue.list().iter().map(|n| n.title.as_str()).collect();
        assert!(titles.contains(&"forever") && titles.contains(&"critical"));
        assert!(!titles.contains(&"short"));
    }

    #[test]
    fn dismiss_removes_exactly_one_and_says_whether_it_did() {
        let mut queue = Queue::default();
        let id = queue.post(&pending("a", Priority::Normal, None), 1_000);
        queue.post(&pending("b", Priority::Normal, None), 2_000);
        assert!(queue.dismiss(id));
        assert_eq!(queue.count(), 1);
        assert!(!queue.dismiss(id), "dismissing twice is not an error");
        assert!(!queue.dismiss(9_999));
    }

    #[test]
    fn the_queue_round_trips_through_its_file_with_ids_intact() {
        // The promise this daemon exists for: what was queued is still queued in
        // a new process, with the same ids.
        let path = fixture("roundtrip");
        let store = Store::new(&path);
        let mut queue = Queue::default();
        let first = queue.post(&pending("one · หนึ่ง", Priority::High, Some(9_000)), 1_000);
        let second = queue.post(&pending("two", Priority::Critical, None), 2_000);
        store.save(&queue).expect("the queue is written");

        let (loaded, skipped) = store.load().expect("the queue is read");
        assert_eq!(skipped, 0);
        assert_eq!(loaded, queue);
        assert_eq!(loaded.list()[0].id, second);
        assert_eq!(loaded.list()[1].id, first);
        assert_eq!(
            loaded.next_id(),
            second,
            "a restart must not reissue an id the desktop has seen"
        );
        cleanup(&path);
    }

    #[test]
    fn an_absent_file_is_an_empty_queue_not_an_error() {
        let path = fixture("absent");
        let store = Store::new(&path);
        let (queue, skipped) = store.load().expect("absent is fine");
        assert_eq!(queue.count(), 0);
        assert_eq!(skipped, 0);
        cleanup(&path);
    }

    #[test]
    fn a_truncated_record_is_skipped_rather_than_shown_as_half_a_message() {
        // A record that never reached `end` may have been half written. Showing
        // half of it as though it were whole is worse than dropping it.
        let text = "\
version=1
next_id=2
note
id=1
app=com.litric.files
priority=normal
title=Copy finished
body=three files
created_ms=1000
expires_ms=0
end
note
id=2
app=com.litric.files
priority=normal
title=Half writ
";
        let (queue, skipped) = parse(text);
        assert_eq!(queue.count(), 1);
        assert_eq!(queue.list()[0].id, 1);
        assert_eq!(skipped, 1);
    }

    #[test]
    fn a_corrupt_field_drops_the_record_and_not_the_queue() {
        let text = "\
version=1
note
id=1
app=a
priority=urgent
title=t
created_ms=1
expires_ms=0
end
note
id=2
app=b
priority=low
title=survivor
created_ms=1
expires_ms=0
end
";
        let (queue, skipped) = parse(text);
        assert_eq!(queue.count(), 1);
        assert_eq!(queue.list()[0].title, "survivor");
        assert_eq!(skipped, 1);
    }

    #[test]
    fn a_next_id_that_lags_behind_the_notes_never_reissues_an_id() {
        // A file edited by hand, or written by a build whose counter was lost.
        let text = "\
version=1
note
id=40
app=a
priority=normal
title=t
created_ms=1
expires_ms=0
end
";
        let (queue, _) = parse(text);
        assert_eq!(queue.next_id(), 40);
        let mut queue = queue;
        assert_eq!(queue.post(&pending("new", Priority::Normal, None), 2), 41);
    }

    #[test]
    fn garbage_produces_an_empty_queue_rather_than_a_failure() {
        let path = fixture("garbage");
        let store = Store::new(&path);
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, [0x00u8, 0xFF, 0x2E, 0x0A, 0x3D]).expect("written");
        let (queue, skipped) = store.load().expect("reading a garbage file works");
        assert_eq!(queue.count(), 0);
        assert!(skipped > 0, "the daemon should log that something was lost");
        cleanup(&path);
    }

    #[test]
    fn the_file_is_private_and_leaves_no_temporary_behind() {
        let path = fixture("private");
        let store = Store::new(&path);
        store.save(&Queue::default()).expect("written");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let file = fs::metadata(&path).expect("exists");
            assert_eq!(file.permissions().mode() & 0o777, 0o600);
            let dir = fs::metadata(path.parent().expect("parent")).expect("exists");
            assert_eq!(dir.permissions().mode() & 0o777, 0o700);
        }
        let names: Vec<String> = fs::read_dir(path.parent().expect("parent"))
            .expect("readable")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["queue.conf".to_owned()], "{names:?}");
        cleanup(&path);
    }

    #[test]
    fn clearing_reports_how_many_went() {
        let mut queue = Queue::default();
        queue.post(&pending("a", Priority::Normal, None), 1);
        queue.post(&pending("b", Priority::Normal, None), 2);
        assert_eq!(queue.clear(), 2);
        assert_eq!(queue.clear(), 0);
    }
}
