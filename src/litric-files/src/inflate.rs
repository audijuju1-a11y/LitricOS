//! DEFLATE decoding, written from RFC 1951.
//!
//! [`Inflater`] is a pull decoder: it implements [`Read`] over any [`BufRead`]
//! source, keeps only the 32 KiB window the format allows a back-reference to
//! reach, and refuses to produce more than a caller-set number of bytes. Every
//! rule the RFC states is checked rather than assumed: a stored block's `NLEN`
//! must complement `LEN`, block type 3 is an error, a code-length repeat with
//! nothing to repeat is an error, an over-subscribed Huffman code is an error,
//! literal/length symbols 286-287 and distance symbols 30-31 are errors, and a
//! distance reaching before the first byte of output is an error.
//!
//! Errors travel through `io::Error` carrying a [`crate::Error`]:
//! `corrupt-archive` for a malformed or truncated stream, `limit` when the
//! output cap is reached.

use std::io::{self, BufRead, Read};
use std::sync::OnceLock;

use crate::{Error, ErrorKind};

/// The window a back-reference may reach.
const WSIZE: usize = 32_768;
/// `WSIZE - 1`.
const WMASK: usize = 32_767;
/// Bits resolved by one lookup in the fast table.
const FAST_BITS: u32 = 9;
/// `1 << FAST_BITS`.
const FAST_SIZE: usize = 512;

/// RFC 1951 3.2.5: base lengths for symbols 257..=285.
const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
/// RFC 1951 3.2.5: extra bits for symbols 257..=285.
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
/// RFC 1951 3.2.5: base distances for distance symbols 0..=29.
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
/// RFC 1951 3.2.5: extra bits for distance symbols 0..=29.
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
/// RFC 1951 3.2.7: the order code-length code lengths are sent in.
const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn corrupt(msg: &str) -> io::Error {
    Error::new(ErrorKind::CorruptArchive, msg).into_io()
}

fn truncated() -> io::Error {
    corrupt("deflate stream is truncated")
}

/// `n` low bits set, for `n <= 64`.
const fn mask(n: u32) -> u64 {
    if n >= 64 {
        u64::MAX
    } else {
        1u64.wrapping_shl(n).wrapping_sub(1)
    }
}

/// Reverse the low `len` bits of `code`.
pub(crate) const fn reverse_bits(code: u32, len: u32) -> u32 {
    if len == 0 {
        return 0;
    }
    code.reverse_bits().wrapping_shr(32u32.wrapping_sub(len))
}

/// Bits in the order RFC 1951 3.1.1 packs them: least significant first.
struct BitReader<R> {
    inner: R,
    bits: u64,
    count: u32,
    /// Bytes consumed from `inner` so far.
    pulled: u64,
}

