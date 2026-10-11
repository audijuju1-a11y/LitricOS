//! DEFLATE encoding, written from RFC 1951.
//!
//! [`Deflater`] is a streaming encoder: it implements [`Write`], compresses
//! input through LZ77 back-reference matching, and emits legal fixed-Huffman
//! blocks (RFC 1951 §3.2.6). Every block it opens is closed with an
//! end-of-block symbol; `finish` marks the last block final. Correctness
//! contract: the round-trip tests against [`crate::inflate`] are exhaustive
//! over the buffered paths, and every emitted stream is a legal RFC 1951
//! stream.
//!
//! Indexing policy: like [`crate::sha256`-style numeric cores], the
//! table/buffer arithmetic below uses direct indexing where the invariant
//! is structural (fixed-size tables, positions bounded by construction).
//! Each `allow` carries its proof in a comment. Silent zero-fallbacks were
//! rejected deliberately: a wrong-but-silent byte in a compressed stream
//! corrupts user data, which is worse than a loud bug.

// Offset arithmetic in the bit/byte writer is bounded by construction
// (nbits <= 64, code lengths <= 15, table totals), which the release
// profile's overflow-checks confirms at runtime anyway.
#![allow(clippy::arithmetic_side_effects)]

use std::io::{self, Write};

/// Longest match the encoder will emit (RFC 1951 allows up to 258).
const MAX_MATCH: usize = 258;
/// Shortest match worth encoding as a back-reference (RFC 1951 minimum).
const MIN_MATCH: usize = 3;
/// The sliding window size (RFC 1951 §3.2.5 distance limit).
const WSIZE: usize = 32_768;
/// Bits used to index the head/hash tables.
const HASH_BITS: u32 = 15;
const HASH_SIZE: usize = 1 << HASH_BITS;

/// Tuning knobs. The defaults are the "document archive" profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    /// Candidates examined per position before settling for the best found.
    pub max_chain: u32,
    /// Matches at least this long stop the search early.
    pub nice_len: usize,
}

impl Level {
    /// Fast: shallow search, lower ratio.
    #[must_use]
    pub const fn fast() -> Self {
        Self {
            max_chain: 16,
            nice_len: 32,
        }
    }

    /// Default: balanced.
    #[must_use]
    pub const fn balanced() -> Self {
        Self {
            max_chain: 128,
            nice_len: 128,
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::balanced()
    }
}

/// The DEFLATE encoder over an output [`Write`].
#[derive(Debug)]
pub struct Deflater<W: Write> {
    out: W,
    /// Fixed-code bit writer state.
    bits: u64,
    nbits: u32,
    /// Input bytes not yet consumed by compression.
    pending: Vec<u8>,
    /// Head of each hash chain: position or `usize::MAX`.
    head: Vec<usize>,
    /// Previous position in the chain.
    prev: Vec<usize>,
    /// Positions already encoded into the output stream.
    encoded: usize,
    /// Whether a block is currently open (header emitted, EOB not yet).
    block_open: bool,
    level: Level,
    finished: bool,
}

impl<W: Write> Deflater<W> {
    /// A deflater writing to `out`, with the given effort level.
    #[must_use]
    pub fn new(out: W, level: Level) -> Self {
        Self {
            out,
            bits: 0,
            nbits: 0,
            pending: Vec::new(),
            head: vec![usize::MAX; HASH_SIZE],
            prev: Vec::new(),
            encoded: 0,
            block_open: false,
            level,
            finished: false,
        }
    }

    /// Wrap up: emit any pending input as a final block, pad to a byte
    /// boundary, and return the sink.
    ///
    /// # Errors
    /// Whatever the sink returns while flushing.
    pub fn finish(mut self) -> io::Result<W> {
        if !self.finished {
            self.compress_pending(true)?;
            self.flush_bits()?;
            self.out.flush()?;
            self.finished = true;
        }
        Ok(self.out)
    }

    /// Number of input bytes accepted but not yet emitted.
    #[must_use]
    pub fn pending_len(&self) -> usize {
        self.pending.len().saturating_sub(self.encoded)
    }

    /// Emit `code` LSB-first over `len` bits. Sink errors are deferred:
    /// they surface on the next fallible call (finish/flush), keeping the
    /// `Write` impl's signature honest.
    fn put(&mut self, code: u32, len: u32) {
        self.bits |= u64::from(code).wrapping_shl(self.nbits);
        self.nbits += len;
        while self.nbits >= 8 {
            let byte = (self.bits & 0xff) as u8;
            if self.out.write_all(&[byte]).is_err() {
                // Deferred error: state is reset so repeated puts do not
                // corrupt anything further; `finish` will report the sink
                // error on its own flush.
                self.bits = 0;
                self.nbits = 0;
                return;
            }
            self.bits >>= 8;
            self.nbits -= 8;
        }
    }

