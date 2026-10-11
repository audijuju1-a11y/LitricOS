//! SHA-256 denylist: a sorted, immutable hash table searched by binary
//! search — the fastest detection tier and the only one with no false
//! positives (ADR-0022 §4).
//!
//! The on-disk format is deliberately line-oriented text so it can be
//! diffed, audited, hand-edited and reviewed without tooling:
//!
//! ```text
//! # comment lines and blank lines are ignored
//! e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  empty-file
//! ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  test-vector
//! ```
//!
//! Malformed lines never abort the load: they are counted and skipped. A
//! partially-broken feed must degrade coverage, not take the scanner down
//! (fail-open load policy, ADR-0022 §6).

use crate::sha256::hex;
use std::collections::BTreeMap;

/// An immutable, sorted SHA-256 denylist.
#[derive(Debug, Clone, Default)]
pub struct Denylist {
    /// Lowercase hex digest -> label. `BTreeMap` keeps iteration order
    /// deterministic for tests and evidence dumps.
    entries: BTreeMap<String, String>,
    /// Lines that could not be parsed, with reasons. Kept so the loader can
    /// report exactly what a feed lost instead of swallowing it.
    rejected: Vec<String>,
}

impl Denylist {
    /// An empty list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse feed text. Returns the list plus counts of accepted and
    /// rejected lines; nothing here can panic on arbitrary input.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut list = Self::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Split "hash label..." on any whitespace run.
            let mut parts = line.split_whitespace();
            let Some(h) = parts.next() else { continue };
            if h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
                list.rejected.push(format!("bad hash: {line}"));
                continue;
            }
            let label = parts.collect::<Vec<_>>().join(" ");
            let key = h.to_ascii_lowercase();
            // First label wins: later entries in a feed are usually merge
            // artifacts. entry() avoids the double lookup of
            // contains_key + insert.
            use std::collections::btree_map::Entry;
            match list.entries.entry(key) {
                Entry::Vacant(slot) => {
                    slot.insert(label);
                }
                Entry::Occupied(_) => {
                    list.rejected.push(format!("duplicate hash: {line}"));
                }
            }
        }
        list
    }

    /// Insert a hash directly (used by tests and by the quarantine planner).
    pub fn insert(&mut self, digest_hex: &str, label: &str) -> bool {
        if digest_hex.len() != 64 || !digest_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return false;
        }
        self.entries
            .insert(digest_hex.to_ascii_lowercase(), label.to_string())
            .is_none()
    }
    /// Is this hex digest deny-listed? `None` = clean / unknown, `Some` = the
    /// recorded label.
    pub fn lookup_hex(&self, digest_hex: &str) -> Option<&str> {
        self.entries
            .get(&digest_hex.to_ascii_lowercase())
            .map(String::as_str)
    }

    /// Lookup by raw digest bytes.
    pub fn lookup(&self, digest: &[u8; 32]) -> Option<&str> {
        self.lookup_hex(&hex(digest))
    }

    /// Number of accepted entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Always true for this type (a map with zero entries is empty), kept for
    /// API symmetry with `len`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Rejected-line records from the last parse.
    #[must_use]
    pub fn rejected(&self) -> &[String] {
        &self.rejected
    }

    /// All (hash, label) pairs in deterministic order — used by the
    /// self-audit dump so two builds of the same feed produce identical
    /// evidence.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn parses_comments_blanks_and_labels() {
        let text = format!("# feed v1\n\n{EMPTY}  empty-file\n  {ABC} test-vector  \n");
        let list = Denylist::parse(&text);
        assert_eq!(list.len(), 2);
        assert!(list.rejected().is_empty());
        assert_eq!(list.lookup_hex(EMPTY), Some("empty-file"));
        assert_eq!(list.lookup_hex(ABC), Some("test-vector"));
    }

    /// Hostile feed: garbage lines must be counted, never fatal.
    #[test]
    fn rejects_malformed_lines_without_panicking() {
        let text = format!(
            "{EMPTY} ok\nnothash\nZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ short\n{ABC} dup-a\n{ABC} dup-b\nnot even a hash\n"
        );
        let list = Denylist::parse(&text);
        assert_eq!(list.len(), 2);
        assert_eq!(list.rejected().len(), 4);
        assert_eq!(list.lookup_hex(ABC), Some("dup-a"));
    }

    /// Uppercase feed entries must still match a lowercase digest.
    #[test]
    fn case_insensitive_lookup() {
        let list = Denylist::parse(&EMPTY.to_ascii_uppercase());
        assert_eq!(list.len(), 1);
        assert_eq!(list.lookup_hex(EMPTY), Some(""));
    }

    #[test]
    fn insert_rejects_short_hashes() {
        let mut list = Denylist::new();
        assert!(!list.insert("deadbeef", "x"));
        assert!(list.insert(&"D".repeat(64), "x"));
        assert_eq!(list.len(), 1);
    }

    /// Deterministic iteration order (evidence dumps must be reproducible).
    #[test]
    fn iteration_is_sorted() {
        let text = format!("{ABC} b\n{EMPTY} a\n");
        let list = Denylist::parse(&text);
        let keys: Vec<&str> = list.iter().map(|(k, _)| k).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(keys, sorted);
    }
}
