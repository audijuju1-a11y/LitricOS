//! Aho–Corasick multi-pattern matcher over byte-signature patterns
//! (ADR-0022 §4, detection tier 3).
//!
//! One pass over the scanned bytes reports every pattern that matches,
//! regardless of how many patterns are loaded. The trie is built eagerly with
//! failure links (BFS) at construction, so matching is linear in the input
//! size plus the emitted matches.
//!
//! Memory model: per-node transitions use a hash map rather than a dense
//! 256-way table. For the signature counts this crate targets (thousands of
//! patterns, not millions — the mmap'd-sorted-table design in ADR-0022 §8 is
//! what scales past that), the hash map wins on memory and loses nothing
//! measurable on speed.
//!
//! Lint note: this module performs zero slice indexing; every node access
//! goes through `get`/`get_mut` with an explicit out-of-range branch, so a
//! corrupted index can degrade a lookup, never panic.

use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Patterns longer than this are rejected at build time. Real byte
/// signatures are tens to hundreds of bytes; an unbounded length would let a
/// hostile feed allocate arbitrary trie depth.
pub const MAX_PATTERN_LEN: usize = 4096;

/// Why a [`Matcher`] could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// A pattern exceeded [`MAX_PATTERN_LEN`].
    /// `id` identifies the offending pattern; `len` is its length in bytes.
    TooLong {
        /// The pattern id supplied at build time.
        id: String,
        /// The offending length in bytes.
        len: usize,
    },
    /// An empty pattern was supplied. It would match at every offset.
    /// `id` identifies the offending pattern.
    Empty {
        /// The pattern id supplied at build time.
        id: String,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong { id, len } => {
                write!(
                    f,
                    "pattern {id:?} is {len} bytes, limit is {MAX_PATTERN_LEN}"
                )
            }
            Self::Empty { id } => write!(f, "pattern {id:?} is empty"),
        }
    }
}

impl std::error::Error for BuildError {}

/// A reported match: which pattern, and the byte range it matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Index into the pattern list given to [`Matcher::build`].
    pub pattern: usize,
    /// Start offset in the scanned bytes (inclusive).
    pub start: usize,
    /// End offset in the scanned bytes (exclusive).
    pub end: usize,
}

#[derive(Debug, Default)]
struct Node {
    /// Byte -> node index. Absent byte = no transition.
    goto: HashMap<u8, usize>,
    /// Failure link (suffix fallback). Root points to itself as 0.
    fail: usize,
    /// Pattern indices that END at this node, including those merged in from
    /// the failure chain during construction.
    output: Vec<usize>,
}

/// An immutable multi-pattern matcher.
#[derive(Debug, Default)]
pub struct Matcher {
    nodes: Vec<Node>,
    /// Pattern bodies, parallel to the ids given at build time.
    bodies: Vec<Vec<u8>>,
    /// Pattern ids, parallel to `bodies`.
    ids: Vec<String>,
}

