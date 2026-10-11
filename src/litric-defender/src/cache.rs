//! Verdict cache: a bounded LRU keyed by (path, size, mtime).
//!
//! Purpose (ADR-0022 §5): an on-access scanner sits on the critical path of
//! every file open on the machine. Re-scanning an unchanged file on every
//! open would tax the whole OS for zero security gain. The cache maps the
//! identity of a file *version* to the last verdict, and drops the oldest
//! entries under pressure so memory is bounded by construction.
//!
//! Key design (ADR-0022 §5): the key includes size and mtime, so an edited
//! file is automatically a new key — no explicit invalidation protocol to
//! get wrong. The path is a string; a renamed file is a miss and is simply
//! re-scanned.
//!
//! Fail-open discipline (ADR-0022 §6): every cache miss resolves to
//! [`CacheMiss`], and the caller's policy decides what an unresolved file
//! does. The cache itself never blocks, never panics, and never returns a
//! verdict for a key it does not hold. A *negative* verdict (deny) is still
//! cached — quarantining a file must not make it slow as well as dead — but
//! the engine treats a deny hit as advisory: the denylist itself is the
//! source of truth and is re-consulted on feed reload via generation
//! bumping (see [`VerdictCache::bump_generation`]).

use std::collections::HashMap;

/// Maximum entries. Memory bound: each entry is one `String` path, two
/// integers and one verdict — tens to hundreds of bytes — so 65,536 entries
/// is a few tens of MiB at the very worst, and typically far less.
pub const DEFAULT_CAPACITY: usize = 65_536;

/// Why a lookup did not yield a cached verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMiss {
    /// No entry for this file version.
    Absent,
    /// An entry exists for this file version, but it was stored under an
    /// older feed generation. The engine must re-derive the verdict from
    /// current data.
    StaleGeneration,
}

/// The cached outcome for a file version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// No detection fired.
    Clean,
    /// A detection fired (denylist or pattern tier).
    Detected,
}

/// Identity of one file version.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    path: String,
    size: u64,
    mtime: u64,
}

/// Feed generation guard: bumped whenever a denylist or pattern set is
/// reloaded. All entries older than the new generation become
/// [`CacheMiss::StaleGeneration`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(u64);

/// The bounded LRU cache.
#[derive(Debug, Default)]
pub struct VerdictCache {
    map: HashMap<Key, (Verdict, Generation)>,
    /// Recency order, oldest first. `Key` is cloned into here; the clone
    /// cost is bounded by `capacity`.
    order: Vec<Key>,
    capacity: usize,
    gen: Generation,
}