impl<R: BufRead> BitReader<R> {
    fn refill(&mut self) -> io::Result<()> {
        while self.count <= 56 {
            let avail = loop {
                match self.inner.fill_buf() {
                    Ok(b) => break b,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            };
            if avail.is_empty() {
                return Ok(());
            }
            let room = 64u32.saturating_sub(self.count).wrapping_shr(3) as usize;
            let take = room.min(avail.len());
            let mut bits = self.bits;
            let mut count = self.count;
            for &b in avail.iter().take(take) {
                bits |= u64::from(b).wrapping_shl(count);
                count = count.wrapping_add(8);
            }
            self.bits = bits;
            self.count = count;
            self.inner.consume(take);
            self.pulled = self.pulled.saturating_add(take as u64);
        }
        Ok(())
    }

    fn drop_bits(&mut self, n: u32) {
        self.bits = if n >= 64 {
            0
        } else {
            self.bits.wrapping_shr(n)
        };
        self.count = self.count.saturating_sub(n);
    }

    /// Take `n <= 32` bits as an integer.
    fn take(&mut self, n: u32) -> io::Result<u32> {
        if self.count < n {
            self.refill()?;
            if self.count < n {
                return Err(truncated());
            }
        }
        let v = (self.bits & mask(n)) as u32;
        self.drop_bits(n);
        Ok(v)
    }

    /// Discard bits up to the next byte boundary.
    fn align(&mut self) {
        let r = self.count & 7;
        self.drop_bits(r);
    }
}

/// A canonical Huffman code (RFC 1951 3.2.2) prepared for decoding.
struct Huffman {
    /// Indexed by the next `FAST_BITS` input bits: `symbol << 4 | length`, or
    /// 0 when the code is longer than `FAST_BITS` or unassigned.
    fast: Vec<u16>,
    /// Number of codes of each length.
    counts: [u16; 16],
    /// Symbols ordered by code.
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> io::Result<Self> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            let slot = counts
                .get_mut(usize::from(l))
                .ok_or_else(|| corrupt("code length above 15"))?;
            *slot = slot.saturating_add(1);
        }
        if let Some(zero) = counts.get_mut(0) {
            *zero = 0;
        }
        // Over-subscription: more codes of some length than the lengths
        // before it leave room for.
        let mut left: i64 = 1;
        for c in counts.iter().skip(1) {
            left = left.saturating_mul(2).saturating_sub(i64::from(*c));
            if left < 0 {
                return Err(corrupt("over-subscribed Huffman code"));
            }
        }
        let mut offsets = [0usize; 16];
        let mut running = 0usize;
        for (off, c) in offsets.iter_mut().zip(counts.iter()).skip(1) {
            *off = running;
            running = running.saturating_add(usize::from(*c));
        }
        let mut symbols = vec![0u16; running];
        let mut next_code = [0u32; 16];
        let mut code = 0u32;
        for bits in 1..16usize {
            let prev = counts.get(bits.wrapping_sub(1)).copied().unwrap_or(0);
            code = code.wrapping_add(u32::from(prev)).wrapping_shl(1);
            if let Some(slot) = next_code.get_mut(bits) {
                *slot = code;
            }
        }
        let mut fast = vec![0u16; FAST_SIZE];
        for (sym, &l) in (0u16..).zip(lengths.iter()) {
            if l == 0 {
                continue;
            }
            let li = usize::from(l);
            if let Some(off) = offsets.get_mut(li) {
                if let Some(slot) = symbols.get_mut(*off) {
                    *slot = sym;
                }
                *off = off.saturating_add(1);
            }
            let Some(nc) = next_code.get_mut(li) else {
                continue;
            };
            let c = *nc;
            *nc = nc.wrapping_add(1);
            let len = u32::from(l);
            if len <= FAST_BITS {
                let entry = sym.wrapping_shl(4) | u16::from(l);
                let step = 1usize.wrapping_shl(len);
                let mut i = reverse_bits(c, len) as usize;
                while let Some(slot) = fast.get_mut(i) {
                    *slot = entry;
                    i = i.saturating_add(step);
                }
            }
        }
        Ok(Self {
            fast,
            counts,
            symbols,
        })
    }

    fn decode<R: BufRead>(&self, br: &mut BitReader<R>) -> io::Result<u16> {
        if br.count < 15 {
            br.refill()?;
        }
        let e = self
            .fast
            .get((br.bits & mask(FAST_BITS)) as usize)
            .copied()
            .unwrap_or(0);
        if e != 0 {
            let len = u32::from(e & 15);
            if len > br.count {
                return Err(truncated());
            }
            br.drop_bits(len);
            return Ok(e.wrapping_shr(4));
        }
        // Codes longer than the fast table: walk the lengths. `first` is the
        // first code of the current length and `index` the position of its
        // symbol; a code below `first + count` belongs to this length.
        let mut code = 0u32;
        let mut first = 0u32;
        let mut index = 0u32;
        for len in 1..16u32 {
            if len > br.count {
                return Err(truncated());
            }
            let bit = (br.bits.wrapping_shr(len.wrapping_sub(1)) & 1) as u32;
            code = code.wrapping_shl(1) | bit;
            let count = u32::from(self.counts.get(len as usize).copied().unwrap_or(0));
            let offset = code.saturating_sub(first);
            if offset < count {
                let sym = self
                    .symbols
                    .get(index.saturating_add(offset) as usize)
                    .copied()
                    .ok_or_else(|| corrupt("invalid Huffman code"))?;
                br.drop_bits(len);
                return Ok(sym);
            }
            index = index.saturating_add(count);
            first = first.saturating_add(count).wrapping_shl(1);
        }
        Err(corrupt("invalid Huffman code"))
    }
}