impl Matcher {
    /// Build from `(pattern, id)` pairs. Overlapping and prefix-related
    /// patterns are fully supported.
    ///
    /// # Errors
    /// Returns [`BuildError`] if any pattern is empty or over the length
    /// limit. Nothing is partially built on error.
    pub fn build<I, P, S>(patterns: I) -> Result<Self, BuildError>
    where
        I: IntoIterator<Item = (P, S)>,
        P: AsRef<[u8]>,
        S: Into<String>,
    {
        let mut bodies = Vec::new();
        let mut ids = Vec::new();
        for (pat, id) in patterns {
            let body = pat.as_ref().to_vec();
            let id = id.into();
            if body.is_empty() {
                return Err(BuildError::Empty { id });
            }
            if body.len() > MAX_PATTERN_LEN {
                return Err(BuildError::TooLong {
                    id,
                    len: body.len(),
                });
            }
            bodies.push(body);
            ids.push(id);
        }

        let mut nodes = vec![Node::default()];
        for (idx, body) in bodies.iter().enumerate() {
            let mut cur = 0usize;
            for &byte in body {
                let next = nodes.get(cur).and_then(|n| n.goto.get(&byte)).copied();
                cur = match next {
                    Some(n) => n,
                    None => {
                        let fresh = nodes.len();
                        nodes.push(Node::default());
                        if let Some(n) = nodes.get_mut(cur) {
                            n.goto.insert(byte, fresh);
                        }
                        fresh
                    }
                };
            }
            if let Some(n) = nodes.get_mut(cur) {
                n.output.push(idx);
            }
        }

        // BFS to assign failure links and merge outputs down the suffix
        // chains, so a node's `output` is complete for matching.
        let mut queue = VecDeque::new();
        // Collect first, mutate after: one borrow at a time.
        let root_children: Vec<usize> = nodes
            .first()
            .map_or_else(Vec::new, |r| r.goto.values().copied().collect());
        for child in root_children {
            if let Some(c) = nodes.get_mut(child) {
                c.fail = 0;
            }
            queue.push_back(child);
        }
        while let Some(u) = queue.pop_front() {
            // Merge the failure node's outputs into u's. Both lists stay
            // small; dedupe is unnecessary because suffix links form a tree.
            let inherited: Vec<usize> = nodes.get(u).map_or_else(Vec::new, |n| {
                nodes
                    .get(n.fail)
                    .map_or_else(Vec::new, |f| f.output.clone())
            });
            if let Some(n) = nodes.get_mut(u) {
                n.output.extend(inherited);
            }
            let u_fail = nodes.get(u).map_or(0, |n| n.fail);
            let transitions: Vec<(u8, usize)> = nodes
                .get(u)
                .map_or_else(Vec::new, |n| n.goto.iter().map(|(k, v)| (*k, *v)).collect());
            for (byte, v) in transitions {
                let mut f = u_fail;
                let target = loop {
                    let hit = nodes.get(f).and_then(|n| n.goto.get(&byte)).copied();
                    match hit {
                        Some(w) if w != v => break w,
                        Some(_) if f == 0 => break 0,
                        Some(_) => f = nodes.get(f).map_or(0, |n| n.fail),
                        None if f == 0 => break 0,
                        None => f = nodes.get(f).map_or(0, |n| n.fail),
                    }
                };
                if let Some(n) = nodes.get_mut(v) {
                    n.fail = target;
                }
                queue.push_back(v);
            }
        }

        Ok(Self { nodes, bodies, ids })
    }

    /// Number of loaded patterns.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    /// True when no patterns are loaded (matching then always yields nothing).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }

    /// The id of a pattern by index, if in range.
    #[must_use]
    pub fn id(&self, index: usize) -> Option<&str> {
        self.ids.get(index).map(String::as_str)
    }

