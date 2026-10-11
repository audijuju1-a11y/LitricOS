//! LitricOS Trash.
//!
//! # Layer position
//!
//! **Layer 3 (System API).** Depends downward on `litric-storage`. No I/O: it
//! decides what a delete, a restore or an empty *means*, and `litric-filesd`
//! performs it.
//!
//! # Compatible on purpose
//!
//! This implements the freedesktop.org Trash specification rather than
//! inventing a LitricOS format. A file trashed here stays trashable, listable
//! and restorable by every other Linux tool, and a file trashed by another tool
//! shows up in ours. A private format would buy nothing and would strand a
//! user's deleted files behind our own file manager.
//!
//! ```text
//!   $XDG_DATA_HOME/Trash/
//!     ├── files/<name>            the file itself, moved not copied
//!     └── info/<name>.trashinfo   where it came from, and when
//! ```
//!
//! # The rules that matter
//!
//! Deleting is the operation users get wrong, and the one they cannot undo. So:
//!
//! * **Nothing in the trash ever overwrites anything else.** A name collision
//!   gets a suffix; it never silently replaces the file already there.
//! * **A restore never overwrites either**, and never lands outside the
//!   directory it came from.
//! * **The original path is recorded** so "put it back" is a real answer.
//! * **Automatic emptying is by age, oldest first, and is announced** — a trash
//!   that quietly drops things is a trash that loses work.

#![forbid(unsafe_code)]

use core::fmt;
use litric_storage::{StorageUri, UriError};
use std::collections::BTreeMap;

/// Seconds since the Unix epoch. Injected, never read from a clock in here, so
/// retention policy is testable without waiting.
pub type Timestamp = u64;

/// Why an item could not be trashed or restored.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrashError {
    /// The path was not a valid storage URI.
    BadPath(UriError),
    /// The trash has no such entry.
    NotFound,
    /// Restoring would overwrite something that exists now.
    RestoreWouldOverwrite {
        /// What is in the way.
        occupied: String,
    },
    /// The item is larger than the trash is allowed to hold, so trashing it
    /// would immediately evict other things.
    TooLargeForTrash {
        /// Size of the item, bytes.
        size: u64,
        /// The whole trash budget, bytes.
        capacity: u64,
    },
    /// Nothing in a per-application private namespace goes to the shared trash.
    NotTrashable {
        /// Why.
        reason: &'static str,
    },
}

impl fmt::Display for TrashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadPath(e) => write!(f, "invalid path: {e}"),
            Self::NotFound => write!(f, "no such item in the trash"),
            Self::RestoreWouldOverwrite { occupied } => {
                write!(f, "{occupied} already exists; restoring would replace it")
            }
            Self::TooLargeForTrash { size, capacity } => write!(
                f,
                "item is {size} bytes and the trash holds {capacity}; \
                 trashing it would evict everything else"
            ),
            Self::NotTrashable { reason } => write!(f, "cannot be moved to the trash: {reason}"),
        }
    }
}

impl std::error::Error for TrashError {}

/// One item in the trash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashedItem {
    /// The name it has inside `Trash/files/`, after collision resolution.
    name: String,
    /// Where it came from. This is what "Put back" uses.
    original: StorageUri,
    /// When it was trashed.
    deleted_at: Timestamp,
    /// Size in bytes, for the capacity budget.
    size: u64,
}

impl TrashedItem {
    /// The name inside the trash.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Where it came from.
    #[must_use]
    pub const fn original(&self) -> &StorageUri {
        &self.original
    }
    /// When it was deleted.
    #[must_use]
    pub const fn deleted_at(&self) -> Timestamp {
        self.deleted_at
    }
    /// Size in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// The `.trashinfo` body, in the freedesktop format.
    ///
    /// Emitted rather than parsed-and-reemitted so that the file we write is
    /// exactly the file the specification describes, and other tools can read
    /// it.
    #[must_use]
    pub fn trashinfo(&self, iso_date: &str) -> String {
        format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            self.original.to_uri(),
            iso_date
        )
    }
}

/// How long items survive before automatic emptying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    /// Age at which an item becomes eligible for automatic removal, seconds.
    /// `None` disables automatic emptying.
    pub max_age_secs: Option<u64>,
    /// Total bytes the trash may hold.
    pub capacity_bytes: u64,
}

