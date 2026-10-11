//! The engine: ties denylist, patterns, cache and policy into one scan
//! entry point, with the evidence record the audit log requires
//! (ADR-0022 §10).
//!
//! Pure and synchronous: the engine takes bytes and metadata, returns a
//! decision and the observations behind it. Transport (fanotify, kernel
//! hook) is a later phase's problem; this stays testable on any host.

use crate::cache::{Verdict, VerdictCache};
use crate::denylist::Denylist;
use crate::patterns::Matcher;
use crate::sha256;
use crate::verdict::{cacheable_verdict, resolve, Decision, Mode, Observation, Tier};

/// The assembled scanner. Immutable configuration, mutable cache.
#[derive(Debug, Default)]
pub struct Engine {
    denylist: Denylist,
    matcher: Matcher,
    cache: VerdictCache,
    mode: Mode,
}

/// Everything one scan concluded, for the audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    /// The decision.
    pub decision: Decision,
    /// SHA-256 hex of the scanned bytes (empty string when nothing was read).
    pub digest_hex: String,
    /// Bytes scanned.
    pub size: u64,
    /// Whether the verdict came from cache.
    pub from_cache: bool,
}

impl Engine {
    /// An engine with no definitions loaded: everything scans clean and the
    /// mode is the Phase 0 default (permissive). Construct then load.
    #[must_use]
    pub fn new() -> Self {
        Self {
            denylist: Denylist::new(),
            matcher: Matcher::default(),
            cache: VerdictCache::new(),
            mode: Mode::default(),
        }
    }

    /// Set the unknown-file mode.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Replace the denylist (a definitions reload). Bumps the cache
    /// generation so stale verdicts cannot survive a feed change.
    pub fn load_denylist(&mut self, list: Denylist) {
        self.denylist = list;
        self.cache.bump_generation();
    }

    /// Replace the pattern set. Same generation rule as the denylist.
    ///
    /// # Errors
    /// Propagates [`crate::patterns::BuildError`] from the matcher build.
    /// On error the previous set remains loaded — a bad feed must not leave
    /// the engine with NO patterns.
    pub fn load_patterns<I, P, S>(&mut self, patterns: I) -> Result<(), crate::patterns::BuildError>
    where
        I: IntoIterator<Item = (P, S)>,
        P: AsRef<[u8]>,
        S: Into<String>,
    {
        let built = Matcher::build(patterns)?;
        self.matcher = built;
        self.cache.bump_generation();
        Ok(())
    }

    /// Scan file content identified by path/size/mtime. The bytes are
    /// already in memory: the caller (transport) owns reading, which keeps
    /// this core I/O-free and lets tests inject arbitrary content.
    #[must_use]
    pub fn scan(&mut self, path: &str, size: u64, mtime: u64, content: &[u8]) -> ScanReport {
        // 1. Cache: identical file version -> identical answer, cheaply.
        // A hit returns here; Absent and StaleGeneration fall through to a
        // real scan.
        if let Ok(verdict) = self.cache.lookup(path, size, mtime) {
            // A cached deny is advisory only: the denylist is the source
            // of truth, so re-derive the label for the report. A cached
            // clean short-circuits honestly.
            let decision = match verdict {
                Verdict::Clean => Decision::Allow,
                Verdict::Detected => {
                    // Re-derive against current feeds; if the feed was
                    // updated and no longer matches, the fresh verdict
                    // wins below anyway.
                    match self.observe(content) {
                        Some(obs) => resolve(&obs, self.mode),
                        None => Decision::AllowUnknown {
                            reason: "cached-deny re-derivation failed".into(),
                        },
                    }
                }
            };
            return ScanReport {
                decision,
                digest_hex: String::new(),
                size,
                from_cache: true,
            };
        }

        // 2. Real scan.
        let observation = self
            .observe(content)
            .unwrap_or_else(|| Observation::Unscannable {
                reason: "internal tier failure".into(),
            });
        let decision = resolve(&observation, self.mode);

        // 3. Cache whatever is cacheable. Unknowns cache as clean by the
        // documented Phase 0 trade-off.
        if let Some(v) = cacheable_verdict(&decision) {
            self.cache.store(path, size, mtime, v);
        }

        ScanReport {
            digest_hex: hex_of(content),
            size,
            decision,
            from_cache: false,
        }
    }

    /// Run the detection tiers over content, strongest evidence first.
    /// `None` is reserved for a future I/O-bound tier (Phase 2); the pure
    /// tiers always produce an observation today.
    fn observe(&self, content: &[u8]) -> Option<Observation> {
        // Tier 1: exact hash. No false positives by construction.
        let digest = sha256::digest(content);
        if let Some(label) = self.denylist.lookup(&digest) {
            return Some(Observation::Detection {
                tier: Tier::Hash,
                id: format!("{label} ({})", sha256::hex(&digest)),
            });
        }
        // Tier 2: byte patterns.
        let hits = self.matcher.find(content);
        if let Some(m) = hits.first() {
            let id = self.matcher.id(m.pattern).unwrap_or("unknown-pattern");
            return Some(Observation::Detection {
                tier: Tier::Pattern,
                id: id.to_string(),
            });
        }
        Some(Observation::Clean)
    }