fn fixed_tables() -> &'static Option<(Huffman, Huffman)> {
    static FIXED: OnceLock<Option<(Huffman, Huffman)>> = OnceLock::new();
    FIXED.get_or_init(|| {
        // RFC 1951 3.2.6.
        let mut lit = [0u8; 288];
        for (i, l) in lit.iter_mut().enumerate() {
            *l = match i {
                0..=143 => 8,
                144..=255 => 9,
                256..=279 => 7,
                _ => 8,
            };
        }
        let dist = [5u8; 30];
        Some((Huffman::new(&lit).ok()?, Huffman::new(&dist).ok()?))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Header,
    Stored(u32),
    Fixed,
    Dynamic,
    Done,
}

/// A streaming DEFLATE decoder.
pub struct Inflater<R> {
    br: BitReader<R>,
    window: Vec<u8>,
    wpos: usize,
    total: u64,
    max_output: u64,
    state: State,
    last: bool,
    lit: Option<Huffman>,
    dist: Option<Huffman>,
    copy_len: usize,
    copy_dist: usize,
}

impl<R> core::fmt::Debug for Inflater<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Inflater")
            .field("total", &self.total)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl<R: BufRead> Inflater<R> {
    /// Decode the stream starting at the current position of `inner`,
    /// producing at most `max_output` bytes.
    pub fn new(inner: R, max_output: u64) -> Self {
        Self {
            br: BitReader {
                inner,
                bits: 0,
                count: 0,
                pulled: 0,
            },
            window: vec![0u8; WSIZE],
            wpos: 0,
            total: 0,
            max_output,
            state: State::Header,
            last: false,
            lit: None,
            dist: None,
            copy_len: 0,
            copy_dist: 0,
        }
    }

    /// Bytes produced so far.
    #[must_use]
    pub const fn total_out(&self) -> u64 {
        self.total
    }

    /// Whether the final block has been decoded completely.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.state == State::Done && self.copy_len == 0
    }

    /// Compressed bytes consumed. Exact once [`Inflater::is_done`]; before
    /// that it may include up to eight bytes read ahead.
    #[must_use]
    pub const fn compressed_len(&self) -> u64 {
        self.br
            .pulled
            .saturating_sub(self.br.count.wrapping_shr(3) as u64)
    }

    /// The whole bytes read from `inner` but not part of the stream, and
    /// `inner` itself. Meaningful once the stream is done: whatever follows
    /// the stream (a gzip trailer, say) starts with the returned bytes.
    pub fn into_parts(self) -> (Vec<u8>, R) {
        let mut br = self.br;
        br.align();
        let mut left = Vec::with_capacity(8);
        while br.count >= 8 {
            left.push(br.bits as u8);
            br.drop_bits(8);
        }
        (left, br.inner)
    }

    fn limit_check(&self, more: u64) -> io::Result<()> {
        if self.total.saturating_add(more) > self.max_output {
            return Err(Error::new(
                ErrorKind::Limit,
                format!("output would exceed {} bytes", self.max_output),
            )
            .into_io());
        }
        Ok(())
    }

    fn start_block(&mut self) -> io::Result<()> {
        if self.last {
            self.br.align();
            self.state = State::Done;
            return Ok(());
        }
        let hdr = self.br.take(3)?;
        self.last = hdr & 1 == 1;
        match hdr.wrapping_shr(1) {
            0 => {
                self.br.align();
                let len = self.br.take(16)?;
                let nlen = self.br.take(16)?;
                if len != (!nlen & 0xFFFF) {
                    return Err(corrupt("stored block length check failed"));
                }
                self.state = State::Stored(len);
            }
            1 => self.state = State::Fixed,
            2 => {
                self.read_dynamic()?;
                self.state = State::Dynamic;
            }
            _ => return Err(corrupt("reserved block type 3")),
        }
        Ok(())
    }

    fn read_dynamic(&mut self) -> io::Result<()> {
        let hlit = self.br.take(5)?.saturating_add(257) as usize;
        let hdist = self.br.take(5)?.saturating_add(1) as usize;
        let hclen = self.br.take(4)?.saturating_add(4) as usize;
        if hlit > 286 || hdist > 30 {
            return Err(corrupt("too many literal/length or distance codes"));
        }
        let mut cl = [0u8; 19];
        for &i in CL_ORDER.iter().take(hclen) {
            if let Some(slot) = cl.get_mut(i) {
                *slot = self.br.take(3)? as u8;
            }
        }
        let clh = Huffman::new(&cl)?;
        let total = hlit.saturating_add(hdist);
        let mut lengths = vec![0u8; total];
        let mut i = 0usize;
        while i < total {
            let sym = clh.decode(&mut self.br)?;
            let (value, repeat) = match sym {
                0..=15 => (sym as u8, 1usize),
                16 => {
                    let prev = i
                        .checked_sub(1)
                        .and_then(|p| lengths.get(p))
                        .copied()
                        .ok_or_else(|| corrupt("repeat code with no previous length"))?;
                    (prev, 3usize.saturating_add(self.br.take(2)? as usize))
                }
                17 => (0, 3usize.saturating_add(self.br.take(3)? as usize)),
                18 => (0, 11usize.saturating_add(self.br.take(7)? as usize)),
                _ => return Err(corrupt("invalid code-length symbol")),
            };
            let end = i.saturating_add(repeat);
            if end > total {
                return Err(corrupt("code lengths overrun the table"));
            }
            for slot in lengths.iter_mut().take(end).skip(i) {
                *slot = value;
            }
            i = end;
        }
        let (ll, dl) = lengths.split_at(hlit);
        if ll.get(256).copied().unwrap_or(0) == 0 {
            return Err(corrupt("dynamic block has no end-of-block code"));
        }
        self.lit = Some(Huffman::new(ll)?);
        self.dist = Some(Huffman::new(dl)?);
        Ok(())
    }

    fn push(&mut self, buf: &mut [u8], n: &mut usize, b: u8) {
        if let Some(slot) = buf.get_mut(*n) {
            *slot = b;
        }
        if let Some(slot) = self.window.get_mut(self.wpos) {
            *slot = b;
        }
        self.wpos = self.wpos.wrapping_add(1) & WMASK;
        self.total = self.total.saturating_add(1);
        *n = n.saturating_add(1);
    }

    /// Decode one symbol of a Huffman block.
    fn symbol(&mut self, fixed: bool, buf: &mut [u8], n: &mut usize) -> io::Result<()> {
        let (lit, dist) = if fixed {
            match fixed_tables() {
                Some((l, d)) => (l, d),
                None => return Err(corrupt("fixed Huffman tables unavailable")),
            }
        } else {
            match (&self.lit, &self.dist) {
                (Some(l), Some(d)) => (l, d),
                _ => return Err(corrupt("Huffman block without tables")),
            }
        };
        let sym = lit.decode(&mut self.br)?;
        if sym < 256 {
            self.limit_check(1)?;
            self.push(buf, n, sym as u8);
            return Ok(());
        }
        if sym == 256 {
            self.state = State::Header;
            return Ok(());
        }
        let li = usize::from(sym.wrapping_sub(257));
        let (Some(base), Some(extra)) = (LEN_BASE.get(li), LEN_EXTRA.get(li)) else {
            return Err(corrupt("invalid literal/length symbol"));
        };
        let len = usize::from(*base).saturating_add(self.br.take(u32::from(*extra))? as usize);
        let dsym = usize::from(dist.decode(&mut self.br)?);
        let (Some(dbase), Some(dextra)) = (DIST_BASE.get(dsym), DIST_EXTRA.get(dsym)) else {
            return Err(corrupt("invalid distance symbol"));
        };
        let d = usize::from(*dbase).saturating_add(self.br.take(u32::from(*dextra))? as usize);
        if d as u64 > self.total {
            return Err(corrupt("distance reaches before the start of the output"));
        }
        self.limit_check(len as u64)?;
        self.copy_len = len;
        self.copy_dist = d;
        Ok(())
    }
}