impl VerdictCache {
    /// A cache with the default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// A cache with an explicit entry bound.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: Vec::new(),
            capacity: capacity.max(1),
            gen: Generation::default(),
        }
    }

    /// Current feed generation.
    #[must_use]
    pub fn generation(&self) -> Generation {
        self.gen
    }

    /// Advance the feed generation, invalidating every cached verdict in one
    /// step (used after a definitions reload).
    pub fn bump_generation(&mut self) {
        self.gen = Generation(self.gen.0.wrapping_add(1));
        // Entries are pruned lazily on insert/lookup; the map itself keeps
        // stale entries only until eviction pressure or their next lookup.
    }

    /// Look up a file version. Returns `Ok(Verdict)` on a hit for the
    /// current generation, or the reason for the miss.
    pub fn lookup(&mut self, path: &str, size: u64, mtime: u64) -> Result<Verdict, CacheMiss> {
        let key = Key {
            path: path.to_string(),
            size,
            mtime,
        };
        let Some(&(verdict, gen)) = self.map.get(&key) else {
            return Err(CacheMiss::Absent);
        };
        if gen != self.gen {
            return Err(CacheMiss::StaleGeneration);
        }
        self.touch(&key);
        Ok(verdict)
    }

    /// Store a verdict for a file version. Overwrites any prior verdict for
    /// the same key; evicts the least-recently-used entry when full.
    pub fn store(&mut self, path: &str, size: u64, mtime: u64, verdict: Verdict) {
        let key = Key {
            path: path.to_string(),
            size,
            mtime,
        };
        self.map.insert(key.clone(), (verdict, self.gen));
        if !self.order.contains(&key) {
            self.order.push(key.clone());
        }
        self.touch(&key);
        while self.map.len() > self.capacity {
            let evict = match self.order.first() {
                Some(k) => k.clone(),
                None => break,
            };
            self.order.remove(0);
            self.map.remove(&evict);
        }
    }

    /// Move a key to the most-recently-used end of the order list.
    fn touch(&mut self, key: &Key) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let k = self.order.remove(pos);
            self.order.push(k);
        }
    }

    /// Number of live entries (may include stale-generation entries until
    /// they are evicted or overwritten).
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True when the cache holds no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Drop every entry, keeping the generation counter. Used by the
    /// engine's policy "reload definitions -> forget all cached verdicts".
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_and_miss_roundtrip() {
        let mut c = VerdictCache::new();
        assert_eq!(c.lookup("/a", 10, 100), Err(CacheMiss::Absent));
        c.store("/a", 10, 100, Verdict::Clean);
        assert_eq!(c.lookup("/a", 10, 100), Ok(Verdict::Clean));
        // Any identity change is a different file version.
        assert_eq!(c.lookup("/a", 11, 100), Err(CacheMiss::Absent));
        assert_eq!(c.lookup("/a", 10, 101), Err(CacheMiss::Absent));
        assert_eq!(c.lookup("/b", 10, 100), Err(CacheMiss::Absent));
    }

    /// Generation bump: entries survive in memory but answer
    /// StaleGeneration, and a fresh store under the new generation wins.
    #[test]
    fn generation_bump_stales_everything() {
        let mut c = VerdictCache::new();
        c.store("/a", 1, 1, Verdict::Detected);
        assert_eq!(c.lookup("/a", 1, 1), Ok(Verdict::Detected));
        c.bump_generation();
        assert_eq!(c.lookup("/a", 1, 1), Err(CacheMiss::StaleGeneration));
        c.store("/a", 1, 1, Verdict::Clean);
        assert_eq!(c.lookup("/a", 1, 1), Ok(Verdict::Clean));
    }

    /// A deny verdict is cached like any other (quarantined files must not
    /// be slow), and can be overwritten by a later re-scan.
    #[test]
    fn deny_verdicts_cache_and_overwrite() {
        let mut c = VerdictCache::new();
        c.store("/evil", 5, 5, Verdict::Detected);
        assert_eq!(c.lookup("/evil", 5, 5), Ok(Verdict::Detected));
        c.store("/evil", 5, 5, Verdict::Clean);
        assert_eq!(c.lookup("/evil", 5, 5), Ok(Verdict::Clean));
    }

    /// Capacity is a hard bound: inserting N+1 distinct keys leaves at most
    /// N entries, and the evicted key is precisely the least-recently-used.
    #[test]
    fn capacity_bounded_lru_eviction() {
        let mut c = VerdictCache::with_capacity(3);
        c.store("/1", 0, 0, Verdict::Clean);
        c.store("/2", 0, 0, Verdict::Clean);
        c.store("/3", 0, 0, Verdict::Clean);
        // Touch /1 so /2 becomes LRU.
        assert_eq!(c.lookup("/1", 0, 0), Ok(Verdict::Clean));
        c.store("/4", 0, 0, Verdict::Clean);
        assert_eq!(c.map.len(), 3);
        assert_eq!(c.lookup("/2", 0, 0), Err(CacheMiss::Absent), "LRU evicted");
        assert_eq!(c.lookup("/1", 0, 0), Ok(Verdict::Clean), "touched survivor");
        assert_eq!(c.lookup("/3", 0, 0), Ok(Verdict::Clean));
        assert_eq!(c.lookup("/4", 0, 0), Ok(Verdict::Clean));
    }

    /// Sustained churn above capacity keeps memory bounded and the cache
    /// coherent — the on-access scanner will do this all day.
    #[test]
    fn churn_stays_bounded() {
        let mut c = VerdictCache::with_capacity(64);
        for i in 0..1_000u32 {
            let p = format!("/f{i}");
            c.store(&p, i as u64, i as u64, Verdict::Clean);
            assert!(c.len() <= 64);
        }
        assert_eq!(c.len(), 64);
    }

    #[test]
    fn clear_and_default_behaviour() {
        let mut c = VerdictCache::new();
        assert!(c.is_empty());
        c.store("/a", 1, 1, Verdict::Detected);
        c.clear();
        assert!(c.is_empty());
        assert_eq!(c.lookup("/a", 1, 1), Err(CacheMiss::Absent));
    }
}