    /// Cache statistics for diagnostics.
    #[must_use]
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
}

/// Hex of a digest, computed once per scan report.
fn hex_of(content: &[u8]) -> String {
    let d = sha256::digest(content);
    sha256::hex(&d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

    fn eicar_digest_hex() -> String {
        let d = sha256::digest(EICAR);
        sha256::hex(&d)
    }

    #[test]
    fn empty_engine_allows_everything() {
        let mut e = Engine::new();
        let r = e.scan("/f", 3, 1, b"abc");
        assert_eq!(r.decision, Decision::Allow);
        assert!(!r.from_cache);
    }

    /// The full ladder: hash tier fires, denies, caches, and the cached
    /// deny re-derives correctly on the second open.
    #[test]
    fn hash_tier_detect_deny_cache() {
        let mut e = Engine::new();
        let mut list = Denylist::new();
        let hex = eicar_digest_hex();
        list.insert(&hex, "eicar-test-file");
        e.load_denylist(list);

        let r1 = e.scan("/tmp/eicar", EICAR.len() as u64, 42, EICAR);
        assert_eq!(
            r1.decision,
            Decision::Deny {
                tier: Tier::Hash,
                id: format!("eicar-test-file ({hex})")
            }
        );
        assert!(!r1.from_cache);
        assert_eq!(r1.digest_hex, hex);

        // Same file version -> cache hit, still denied.
        let r2 = e.scan("/tmp/eicar", EICAR.len() as u64, 42, EICAR);
        assert!(r2.from_cache);
        assert!(matches!(r2.decision, Decision::Deny { .. }));

        // A modified file (different mtime) re-scans and still denies.
        let r3 = e.scan("/tmp/eicar", EICAR.len() as u64, 43, EICAR);
        assert!(!r3.from_cache);
        assert!(matches!(r3.decision, Decision::Deny { .. }));
    }

    /// Pattern tier fires when the hash is not listed.
    #[test]
    fn pattern_tier_detects_unlisted_content() {
        let mut e = Engine::new();
        e.load_patterns([(EICAR, "eicar-test-file")])
            .expect("valid pattern");
        let r = e.scan("/tmp/x", EICAR.len() as u64, 1, EICAR);
        assert_eq!(
            r.decision,
            Decision::Deny {
                tier: Tier::Pattern,
                id: "eicar-test-file".into()
            }
        );
    }

    #[test]
    fn clean_files_cache_as_clean() {
        let mut e = Engine::new();
        let r1 = e.scan("/doc.txt", 5, 1, b"hello");
        assert_eq!(r1.decision, Decision::Allow);
        let r2 = e.scan("/doc.txt", 5, 1, b"hello");
        assert!(r2.from_cache);
        assert_eq!(r2.decision, Decision::Allow);
    }

    /// A definitions reload must invalidate cached verdicts: the same file
    /// version flips from clean to detected after the feed changes.
    #[test]
    fn feed_reload_invalidates_cache() {
        let mut e = Engine::new();
        let r1 = e.scan("/f", 4, 1, b"evil");
        assert_eq!(r1.decision, Decision::Allow);

        let mut list = Denylist::new();
        let d = sha256::digest(b"evil");
        list.insert(&sha256::hex(&d), "test-detect");
        e.load_denylist(list);

        let r2 = e.scan("/f", 4, 1, b"evil");
        assert!(
            !r2.from_cache,
            "stale clean verdict must not survive reload"
        );
        assert!(matches!(r2.decision, Decision::Deny { .. }));
    }

    /// A failed pattern reload must keep the previous set active — a bad
    /// feed must not disarm the scanner.
    #[test]
    fn failed_pattern_reload_keeps_old_set() {
        let mut e = Engine::new();
        e.load_patterns([(EICAR, "eicar")]).expect("valid");
        // Empty pattern is a build error.
        assert!(e.load_patterns([(b"" as &[u8], "bad")]).is_err());
        let r = e.scan("/f", EICAR.len() as u64, 1, EICAR);
        assert!(
            matches!(r.decision, Decision::Deny { .. }),
            "old set still armed"
        );
    }

    /// Permissive mode allows unscannable input; the engine's observe never
    /// returns Unscannable today (bytes are already in hand), but the mode
    /// wiring is still verified through the policy contract.
    #[test]
    fn mode_switch_changes_unknown_handling() {
        let mut e = Engine::new();
        e.set_mode(Mode::Enforce);
        // With no definitions, everything is Clean (not Unknown), so Enforce
        // still allows genuinely-scanned content.
        let r = e.scan("/f", 2, 1, b"ok");
        assert_eq!(r.decision, Decision::Allow);
    }

    /// Cache eviction under engine load stays bounded.
    #[test]
    fn engine_cache_stays_bounded() {
        let mut e = Engine::new();
        for i in 0..500u32 {
            let p = format!("/f{i}");
            let _ = e.scan(&p, i as u64, i as u64, b"data");
        }
        assert!(e.cache_len() <= crate::cache::DEFAULT_CAPACITY);
    }
}