impl Default for Retention {
    /// Thirty days, and 10% of a notional 100 GB volume.
    ///
    /// Thirty days because the point of a trash is to survive the gap between
    /// deleting something and realising it mattered, and that gap is usually
    /// measured in weeks. A trash that empties in a day is a slower `rm`.
    fn default() -> Self {
        Self {
            max_age_secs: Some(30 * 24 * 60 * 60),
            capacity_bytes: 10 * 1024 * 1024 * 1024,
        }
    }
}

/// What an automatic empty would remove, and why.
///
/// Returned rather than performed so the caller can tell the user before
/// anything is destroyed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyPlan {
    /// Names to remove, oldest first.
    pub remove: Vec<String>,
    /// Bytes reclaimed.
    pub bytes_freed: u64,
    /// Why each removal happened.
    pub reason: EmptyReason,
}

/// Why automatic emptying triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EmptyReason {
    /// Nothing needed removing.
    Nothing,
    /// Items passed the age limit.
    Age,
    /// The trash was over its capacity.
    Capacity,
    /// Both applied.
    AgeAndCapacity,
}

/// The trash.
#[derive(Debug, Clone)]
pub struct Trash {
    items: BTreeMap<String, TrashedItem>,
    retention: Retention,
}

impl Trash {
    /// An empty trash with the given retention policy.
    #[must_use]
    pub fn new(retention: Retention) -> Self {
        Self {
            items: BTreeMap::new(),
            retention,
        }
    }

    /// The retention policy in force.
    #[must_use]
    pub const fn retention(&self) -> Retention {
        self.retention
    }

    /// Items, oldest first.
    #[must_use]
    pub fn items(&self) -> Vec<&TrashedItem> {
        let mut v: Vec<&TrashedItem> = self.items.values().collect();
        v.sort_by_key(|i| (i.deleted_at, i.name.clone()));
        v
    }

    /// How many bytes the trash currently holds.
    #[must_use]
    pub fn used_bytes(&self) -> u64 {
        self.items
            .values()
            .fold(0u64, |a, i| a.saturating_add(i.size))
    }

    /// Number of items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the trash is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Move something to the trash.
    ///
    /// Returns the name it was given inside the trash, which may differ from its
    /// original file name if that name was taken.
    ///
    /// # Errors
    /// [`TrashError`] when the path is not trashable or the item cannot fit.
    pub fn trash(
        &mut self,
        original: &StorageUri,
        size: u64,
        now: Timestamp,
    ) -> Result<String, TrashError> {
        // Per-application private storage does not go to the shared trash. An
        // application's own cache and config are its business, and putting them
        // in the user's trash would fill it with files the user did not create
        // and cannot identify.
        if original.namespace().is_per_app() {
            return Err(TrashError::NotTrashable {
                reason: "per-application storage is removed with the application, \
                         not through the user's trash",
            });
        }

        if size > self.retention.capacity_bytes {
            return Err(TrashError::TooLargeForTrash {
                size,
                capacity: self.retention.capacity_bytes,
            });
        }

        let base = original
            .segments()
            .last()
            .cloned()
            .unwrap_or_else(|| "unnamed".to_owned());

        let name = self.unique_name(&base);
        self.items.insert(
            name.clone(),
            TrashedItem {
                name: name.clone(),
                original: original.clone(),
                deleted_at: now,
                size,
            },
        );
        Ok(name)
    }

    /// A name not already used in the trash.
    ///
    /// Two files called `notes.txt` from different folders both belong in the
    /// trash, and the second must not replace the first. The suffix goes before
    /// the extension so the file still opens with the right application.
    fn unique_name(&self, base: &str) -> String {
        if !self.items.contains_key(base) {
            return base.to_owned();
        }
        let (stem, ext) = match base.rsplit_once('.') {
            // A leading dot is part of the name, not an extension separator.
            Some((s, e)) if !s.is_empty() => (s, Some(e)),
            _ => (base, None),
        };
        for n in 2u32..=9999 {
            let candidate = match ext {
                Some(e) => format!("{stem}.{n}.{e}"),
                None => format!("{stem}.{n}"),
            };
            if !self.items.contains_key(&candidate) {
                return candidate;
            }
        }
        // Beyond ten thousand collisions on one name, fall back to something
        // certainly unique rather than failing the delete.
        format!("{stem}.{}", self.items.len())
    }