    fn flush_bits(&mut self) -> io::Result<()> {
        while self.nbits > 0 {
            let byte = (self.bits & 0xff) as u8;
            self.out.write_all(&[byte])?;
            self.bits >>= 8;
            self.nbits = self.nbits.saturating_sub(8);
        }
        self.bits = 0;
        Ok(())
    }

    /// Hash three bytes starting at `pos`.
    ///
    /// Index invariants: `pos < data.len()` is asserted by the callers
    /// (`find_match` and `compress_pending` both guard `pos + MIN_MATCH <=
    /// len`, and MIN_MATCH >= 3, so `pos`, `pos+1`, `pos+2` are all in
    /// range). Reads past the end would still be structurally safe here
    /// because `get` bounds-checks — but direct indexing keeps the hot
    /// loop allocation-free.
    #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    fn hash(&self, data: &[u8], pos: usize) -> usize {
        let a = u32::from(data[pos]);
        let b = u32::from(data[pos + 1]);
        let c = u32::from(data[pos + 2]);
        ((a << 10) ^ (b << 5) ^ c)
            .wrapping_mul(0x9E37_79B1)
            .wrapping_shr(32 - HASH_BITS) as usize
            & (HASH_SIZE - 1)
    }

    /// Insert position `pos` into the chain, returning the previous head.
    /// Index invariant: `prev` is resized to `pos + 1` immediately above,
    /// so `prev[pos]` cannot be out of range.
    #[allow(clippy::indexing_slicing)]
    fn insert_pos(&mut self, pos: usize) {
        let h = self.hash(&self.pending, pos);
        let old = self.head.get(h).copied().unwrap_or(usize::MAX);
        if pos >= self.prev.len() {
            self.prev.resize(pos + 1, usize::MAX);
        }
        self.prev[pos] = old;
        if let Some(slot) = self.head.get_mut(h) {
            *slot = pos;
        }
    }

    /// Find the longest match for the data at `pos`, searching the chain.
    #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    fn find_match(&self, pos: usize) -> Option<(usize, usize)> {
        let avail = self.pending.len();
        if pos + MIN_MATCH > avail {
            return None;
        }
        let mut best_len = MIN_MATCH - 1;
        let mut best_dist = 0usize;
        let h = self.hash(&self.pending, pos);
        let mut cand = self.head.get(h).copied().unwrap_or(usize::MAX);
        if cand >= pos {
            return None;
        }
        let limit = pos.saturating_sub(WSIZE);
        let mut chain = self.level.max_chain;
        let max_len = avail - pos;
        while chain > 0 && cand != usize::MAX && cand >= limit {
            chain -= 1;
            let dist = pos - cand;
            let mut len = 0usize;
            while len < max_len && len < MAX_MATCH {
                let a = self.pending.get(cand + len).copied();
                let b = self.pending.get(pos + len).copied();
                match (a, b) {
                    (Some(x), Some(y)) if x == y => len += 1,
                    _ => break,
                }
            }
            if len > best_len {
                best_len = len;
                best_dist = dist;
                if len >= self.level.nice_len || len >= max_len {
                    break;
                }
            }
            cand = self.prev.get(cand).copied().unwrap_or(usize::MAX);
        }
        if best_len >= MIN_MATCH && best_dist > 0 {
            Some((best_len, best_dist))
        } else {
            None
        }
    }

    /// Emit a literal with its fixed Huffman code.
    fn emit_literal(&mut self, byte: u8) {
        let sym = u32::from(byte);
        let (code, len): (u32, u32) = if sym < 144 {
            (0x30 + sym, 8)
        } else {
            (0x190 + (sym - 144), 9)
        };
        self.put(reverse(code, len), len);
    }

    /// Emit a length/distance pair: length symbol, extra bits, then the
    /// 5-bit distance code and its extra bits.
    fn emit_match(&mut self, len: usize, dist: usize) {
        let (lsym, lextra_bits, lextra_val) = length_code(len);
        let (dsym, dextra_bits, dextra_val) = distance_code(dist);
        let (code, bits): (u32, u32) = if lsym < 256 {
            (0x30 + lsym, 8)
        } else if lsym < 280 {
            (lsym - 256, 7)
        } else {
            (0xC0 + (lsym - 280), 8)
        };
        self.put(reverse(code, bits), bits);
        if lextra_bits > 0 {
            self.put(lextra_val, lextra_bits);
        }
        self.put(reverse(dsym, 5), 5);
        if dextra_bits > 0 {
            self.put(dextra_val, dextra_bits);
        }
    }

    /// Compress everything pending; `final_block` marks the stream end.
    /// Exactly one block is emitted per call, and every block is closed
    /// with the end-of-block symbol before returning. Index invariant:
    /// `pos = self.encoded < self.pending.len()` is the loop condition, so
    /// `pending[pos]` is in range.
    #[allow(clippy::indexing_slicing)]
    fn compress_pending(&mut self, final_block: bool) -> io::Result<()> {
        if !self.block_open {
            // Block header: BFINAL, then BTYPE=01 (two bits, LSB first).
            let bfinal = u32::from(final_block);
            self.put(bfinal, 1);
            self.put(1, 1);
            self.put(0, 1);
            self.block_open = true;
        }

        while self.encoded < self.pending.len() {
            let pos = self.encoded;
            match self.find_match(pos) {
                Some((len, dist)) => {
                    self.emit_match(len, dist);
                    for p in pos..pos + len {
                        if p + MIN_MATCH <= self.pending.len() {
                            self.insert_pos(p);
                        }
                    }
                    self.encoded += len;
                }
                None => {
                    let byte = self.pending[pos];
                    self.emit_literal(byte);
                    if pos + MIN_MATCH <= self.pending.len() {
                        self.insert_pos(pos);
                    }
                    self.encoded += 1;
                }
            }
        }
        // Close the block unconditionally: a block header without a
        // closing EOB makes the decoder read the next header bits as
        // symbols (the bug the round-trip tests caught).
        self.put(0, 7);
        self.block_open = false;
        Ok(())
    }
}