    /// Find every pattern occurrence in `haystack`, left to right.
    /// Overlapping matches are all reported.
    #[must_use]
    pub fn find(&self, haystack: &[u8]) -> Vec<Match> {
        let mut out = Vec::new();
        if self.nodes.is_empty() {
            return out;
        }
        let mut state = 0usize;
        for (i, &byte) in haystack.iter().enumerate() {
            // Advance: take a goto if present, else follow failure links
            // until one is found or we sit at the root.
            loop {
                let next = self
                    .nodes
                    .get(state)
                    .and_then(|n| n.goto.get(&byte))
                    .copied();
                match next {
                    Some(s) => {
                        state = s;
                        break;
                    }
                    None if state == 0 => break,
                    None => state = self.nodes.get(state).map_or(0, |n| n.fail),
                }
            }
            let outputs: Vec<usize> = self
                .nodes
                .get(state)
                .map_or_else(Vec::new, |n| n.output.clone());
            for pat_idx in outputs {
                let Some(body) = self.bodies.get(pat_idx) else {
                    continue;
                };
                // Invariant: the match ends at i+1, so i+1 >= body.len()
                // and the subtraction below cannot underflow (a trie match
                // of length L spans exactly the L bytes ending here). i is
                // an index into the scanned buffer, so i+1 cannot overflow.
                #[allow(clippy::arithmetic_side_effects)]
                let end = i + 1;
                #[allow(clippy::arithmetic_side_effects)]
                let start = end - body.len();
                out.push(Match {
                    pattern: pat_idx,
                    start,
                    end,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    // Tests plant needles by slicing the haystack; every slice is bounds-
    // checked by the assertion that follows it.
    #![allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

    use super::*;

    /// The classic textbook set, on the classic textbook input.
    #[test]
    fn classic_ushers() {
        let m = Matcher::build([
            (b"he" as &[u8], "he"),
            (b"she" as &[u8], "she"),
            (b"his" as &[u8], "his"),
            (b"hers" as &[u8], "hers"),
        ])
        .expect("valid set");
        let hits = m.find(b"ushers");
        let mut got: Vec<(&str, usize, usize)> = hits
            .iter()
            .filter_map(|mt| Some((m.id(mt.pattern)?, mt.start, mt.end)))
            .collect();
        got.sort();
        assert_eq!(
            got,
            vec![("he", 2, 4), ("hers", 2, 6), ("she", 1, 4)],
            "expected she/he/hers, each with correct offsets"
        );
    }

    /// Overlapping occurrences of one pattern must all be found — this
    /// exercises the failure-link reuse path.
    #[test]
    fn overlapping_self_matches() {
        let m = Matcher::build([(b"aa" as &[u8], "aa")]).expect("valid");
        let hits = m.find(b"aaaa");
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].start, 0);
        assert_eq!(hits[1].start, 1);
        assert_eq!(hits[2].start, 2);
    }

    /// A pattern that is a prefix of another: both must fire.
    #[test]
    fn prefix_patterns_both_fire() {
        let m = Matcher::build([(b"ab" as &[u8], "ab"), (b"abc" as &[u8], "abc")]).expect("valid");
        let hits = m.find(b"xabc");
        assert_eq!(hits.len(), 2);
        let ids: Vec<&str> = hits.iter().filter_map(|mt| m.id(mt.pattern)).collect();
        assert!(ids.contains(&"ab") && ids.contains(&"abc"));
    }

    /// The EICAR test string embedded in surrounding noise is found exactly
    /// once, at the right offsets. (EICAR is the industry-standard antivirus
    /// test file — a legitimate, universally-used test input.)
    #[test]
    fn finds_eicar_in_noise() {
        const EICAR: &[u8] =
            b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";
        let m = Matcher::build([(EICAR, "eicar")]).expect("valid");
        let mut noisy = vec![0x41u8; 100];
        noisy.extend_from_slice(EICAR);
        noisy.extend_from_slice(&[0x42u8; 100]);
        let hits = m.find(&noisy);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].start, 100);
        assert_eq!(hits[0].end, 100 + EICAR.len());
    }

    #[test]
    fn clean_input_yields_nothing() {
        let m = Matcher::build([(b"malware" as &[u8], "m")]).expect("valid");
        assert!(m.find(b"completely innocent text").is_empty());
        assert!(m.find(b"").is_empty());
    }

    /// Hostile feeds: empty and oversized patterns are rejected, not matched
    /// or truncated.
    #[test]
    fn build_rejects_degenerate_patterns() {
        assert!(matches!(
            Matcher::build([(b"" as &[u8], "empty")]),
            Err(BuildError::Empty { .. })
        ));
        let big = vec![0x90u8; MAX_PATTERN_LEN + 1];
        assert!(matches!(
            Matcher::build([(big.as_slice(), "big")]),
            Err(BuildError::TooLong { .. })
        ));
    }

    /// Scale smoke: 1,000 patterns over 100 KiB of input completes and finds
    /// every planted needle. Keeps the test suite fast while proving the
    /// failure-link machinery stays correct at non-trivial set sizes.
    #[test]
    fn thousand_patterns_smoke() {
        let pats: Vec<(Vec<u8>, String)> = (0..1_000)
            .map(|i| {
                let mut p = vec![b'A'];
                p.extend_from_slice(i.to_string().as_bytes());
                p.push(b'Z');
                (p, format!("sig{i}"))
            })
            .collect();
        let m = Matcher::build(pats.iter().map(|(p, id)| (p.as_slice(), id.clone())))
            .expect("valid set");
        let mut hay = vec![b'.'; 100_000];
        for i in [7usize, 500, 9_999, 99_000] {
            let needle = format!("A{}Z", i % 1_000);
            let off = i % (hay.len() - needle.len());
            hay[off..off + needle.len()].copy_from_slice(needle.as_bytes());
        }
        let hits = m.find(&hay);
        assert_eq!(hits.len(), 4, "every planted needle found exactly once");
    }
}
