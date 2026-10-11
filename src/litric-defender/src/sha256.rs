//! SHA-256 (FIPS 180-4), implemented here because ADR-0022 §7 forbids
//! third-party dependencies in the policy core.
//!
//! The streaming [`Sha256`] type allows hashing without owning the whole file;
//! the Defender reads chunks into a fixed 8 KiB buffer, so peak memory per
//! file is that buffer, never the file size.
//!
//! Indexing policy: this module is one of the few places the workspace's
//! `indexing_slicing = deny` is intentionally relaxed, and every relaxation
//! carries its invariant in a comment beside the `allow`. The alternative —
//! `get()` with a silent fallback — would be worse: a silent zero-block
//! digest on an impossible branch is a verdict the scanner would trust.
//! The million-'a' NIST vector and the chunk-boundary tests exercise every
//! buffered path these invariants describe.
//!
//! Fuzzability note: this module processes untrusted bytes, so the release
//! profile's `overflow-checks = true` is load-bearing here, and the
//! workspace denies panics. All chunk arithmetic is done on `u64`/`u32` with
//! wrapping semantics defined by the standard itself.

/// Block size in bytes (FIPS 180-4 §5.1).
const BLOCK: usize = 64;

/// First 32 bits of the fractional parts of the square roots of the first
/// 8 primes (FIPS 180-4 §4.2.2).
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// First 32 bits of the fractional parts of the cube roots of the first
/// 64 primes (FIPS 180-4 §4.2.3).
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// A streaming SHA-256 hasher.
///
/// Construct with [`Sha256::new`], feed with [`Sha256::update`], finish with
/// [`Sha256::finish`]. The type is deliberately bare: no resumable state,
/// so a partially-finalized hasher cannot be reused by accident.
#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    /// Bytes fed so far, for the length suffix.
    len: u64,
    /// Partial block awaiting more input.
    buf: [u8; BLOCK],
    /// Valid bytes in `buf`. Invariant: always `< BLOCK` outside `update`'s
    /// internals (a full block is compressed and the counter reset there).
    used: usize,
}

impl Sha256 {
    /// A fresh hasher.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: H0,
            len: 0,
            buf: [0; BLOCK],
            used: 0,
        }
    }

    // Invariants used by the indexing below (all locally provable):
    // - `used + take <= BLOCK`: take = min(BLOCK - used, data.len()).
    // - `data[..take]`: take <= data.len().
    // - `self.buf[..data.len()]`: the remainder path only runs when
    //   data.len() < BLOCK.
    /// Feed more bytes into the hasher.
    #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    pub fn update(&mut self, mut data: &[u8]) {
        self.len = self.len.wrapping_add(data.len() as u64);

        // Fill the partial block first, if any.
        if self.used > 0 {
            let need = BLOCK - self.used;
            let take = if data.len() < need { data.len() } else { need };
            self.buf[self.used..self.used + take].copy_from_slice(&data[..take]);
            self.used += take;
            data = &data[take..];
            if self.used == BLOCK {
                let block = self.buf;
                self.compress(&block);
                self.used = 0;
            }
        }

        // Compress full blocks directly from the input.
        while data.len() >= BLOCK {
            let (block, rest) = data.split_at(BLOCK);
            let mut b = [0u8; BLOCK];
            b.copy_from_slice(block);
            self.compress(&b);
            data = rest;
        }

        // Stash the remainder.
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.used = data.len();
        }
    }

    /// Finish and return the 32-byte digest. The hasher is consumed so a
    /// partially-finalized state cannot be reused by accident.
    #[must_use]
    pub fn finish(mut self) -> [u8; 32] {
        let bit_len = self.len.wrapping_mul(8);

        // Padding: 0x80, then zeros, then the 64-bit big-endian bit length.
        // Invariants: `used < BLOCK` (see struct field), so `buf[used]` is
        // in range; `end <= BLOCK`; the slice bounds below stay inside the
        // block for both the one-block and two-block padding shapes.
        #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
        {
            let pad_start = self.used;
            self.buf[pad_start] = 0x80;
            let mut end = pad_start + 1;
            if end > BLOCK - 8 {
                self.buf[end..BLOCK].fill(0);
                let block = self.buf;
                self.compress(&block);
                end = 0;
            }
            self.buf[end..BLOCK - 8].fill(0);
            self.buf[BLOCK - 8..BLOCK].copy_from_slice(&bit_len.to_be_bytes());
            let block = self.buf;
            self.compress(&block);

            let mut out = [0u8; 32];
            // Invariant: i in 0..8, so i*4+4 <= 32 = out.len().
            #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
            for (i, word) in self.state.iter().enumerate() {
                out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
            }
            out
        }
    }

    /// One compression round over a full 64-byte block (FIPS 180-4 §6.2.2).
    /// Every index below is a structural constant of the schedule
    /// (`i-15 >= 0`, `i-2 < 64`, `K[i]` for `i < 64`), which the fixed-size
    /// array types make unrepresentable to violate; slicing a copy of the
    /// input block is exact-size.
    #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    fn compress(&mut self, block: &[u8; BLOCK]) {
        let mut w = [0u32; 64];
        for (i, chunk) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        let deltas = [a, b, c, d, e, f, g, h];
        for (s, d) in self.state.iter_mut().zip(deltas) {
            *s = s.wrapping_add(d);
        }
    }
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

/// One-shot digest of a byte slice.
#[must_use]
pub fn digest(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finish()
}

/// Lowercase hex encoding of a digest.
#[must_use]
pub fn hex(digest: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap_or('0'));
        s.push(char::from_digit((b & 0xf) as u32, 16).unwrap_or('0'));
    }
    s
}

#[cfg(test)]
mod tests {
    // Tests deliberately re-create the buffered edge cases by slicing;
    // every slice below is constructed from the same array it reads, with
    // bounds asserted by the round-trip equality itself.
    #![allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

    use super::*;

    /// NIST / FIPS 180-4 known-answer vectors.
    #[test]
    fn nist_vectors() {
        assert_eq!(
            hex(&digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(&digest(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    /// The million-'a' vector, streamed in odd-sized chunks to exercise the
    /// buffering paths (partial block, exact block, remainder). Feeds
    /// exactly 1,000,000 bytes.
    #[test]
    fn million_a_streamed() {
        #![allow(clippy::arithmetic_side_effects, clippy::integer_division)]
        let mut h = Sha256::new();
        let chunk = [b'a'; 777];
        let full = 1_000_000 / chunk.len();
        let rem = 1_000_000 % chunk.len();
        for _ in 0..full {
            h.update(&chunk);
        }
        h.update(chunk.get(..rem).expect("rem < chunk.len()"));
        assert_eq!(
            hex(&h.finish()),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    /// Two updates must equal one update of the concatenation.
    #[test]
    fn streaming_matches_one_shot() {
        let data: Vec<u8> = (0..10_000u32).map(|i| (i % 251) as u8).collect();
        let mut h = Sha256::new();
        h.update(&data[..1234]);
        h.update(&data[1234..]);
        assert_eq!(h.finish(), digest(&data));
    }

    /// Each update pattern (fill partial, exact block, rest) must agree.
    /// Splits at every structurally interesting boundary, 0..=300, so the
    /// invariants above are exercised exhaustively, not spot-checked.
    #[test]
    fn chunk_boundaries_agree() {
        let data = vec![0x5au8; 300];
        for split in 0..=data.len() {
            let mut h = Sha256::new();
            h.update(&data[..split]);
            h.update(&data[split..]);
            assert_eq!(h.finish(), digest(&data), "split at {split}");
        }
    }
}