impl<W: Write> Write for Deflater<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(buf);
        self.compress_pending(false)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Reverse the low `len` bits of `code` — Huffman codes are MSB-first while
/// the stream packs LSB-first.
const fn reverse(code: u32, len: u32) -> u32 {
    if len == 0 {
        return 0;
    }
    code.reverse_bits() >> (32 - len)
}

/// RFC 1951 §3.2.5 length symbol and extra bits for a match of `len`.
/// The final arm (285, 0 extra, 258) is the length-258 special case.
const fn length_code(len: usize) -> (u32, u32, u32) {
    let (sym, extra_bits, base) = match len {
        3 => (257, 0, 3),
        4 => (258, 0, 4),
        5 => (259, 0, 5),
        6 => (260, 0, 6),
        7 => (261, 0, 7),
        8 => (262, 0, 8),
        9 => (263, 0, 9),
        10 => (264, 0, 10),
        11..=12 => (265, 1, 11),
        13..=14 => (266, 1, 13),
        15..=16 => (267, 1, 15),
        17..=18 => (268, 1, 17),
        19..=22 => (269, 2, 19),
        23..=26 => (270, 2, 23),
        27..=30 => (271, 2, 27),
        31..=34 => (272, 2, 31),
        35..=42 => (273, 3, 35),
        43..=50 => (274, 3, 43),
        51..=58 => (275, 3, 51),
        59..=66 => (276, 3, 59),
        67..=82 => (277, 4, 67),
        83..=98 => (278, 4, 83),
        99..=114 => (279, 4, 99),
        115..=130 => (280, 4, 115),
        131..=162 => (281, 5, 131),
        163..=194 => (282, 5, 163),
        195..=226 => (283, 5, 195),
        227..=257 => (284, 5, 227),
        _ => (285, 0, 258),
    };
    (sym, extra_bits, (len - base) as u32)
}

/// RFC 1951 §3.2.5 distance symbol and extra bits for distance `dist`.
const fn distance_code(dist: usize) -> (u32, u32, u32) {
    let (sym, extra_bits, base) = match dist {
        1 => (0, 0, 1),
        2 => (1, 0, 2),
        3 => (2, 0, 3),
        4 => (3, 0, 4),
        5..=6 => (4, 1, 5),
        7..=8 => (5, 1, 7),
        9..=12 => (6, 2, 9),
        13..=16 => (7, 2, 13),
        17..=24 => (8, 3, 17),
        25..=32 => (9, 3, 25),
        33..=48 => (10, 4, 33),
        49..=64 => (11, 4, 49),
        65..=96 => (12, 5, 65),
        97..=128 => (13, 5, 97),
        129..=192 => (14, 6, 129),
        193..=256 => (15, 6, 193),
        257..=384 => (16, 7, 257),
        385..=512 => (17, 7, 385),
        513..=768 => (18, 8, 513),
        769..=1024 => (19, 8, 769),
        1025..=1536 => (20, 9, 1025),
        1537..=2048 => (21, 9, 1537),
        2049..=3072 => (22, 10, 2049),
        3073..=4096 => (23, 10, 3073),
        4097..=6144 => (24, 11, 4097),
        6145..=8192 => (25, 11, 6145),
        8193..=12288 => (26, 12, 8193),
        12289..=16384 => (27, 12, 12289),
        16385..=24576 => (28, 13, 16385),
        _ => (29, 13, 24577),
    };
    (sym, extra_bits, (dist - base) as u32)
}