impl<R: BufRead> Read for Inflater<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0usize;
        while n < buf.len() {
            if self.copy_len > 0 {
                while self.copy_len > 0 && n < buf.len() {
                    let src = self.wpos.wrapping_sub(self.copy_dist) & WMASK;
                    let b = self.window.get(src).copied().unwrap_or(0);
                    self.push(buf, &mut n, b);
                    self.copy_len = self.copy_len.saturating_sub(1);
                }
                continue;
            }
            match self.state {
                State::Done => break,
                State::Header => self.start_block()?,
                State::Stored(0) => self.state = State::Header,
                State::Stored(rem) => {
                    let want = (rem as usize).min(buf.len().saturating_sub(n));
                    self.limit_check(want as u64)?;
                    let mut got = 0usize;
                    while got < want && self.br.count >= 8 {
                        let b = self.br.take(8)? as u8;
                        self.push(buf, &mut n, b);
                        got = got.saturating_add(1);
                    }
                    if got == 0 {
                        let avail = loop {
                            match self.br.inner.fill_buf() {
                                Ok(b) => break b,
                                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                                Err(e) => return Err(e),
                            }
                        };
                        if avail.is_empty() {
                            return Err(truncated());
                        }
                        let k = want.min(avail.len());
                        for (&b, slot) in avail.iter().take(k).zip(buf.iter_mut().skip(n)) {
                            *slot = b;
                            if let Some(w) = self.window.get_mut(self.wpos) {
                                *w = b;
                            }
                            self.wpos = self.wpos.wrapping_add(1) & WMASK;
                        }
                        self.br.inner.consume(k);
                        self.br.pulled = self.br.pulled.saturating_add(k as u64);
                        self.total = self.total.saturating_add(k as u64);
                        n = n.saturating_add(k);
                        got = k;
                    }
                    self.state = State::Stored(rem.saturating_sub(got as u32));
                }
                State::Fixed => self.symbol(true, buf, &mut n)?,
                State::Dynamic => self.symbol(false, buf, &mut n)?,
            }
        }
        Ok(n)
    }
}