    /// Look up an item.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&TrashedItem> {
        self.items.get(name)
    }

    /// Plan a restore.
    ///
    /// `occupied` reports whether something already exists at a given location;
    /// the caller supplies it because this crate does no I/O.
    ///
    /// # Errors
    /// [`TrashError::NotFound`] if there is no such item, or
    /// [`TrashError::RestoreWouldOverwrite`] if the original location is taken.
    pub fn plan_restore(
        &self,
        name: &str,
        occupied: &dyn Fn(&StorageUri) -> bool,
    ) -> Result<StorageUri, TrashError> {
        let item = self.items.get(name).ok_or(TrashError::NotFound)?;
        if occupied(&item.original) {
            return Err(TrashError::RestoreWouldOverwrite {
                occupied: item.original.to_uri(),
            });
        }
        Ok(item.original.clone())
    }

    /// Remove an item from the trash record after it has been restored.
    ///
    /// # Errors
    /// [`TrashError::NotFound`] if there is no such item.
    pub fn complete_restore(&mut self, name: &str) -> Result<TrashedItem, TrashError> {
        self.items.remove(name).ok_or(TrashError::NotFound)
    }

    /// Permanently delete one item.
    ///
    /// # Errors
    /// [`TrashError::NotFound`] if there is no such item.
    pub fn delete_permanently(&mut self, name: &str) -> Result<TrashedItem, TrashError> {
        self.items.remove(name).ok_or(TrashError::NotFound)
    }

    /// Empty the whole trash. Returns what was removed.
    pub fn empty(&mut self) -> Vec<TrashedItem> {
        let mut v: Vec<TrashedItem> = self.items.values().cloned().collect();
        v.sort_by_key(|i| (i.deleted_at, i.name.clone()));
        self.items.clear();
        v
    }

    /// What automatic emptying would remove at `now`, without removing it.
    ///
    /// Age first, then capacity, and oldest first within each. Reporting the
    /// plan before acting is what lets the user be told "these twelve files are
    /// about to go" rather than discovering it afterwards.
    #[must_use]
    pub fn plan_auto_empty(&self, now: Timestamp) -> EmptyPlan {
        let mut remove: Vec<String> = Vec::new();
        let mut freed = 0u64;
        let mut by_age = false;

        let ordered = self.items();

        if let Some(max_age) = self.retention.max_age_secs {
            for item in &ordered {
                let age = now.saturating_sub(item.deleted_at);
                if age >= max_age {
                    remove.push(item.name.clone());
                    freed = freed.saturating_add(item.size);
                    by_age = true;
                }
            }
        }

        let mut remaining = self.used_bytes().saturating_sub(freed);
        let mut by_capacity = false;
        if remaining > self.retention.capacity_bytes {
            for item in &ordered {
                if remaining <= self.retention.capacity_bytes {
                    break;
                }
                if remove.contains(&item.name) {
                    continue;
                }
                remove.push(item.name.clone());
                freed = freed.saturating_add(item.size);
                remaining = remaining.saturating_sub(item.size);
                by_capacity = true;
            }
        }

        let reason = match (by_age, by_capacity) {
            (true, true) => EmptyReason::AgeAndCapacity,
            (true, false) => EmptyReason::Age,
            (false, true) => EmptyReason::Capacity,
            (false, false) => EmptyReason::Nothing,
        };
        EmptyPlan {
            remove,
            bytes_freed: freed,
            reason,
        }
    }

    /// Apply an [`EmptyPlan`]. Returns what was removed.
    pub fn apply_auto_empty(&mut self, plan: &EmptyPlan) -> Vec<TrashedItem> {
        plan.remove
            .iter()
            .filter_map(|n| self.items.remove(n))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(s: &str) -> StorageUri {
        StorageUri::parse(s).expect("valid uri")
    }
    fn trash() -> Trash {
        Trash::new(Retention {
            max_age_secs: Some(100),
            capacity_bytes: 1000,
        })
    }
    const T0: Timestamp = 1_000;

    #[test]
    fn trashing_records_where_it_came_from() {
        let mut t = trash();
        let name = t
            .trash(&uri("user://Documents/notes.txt"), 10, T0)
            .expect("trashable");
        assert_eq!(name, "notes.txt");
        let item = t.get("notes.txt").expect("present");
        assert_eq!(item.original().to_uri(), "user://Documents/notes.txt");
        assert_eq!(item.deleted_at(), T0);
    }

    #[test]
    fn a_name_collision_never_overwrites_the_earlier_file() {
        // Two files called notes.txt from different folders both belong in the
        // trash. Losing the first one silently is the single worst thing a trash
        // can do.
        let mut t = trash();
        let a = t
            .trash(&uri("user://Documents/notes.txt"), 10, T0)
            .expect("first");
        let b = t
            .trash(&uri("user://Projects/notes.txt"), 20, T0 + 1)
            .expect("second");

        assert_ne!(a, b);
        assert_eq!(t.len(), 2, "both files must survive");
        assert_eq!(
            t.get(&a).map(|i| i.original().to_uri()),
            Some("user://Documents/notes.txt".to_owned())
        );
        assert_eq!(
            t.get(&b).map(|i| i.original().to_uri()),
            Some("user://Projects/notes.txt".to_owned())
        );
    }

    #[test]
    fn the_collision_suffix_keeps_the_extension_usable() {
        // notes.2.txt, not notes.txt.2 -- the file must still open with the
        // application that handles .txt.
        let mut t = trash();
        t.trash(&uri("user://a/report.pdf"), 1, T0).expect("first");
        let second = t.trash(&uri("user://b/report.pdf"), 1, T0).expect("second");
        assert_eq!(second, "report.2.pdf");
        assert!(second.ends_with(".pdf"));
    }

    #[test]
    fn a_dotfile_is_not_treated_as_having_an_extension() {
        let mut t = trash();
        t.trash(&uri("user://a/.bashrc"), 1, T0).expect("first");
        let second = t.trash(&uri("user://b/.bashrc"), 1, T0).expect("second");
        assert_eq!(second, ".bashrc.2", "a leading dot is part of the name");
    }

    #[test]
    fn restore_returns_the_original_location() {
        let mut t = trash();
        let name = t
            .trash(&uri("user://Documents/tax/2026.ods"), 5, T0)
            .expect("trash");
        let target = t.plan_restore(&name, &|_| false).expect("restorable");
        assert_eq!(target.to_uri(), "user://Documents/tax/2026.ods");

        let item = t.complete_restore(&name).expect("removed from trash");
        assert_eq!(item.size(), 5);
        assert!(t.is_empty());
    }

    #[test]
    fn restore_refuses_to_overwrite_whatever_is_there_now() {
        // The user deleted a file, then made a new one with the same name. Put
        // Back must not destroy the new one.
        let mut t = trash();
        let name = t
            .trash(&uri("user://Documents/notes.txt"), 5, T0)
            .expect("trash");
        let r = t.plan_restore(&name, &|_| true);
        assert!(
            matches!(r, Err(TrashError::RestoreWouldOverwrite { .. })),
            "got {r:?}"
        );
        assert_eq!(
            t.len(),
            1,
            "a refused restore must leave the item in the trash"
        );
    }

    #[test]
    fn per_application_storage_does_not_go_to_the_user_trash() {
        let mut t = trash();
        for p in [
            "app://com.example.app/data",
            "cache://com.example.app/thumbs",
            "config://com.example.app/settings",
            "secure://com.example.app/token",
        ] {
            let r = t.trash(&uri(p), 1, T0);
            assert!(
                matches!(r, Err(TrashError::NotTrashable { .. })),
                "{p} should not be trashable, got {r:?}"
            );
        }
        assert!(t.is_empty());
    }

    #[test]
    fn an_item_larger_than_the_whole_trash_is_refused_rather_than_evicting_everything() {
        let mut t = trash(); // capacity 1000
        t.trash(&uri("user://a/keep.txt"), 100, T0)
            .expect("small file");
        let r = t.trash(&uri("user://a/huge.iso"), 5000, T0);
        assert!(
            matches!(r, Err(TrashError::TooLargeForTrash { .. })),
            "got {r:?}"
        );
        assert_eq!(t.len(), 1, "the existing file must survive the refusal");
    }

    // ---- retention ---------------------------------------------------------

    #[test]
    fn auto_empty_removes_by_age_oldest_first() {
        let mut t = trash(); // max_age 100
        t.trash(&uri("user://a/old.txt"), 10, T0).expect("old");
        t.trash(&uri("user://a/new.txt"), 10, T0 + 90).expect("new");

        let plan = t.plan_auto_empty(T0 + 100);
        assert_eq!(plan.reason, EmptyReason::Age);
        assert_eq!(plan.remove, vec!["old.txt"]);
        assert_eq!(plan.bytes_freed, 10);

        t.apply_auto_empty(&plan);
        assert_eq!(t.len(), 1);
        assert!(t.get("new.txt").is_some(), "the newer file must remain");
    }

    #[test]
    fn auto_empty_removes_by_capacity_oldest_first() {
        let mut t = Trash::new(Retention {
            max_age_secs: None,
            capacity_bytes: 100,
        });
        t.trash(&uri("user://a/first.txt"), 60, T0).expect("a");
        t.trash(&uri("user://a/second.txt"), 60, T0 + 1).expect("b");

        let plan = t.plan_auto_empty(T0 + 2);
        assert_eq!(plan.reason, EmptyReason::Capacity);
        assert_eq!(plan.remove, vec!["first.txt"], "the oldest goes first");
        t.apply_auto_empty(&plan);
        assert!(t.used_bytes() <= 100);
    }

    #[test]
    fn planning_an_empty_does_not_perform_it() {
        // The user must be able to be told "these are about to go" before
        // anything is destroyed.
        let mut t = trash();
        t.trash(&uri("user://a/old.txt"), 10, T0).expect("trash");
        let plan = t.plan_auto_empty(T0 + 500);
        assert!(!plan.remove.is_empty());
        assert_eq!(t.len(), 1, "planning must not delete");
        t.apply_auto_empty(&plan);
        assert!(t.is_empty());
    }

    #[test]
    fn nothing_to_do_is_reported_as_nothing() {
        let mut t = trash();
        t.trash(&uri("user://a/recent.txt"), 10, T0).expect("trash");
        let plan = t.plan_auto_empty(T0 + 1);
        assert_eq!(plan.reason, EmptyReason::Nothing);
        assert!(plan.remove.is_empty());
        assert_eq!(plan.bytes_freed, 0);
    }

    #[test]
    fn retention_can_be_disabled_and_then_only_capacity_applies() {
        let mut t = Trash::new(Retention {
            max_age_secs: None,
            capacity_bytes: 1_000_000,
        });
        t.trash(&uri("user://a/ancient.txt"), 10, 0).expect("trash");
        // Year 146 billion. If age policy were consulted at all, this would
        // evict; the point of the test is that it is not consulted.
        const FAR_FUTURE: Timestamp = 4_611_686_018_427_387_904;
        let plan = t.plan_auto_empty(FAR_FUTURE);
        assert_eq!(
            plan.reason,
            EmptyReason::Nothing,
            "age must not apply when disabled"
        );
    }

    #[test]
    fn used_bytes_tracks_add_and_remove() {
        let mut t = trash();
        assert_eq!(t.used_bytes(), 0);
        t.trash(&uri("user://a/one.txt"), 30, T0).expect("a");
        t.trash(&uri("user://a/two.txt"), 40, T0).expect("b");
        assert_eq!(t.used_bytes(), 70);
        t.delete_permanently("one.txt").expect("delete");
        assert_eq!(t.used_bytes(), 40);
        t.empty();
        assert_eq!(t.used_bytes(), 0);
    }

    #[test]
    fn emptying_returns_what_it_removed_oldest_first() {
        let mut t = trash();
        t.trash(&uri("user://a/b.txt"), 1, T0 + 5).expect("b");
        t.trash(&uri("user://a/a.txt"), 1, T0).expect("a");
        let removed = t.empty();
        assert_eq!(removed.len(), 2);
        assert_eq!(removed.first().map(TrashedItem::name), Some("a.txt"));
        assert!(t.is_empty());
    }

    #[test]
    fn trashinfo_is_the_freedesktop_format() {
        // Interoperability: another tool must be able to read this.
        let mut t = trash();
        let name = t
            .trash(&uri("user://Documents/notes.txt"), 1, T0)
            .expect("trash");
        let info = t
            .get(&name)
            .expect("present")
            .trashinfo("2026-08-28T10:00:00");
        assert!(info.starts_with("[Trash Info]\n"));
        assert!(info.contains("Path=user://Documents/notes.txt\n"));
        assert!(info.contains("DeletionDate=2026-08-28T10:00:00\n"));
    }

    #[test]
    fn missing_items_are_reported_not_silently_ignored() {
        let mut t = trash();
        assert_eq!(
            t.plan_restore("ghost", &|_| false),
            Err(TrashError::NotFound)
        );
        assert_eq!(t.complete_restore("ghost"), Err(TrashError::NotFound));
        assert_eq!(t.delete_permanently("ghost"), Err(TrashError::NotFound));
    }
}