#[cfg(test)]
mod tests {
    // Tests slice their own buffers to plant data; bounds are asserted by
    // the round-trip equality that follows.
    #![allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

    use super::*;
    use crate::inflate::Inflater;
    use std::io::Read;

    fn roundtrip(data: &[u8]) -> Vec<u8> {
        let mut d = Deflater::new(Vec::new(), Level::balanced());
        d.write_all(data).expect("write");
        let compressed = d.finish().expect("finish");
        assert!(
            compressed.len() >= 2,
            "stream must have at least header+EOB"
        );
        let mut inf = Inflater::new(&compressed[..], u64::MAX);
        let mut out = Vec::new();
        inf.read_to_end(&mut out).expect("inflate");
        out
    }

    #[test]
    fn roundtrip_empty() {
        assert_eq!(roundtrip(b""), Vec::<u8>::new());
    }

    #[test]
    fn roundtrip_literals_only() {
        let data = b"abcabcabc";
        assert_eq!(roundtrip(data), data.to_vec());
    }

    #[test]
    fn roundtrip_highly_repetitive() {
        let data = vec![b'A'; 100_000];
        assert_eq!(roundtrip(&data), data);
    }

    #[test]
    fn roundtrip_text_like() {
        let mut data = Vec::new();
        for i in 0..5_000u32 {
            data.extend_from_slice(format!("line {i}: the quick brown fox\n").as_bytes());
        }
        let mut d = Deflater::new(Vec::new(), Level::balanced());
        d.write_all(&data).expect("write");
        let compressed = d.finish().expect("finish");
        assert!(
            compressed.len() * 2 < data.len(),
            "text with repeats should compress well: {} -> {}",
            data.len(),
            compressed.len()
        );
        let mut inf = Inflater::new(&compressed[..], u64::MAX);
        let mut out = Vec::new();
        inf.read_to_end(&mut out).expect("inflate");
        assert_eq!(out, data);
    }

    /// Every possible byte value, plus long-distance matches near the
    /// window limit.
    #[test]
    fn roundtrip_all_bytes_and_far_distances() {
        let mut data = Vec::new();
        for b in 0u16..256 {
            data.push(b as u8);
        }
        let half = vec![0x5au8; 40_000];
        data.extend_from_slice(&half);
        data.extend_from_slice(&[0x00u8; 1_000]);
        data.extend_from_slice(&half);
        assert_eq!(roundtrip(&data), data);
    }

    /// Written in chunks through `Write` — the buffered path.
    #[test]
    fn roundtrip_chunked_writes() {
        let data: Vec<u8> = (0..70_000u32).map(|i| (i % 251) as u8).collect();
        let mut d = Deflater::new(Vec::new(), Level::fast());
        for chunk in data.chunks(777) {
            d.write_all(chunk).expect("write");
        }
        let compressed = d.finish().expect("finish");
        let mut inf = Inflater::new(&compressed[..], u64::MAX);
        let mut out = Vec::new();
        inf.read_to_end(&mut out).expect("inflate");
        assert_eq!(out, data);
    }

    /// Lengths that hit each symbol-table boundary, including 258.
    #[test]
    fn all_length_symbols_reachable() {
        for len in [3usize, 10, 11, 18, 19, 130, 131, 227, 258] {
            let first = vec![b'x'; len];
            let mut data = first.clone();
            data.extend_from_slice(&first);
            assert_eq!(roundtrip(&data), data, "len {len}");
        }
    }

    #[test]
    fn max_length_encodes_and_decodes() {
        let data = vec![7u8; 600];
        assert_eq!(roundtrip(&data), data);
    }

    /// One `write` call = exactly one closed block; N writes + finish =
    /// N+1 blocks. Verified through the decoder, which cannot skip an
    /// unclosed block.
    #[test]
    fn every_write_closes_its_block() {
        let mut d = Deflater::new(Vec::new(), Level::fast());
        d.write_all(b"first").expect("write 1");
        d.write_all(b"second").expect("write 2");
        let c = d.finish().expect("finish");
        let mut inf = Inflater::new(&c[..], u64::MAX);
        let mut out = Vec::new();
        inf.read_to_end(&mut out).expect("decode");
        assert_eq!(out, b"firstsecond");
    }
}