/// Decode a complete raw DEFLATE stream held in memory.
///
/// Returns the output and the number of input bytes the stream occupied.
///
/// # Errors
/// `corrupt-archive` for a malformed or truncated stream and `limit` when the
/// output would exceed `max_output`.
pub fn inflate_slice(data: &[u8], max_output: u64) -> Result<(Vec<u8>, usize), Error> {
    let mut inf = Inflater::new(data, max_output);
    let mut out = Vec::new();
    inf.read_to_end(&mut out)
        .map_err(|e| Error::from_io("inflate", &e))?;
    if !inf.is_done() {
        return Err(Error::new(ErrorKind::CorruptArchive, "stream did not end"));
    }
    let used = usize::try_from(inf.compressed_len()).unwrap_or(usize::MAX);
    Ok((out, used))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hand_assembled_fixed_block() {
        // BFINAL=1, BTYPE=01, literal 'a' (8-bit code 0x30+0x61), end of block.
        let (out, used) = inflate_slice(&[0x4B, 0x04, 0x00], 100).expect("decodes");
        assert_eq!(out, b"a");
        assert_eq!(used, 3);
    }

    #[test]
    fn a_stored_block_and_what_follows_it() {
        let data = [0x01, 0x03, 0x00, 0xFC, 0xFF, b'x', b'y', b'z', 0xAA, 0xBB];
        let mut inf = Inflater::new(&data[..], 100);
        let mut out = Vec::new();
        inf.read_to_end(&mut out).expect("decodes");
        assert_eq!(out, b"xyz");
        assert!(inf.is_done());
        assert_eq!(inf.compressed_len(), 8);
        let (left, rest) = inf.into_parts();
        let mut tail = left;
        tail.extend_from_slice(rest);
        assert_eq!(tail, vec![0xAA, 0xBB]);
    }

    #[test]
    fn malformed_streams_are_refused_with_corrupt_archive() {
        for (bad, why) in [
            (&[0x01, 0x03, 0x00, 0xFC, 0xFE, 1, 2, 3][..], "bad NLEN"),
            (&[0x07][..], "block type 3"),
            (&[0x01, 0x03, 0x00, 0xFC, 0xFF, 1][..], "truncated stored"),
            (&[][..], "empty"),
            (&[0x4B][..], "truncated fixed"),
        ] {
            let e = inflate_slice(bad, 100).expect_err(why);
            assert_eq!(e.kind(), ErrorKind::CorruptArchive, "{why}: {e}");
        }
    }

    #[test]
    fn a_distance_before_the_output_start_is_refused() {
        // Fixed block: length symbol 257 (7-bit code 0000001) then distance
        // code 0 (5 bits) with no output before it.
        // Bits: 1,1,0 | 0,0,0,0,0,0,1 | 0,0,0,0,0 | EOB 0000000
        let data = [0x03, 0x02, 0x00, 0x00];
        let e = inflate_slice(&data, 100).expect_err("must fail");
        assert_eq!(e.kind(), ErrorKind::CorruptArchive);
    }

    #[test]
    fn the_output_cap_is_a_limit_error() {
        let data = [0x01, 0x03, 0x00, 0xFC, 0xFF, b'x', b'y', b'z'];
        let e = inflate_slice(&data, 2).expect_err("over the cap");
        assert_eq!(e.kind(), ErrorKind::Limit);
        assert!(inflate_slice(&data, 3).is_ok());
    }

    #[test]
    fn bit_reversal() {
        assert_eq!(reverse_bits(0b1011, 4), 0b1101);
        assert_eq!(reverse_bits(1, 1), 1);
        assert_eq!(reverse_bits(0b100, 3), 0b001);
    }
}
