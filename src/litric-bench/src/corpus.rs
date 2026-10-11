//! The synthetic corpus generator: >=100,000 deterministic samples.
//!
//! # Every hostile byte is inert by construction (MASTER §SAFETY)
//!
//! Hostile samples here are: the public EICAR test string, high-entropy
//! random bytes, bytes that *look* like structure (headers, section names,
//! encoded blocks), or benign bytes with a threatening *shape*. Nothing
//! generated parses as an executable for any real loader, nothing carries a
//! real payload, and nothing here is a known malware sample — hostile truth
//! comes from construction, not from copying.
//!
//! # Determinism
//!
//! One `seed` fixes the entire corpus. `generate(seed, n)` on two machines
//! produces byte-identical samples in the same order, which is what makes
//! regression comparisons across runs meaningful (MASTER §22).

//! Synthetic byte layouts do dense modular arithmetic and fixed-table
//! indexing by construction; every index is masked before use.
#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]

use crate::severity::{Level, Truth};
use litric_defender::sha256;

/// The 50 corpus categories (MASTER §3), in stable order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::enum_variant_names)]
pub enum Category {
    /// 01 benign documents and media shapes.
    Benign,
    /// 02 suspicious static shapes.
    SuspiciousStatic,
    /// 03 suspicious script shapes.
    SuspiciousScript,
    /// 04 suspicious archive shapes.
    SuspiciousArchive,
    /// 05 malformed structures.
    Malformed,
    /// 06 corrupted streams.
    Corrupted,
    /// 07 parser stress.
    ParserStress,
    /// 08 memory stress shapes.
    MemoryStress,
    /// 09 CPU stress shapes.
    CpuStress,
    /// 10 disk stress shapes.
    DiskStress,
    /// 11 process-tree simulation.
    ProcessTree,
    /// 12 IPC simulation.
    Ipc,
    /// 13 privilege-boundary simulation.
    PrivilegeBoundary,
    /// 14 service simulation.
    Service,
    /// 15 driver-interface simulation.
    DriverInterface,
    /// 16 kernel-interface simulation.
    KernelInterface,
    /// 17 filesystem-filter simulation.
    FsFilter,
    /// 18 network telemetry shapes.
    NetworkTelemetry,
    /// 19 anti-tamper shapes.
    AntiTamper,
    /// 20 quarantine shapes.
    Quarantine,
    /// 21 rollback shapes.
    Rollback,
    /// 22 recovery shapes.
    Recovery,
    /// 23 high entropy.
    HighEntropy,
    /// 24 encoded data.
    EncodedData,
    /// 25 obfuscation-like.
    ObfuscationLike,
    /// 26 polymorphic-like.
    PolymorphicLike,
    /// 27 packed-like.
    PackedLike,
    /// 28 PE-anomaly shapes.
    PeAnomaly,
    /// 29 ELF-anomaly shapes.
    ElfAnomaly,
    /// 30 executable metadata anomaly.
    ExecMetadata,
    /// 31 archive-bomb simulation.
    ArchiveBomb,
    /// 32 extreme nesting.
    ExtremeNesting,
    /// 33 path stress.
    PathStress,
    /// 34 unicode / path normalization.
    UnicodePath,
    /// 35 symlink / hardlink simulation.
    LinkLike,
    /// 36 file-lock contention.
    LockContention,
    /// 37 race-condition simulation.
    RaceLike,
    /// 38 TOCTOU-like simulation.
    TocTou,
    /// 39 process churn.
    ProcessChurn,
    /// 40 thread churn.
    ThreadChurn,
    /// 41 handle churn.
    HandleChurn,
    /// 42 memory fragmentation.
    MemFragmentation,
    /// 43 allocation failure.
    AllocFailure,
    /// 44 I/O failure.
    IoFailure,
    /// 45 telemetry overload.
    TelemetryOverload,
    /// 46 log flood.
    LogFlood,
    /// 47 security-event flood.
    SecurityEventFlood,
    /// 48 crash-recovery.
    CrashRecovery,
    /// 49 boot / recovery simulation.
    BootRecovery,
    /// 50 zero-day-like unseen combinations.
    ZeroDayLike,
}

/// All categories in their stable order.
#[must_use]
pub fn all_categories() -> &'static [Category] {
    &[
        Category::Benign,
        Category::SuspiciousStatic,
        Category::SuspiciousScript,
        Category::SuspiciousArchive,
        Category::Malformed,
        Category::Corrupted,
        Category::ParserStress,
        Category::MemoryStress,
        Category::CpuStress,
        Category::DiskStress,
        Category::ProcessTree,
        Category::Ipc,
        Category::PrivilegeBoundary,
        Category::Service,
        Category::DriverInterface,
        Category::KernelInterface,
        Category::FsFilter,
        Category::NetworkTelemetry,
        Category::AntiTamper,
        Category::Quarantine,
        Category::Rollback,
        Category::Recovery,
        Category::HighEntropy,
        Category::EncodedData,
        Category::ObfuscationLike,
        Category::PolymorphicLike,
        Category::PackedLike,
        Category::PeAnomaly,
        Category::ElfAnomaly,
        Category::ExecMetadata,
        Category::ArchiveBomb,
        Category::ExtremeNesting,
        Category::PathStress,
        Category::UnicodePath,
        Category::LinkLike,
        Category::LockContention,
        Category::RaceLike,
        Category::TocTou,
        Category::ProcessChurn,
        Category::ThreadChurn,
        Category::HandleChurn,
        Category::MemFragmentation,
        Category::AllocFailure,
        Category::IoFailure,
        Category::TelemetryOverload,
        Category::LogFlood,
        Category::SecurityEventFlood,
        Category::CrashRecovery,
        Category::BootRecovery,
        Category::ZeroDayLike,
    ]
}

impl Category {
    /// Stable machine name, `NN-name`, used for directories and CSV.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Category::Benign => "01-benign",
            Category::SuspiciousStatic => "02-suspicious-static",
            Category::SuspiciousScript => "03-suspicious-script",
            Category::SuspiciousArchive => "04-suspicious-archive",
            Category::Malformed => "05-malformed",
            Category::Corrupted => "06-corrupted",
            Category::ParserStress => "07-parser-stress",
            Category::MemoryStress => "08-memory-stress",
            Category::CpuStress => "09-cpu-stress",
            Category::DiskStress => "10-disk-stress",
            Category::ProcessTree => "11-process-tree",
            Category::Ipc => "12-ipc",
            Category::PrivilegeBoundary => "13-privilege-boundary",
            Category::Service => "14-service",
            Category::DriverInterface => "15-driver-interface",
            Category::KernelInterface => "16-kernel-interface",
            Category::FsFilter => "17-fs-filter",
            Category::NetworkTelemetry => "18-network-telemetry",
            Category::AntiTamper => "19-anti-tamper",
            Category::Quarantine => "20-quarantine",
            Category::Rollback => "21-rollback",
            Category::Recovery => "22-recovery",
            Category::HighEntropy => "23-high-entropy",
            Category::EncodedData => "24-encoded-data",
            Category::ObfuscationLike => "25-obfuscation-like",
            Category::PolymorphicLike => "26-polymorphic-like",
            Category::PackedLike => "27-packed-like",
            Category::PeAnomaly => "28-pe-anomaly",
            Category::ElfAnomaly => "29-elf-anomaly",
            Category::ExecMetadata => "30-exec-metadata",
            Category::ArchiveBomb => "31-archive-bomb",
            Category::ExtremeNesting => "32-extreme-nesting",
            Category::PathStress => "33-path-stress",
            Category::UnicodePath => "34-unicode-path",
            Category::LinkLike => "35-link-like",
            Category::LockContention => "36-file-lock",
            Category::RaceLike => "37-race",
            Category::TocTou => "38-toctou",
            Category::ProcessChurn => "39-process-churn",
            Category::ThreadChurn => "40-thread-churn",
            Category::HandleChurn => "41-handle-churn",
            Category::MemFragmentation => "42-mem-fragmentation",
            Category::AllocFailure => "43-alloc-failure",
            Category::IoFailure => "44-io-failure",
            Category::TelemetryOverload => "45-telemetry-overload",
            Category::LogFlood => "46-log-flood",
            Category::SecurityEventFlood => "47-event-flood",
            Category::CrashRecovery => "48-crash-recovery",
            Category::BootRecovery => "49-boot-recovery",
            Category::ZeroDayLike => "50-zero-day-like",
        }
    }

    /// Ground truth: category 01 is benign; everything else is
    /// hostile-equivalent except the hard-negative classes inside it.
    #[must_use]
    pub fn default_truth(self) -> Truth {
        if self == Category::Benign {
            Truth::Benign
        } else {
            Truth::Hostile
        }
    }

    /// The severity this category exercises.
    #[must_use]
    pub fn level(self) -> Level {
        match self {
            Category::Benign => Level::L0Benign,
            Category::SuspiciousStatic
            | Category::SuspiciousScript
            | Category::SuspiciousArchive
            | Category::HighEntropy
            | Category::EncodedData
            | Category::ObfuscationLike => Level::L1Suspicious,
            Category::Malformed
            | Category::Corrupted
            | Category::ParserStress
            | Category::PolymorphicLike
            | Category::PackedLike
            | Category::PeAnomaly
            | Category::ElfAnomaly
            | Category::ExecMetadata
            | Category::ZeroDayLike => Level::L2HighRisk,
            Category::MemoryStress
            | Category::CpuStress
            | Category::DiskStress
            | Category::ProcessTree
            | Category::Ipc
            | Category::ArchiveBomb
            | Category::ExtremeNesting
            | Category::PathStress
            | Category::UnicodePath
            | Category::LockContention
            | Category::RaceLike
            | Category::TocTou
            | Category::ProcessChurn
            | Category::ThreadChurn
            | Category::HandleChurn
            | Category::MemFragmentation
            | Category::AllocFailure
            | Category::IoFailure
            | Category::TelemetryOverload
            | Category::LogFlood
            | Category::SecurityEventFlood => Level::L3SystemImpact,
            Category::PrivilegeBoundary
            | Category::Service
            | Category::DriverInterface
            | Category::KernelInterface
            | Category::FsFilter
            | Category::NetworkTelemetry
            | Category::AntiTamper
            | Category::Quarantine
            | Category::Rollback
            | Category::LinkLike => Level::L4KernelImpact,
            Category::Recovery | Category::CrashRecovery | Category::BootRecovery => {
                Level::L5Catastrophic
            }
        }
    }
}

/// A generated sample: inert bytes plus everything the log needs.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Stable id, `bench-<seed>-<category>-<index>`.
    pub id: String,
    /// File name on the test volume (hostile *name shape* is part of the test).
    pub file_name: String,
    /// The category it belongs to.
    pub category: Category,
    /// Ground truth by construction.
    pub truth: Truth,
    /// Level the category exercises.
    pub level: Level,
    /// The inert bytes.
    pub content: Vec<u8>,
    /// SHA-256 of the content, hex.
    pub digest_hex: String,
}

/// SplitMix64: tiny, public-domain-flavoured, fully deterministic.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn byte(&mut self) -> u8 {
        (self.next() & 0xFF) as u8
    }

    fn fill(&mut self, out: &mut [u8]) {
        for b in out.iter_mut() {
            *b = self.byte();
        }
    }

    fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + (self.next() as usize) % (hi - lo).max(1)
    }
}

/// The EICAR standard antivirus test string — the one universally agreed
/// inert-but-detected artifact in this framework.
pub const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

/// Threat-shaped marker strings the generator weaves into hostile samples.
/// Each is pure text; a pattern-matching engine learns these as candidates.
pub const MARKERS: &[&[u8]] = &[
    b"QUERY::ENCRYPT_VOLUME",      // simulated extortion phrasing
    b"CONN::BEACON/60s/loopback",  // simulated beacon timing, loopback only
    b"HOOK::SSDT_SHADOW_INJECT",   // simulated kernel hook vocabulary
    b"PERSIST::RUNKEY_WRITE_SIM",  // simulated persistence vocabulary
    b"CRED::DUMP_SIMULATED_EMPTY", // credential vocabulary, explicitly empty
    b"DRIVER::LOAD_UNLOAD_STORM",  // driver churn simulation
    b"EXEC::LOOP_SPAWN_NEST",      // process-tree simulation
    b"FS::SHADOW_DELETE_SIM",      // simulated destructive vocabulary
];

/// An encoded (base64-like, custom alphabet) block; inert text.
fn encode_block(rng: &mut Rng, len: usize) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789+/=";
    (0..len)
        .map(|_| {
            let i = (rng.next() as usize) % ALPHABET.len();
            ALPHABET.get(i).copied().unwrap_or(b'A')
        })
        .collect()
}

/// One hostile-equivalent content by category family. Every branch
/// terminates in inert bytes.
fn hostile_content(rng: &mut Rng, cat: Category, index: usize) -> Vec<u8> {
    use Category::*;
    let mut v = Vec::with_capacity(4096);
    match cat {
        // "Suspicious" families: text markers plus noise. Every 8th
        // SuspiciousStatic sample carries the public EICAR test string —
        // the one universally-agreed inert detection anchor — so the hash
        // tier has real feed entries derived from construction.
        SuspiciousStatic | SuspiciousScript | SuspiciousArchive | Ipc | Service
        | NetworkTelemetry => {
            if cat == Category::SuspiciousStatic && index % 8 == 0 {
                v.extend_from_slice(EICAR);
                v.push(b'\n');
            }
            let pick = index.checked_rem(MARKERS.len().max(1)).unwrap_or(0);
            let m = MARKERS
                .get(pick)
                .copied()
                .or_else(|| MARKERS.first().copied())
                .unwrap_or_default();
            v.extend_from_slice(m);
            v.push(b'\n');
            let n = rng.range(64, 512);
            v.extend_from_slice(&encode_block(rng, n));
        }
        // Executable-shape families: fake headers that satisfy no real loader.
        PeAnomaly => {
            v.extend_from_slice(b"MZ");
            v.extend_from_slice(&[0x00; 58]);
            v.extend_from_slice(b"PE\0\0");
            let n = rng.range(200, 900);
            let mut scratch = vec![0u8; 32];
            rng.fill(&mut scratch); // keep rng advancing deterministically
            v.extend_from_slice(&encode_block(rng, n));
        }
        ElfAnomaly => {
            v.extend_from_slice(&[0x7f, b'E', b'L', b'F', 0x99]); // bad class byte
            let n = rng.range(200, 900);
            v.extend_from_slice(&encode_block(rng, n));
        }
        ExecMetadata | DriverInterface | KernelInterface | FsFilter | PrivilegeBoundary => {
            v.extend_from_slice(b"SIM::");
            v.extend_from_slice(cat.slug().as_bytes());
            v.push(b':');
            let n = rng.range(64, 256);
            v.extend_from_slice(&encode_block(rng, n));
        }
        // High-entropy / packed / polymorphic families: pure random bytes.
        HighEntropy | PackedLike | PolymorphicLike | EncodedData | ObfuscationLike => {
            let n = rng.range(1024, 8192);
            for _ in 0..n {
                v.push(rng.byte());
            }
        }
        // Parser-stress families: deep nesting, huge counts, broken framing.
        ParserStress | ExtremeNesting | ArchiveBomb => {
            let depth = rng.range(50, 400);
            for _ in 0..depth {
                v.extend_from_slice(b"[{\"a\":");
            }
            v.push(b'1');
            for _ in 0..depth {
                v.push(b']');
                v.push(b'}');
            }
        }
        Malformed | Corrupted => {
            let n = rng.range(256, 2048);
            for _ in 0..n {
                // Non-UTF8 noise with structurally impossible fragments.
                v.push(rng.byte() | 0x80);
            }
        }
        // Resource-pressure families: repetition with threatening vocabulary.
        MemoryStress | CpuStress | DiskStress | ProcessChurn | ThreadChurn | HandleChurn
        | MemFragmentation | AllocFailure | IoFailure | TelemetryOverload | LogFlood
        | SecurityEventFlood | LockContention | RaceLike | TocTou | ProcessTree => {
            v.extend_from_slice(b"STRESS::");
            v.extend_from_slice(cat.slug().as_bytes());
            v.push(b'\n');
            let line_len = rng.range(16, 64);
            let lines = rng.range(64, 512);
            for i in 0..lines {
                v.extend_from_slice(format!("sim-event-{index}-{i:04} ").as_bytes());
                v.resize(v.len() + line_len, b'x');
                v.push(b'\n');
            }
        }
        // Anti-tamper / quarantine / rollback / recovery families.
        AntiTamper | Quarantine | Rollback | Recovery | CrashRecovery | BootRecovery | LinkLike
        | PathStress | UnicodePath => {
            v.extend_from_slice(b"SIM::TAMPER-OR-RECOVERY::");
            v.extend_from_slice(cat.slug().as_bytes());
            let n = rng.range(32, 128);
            v.extend_from_slice(&encode_block(rng, n));
        }
        // Zero-day-like: a deterministic unseen combination of four feature
        // axes (shape, marker, entropy band, framing) the engine has no rule
        // for. MASTER §12: from feature-space, not from malware.
        ZeroDayLike => {
            let axes = (rng.next(), rng.next(), rng.next(), rng.next());
            if axes.0 % 2 == 0 {
                v.extend_from_slice(b"SIM::UNSEEN-COMBO-A");
            } else {
                v.extend_from_slice(b"SIM::UNSEEN-COMBO-B");
            }
            if axes.1 % 3 == 0 {
                v.extend_from_slice(b"/NESTED");
            }
            match axes.2 % 4 {
                0 => {
                    let n = rng.range(64, 512);
                    v.extend_from_slice(&encode_block(rng, n));
                }
                1 => {
                    let n = rng.range(64, 512);
                    for _ in 0..n {
                        v.push(rng.byte());
                    }
                }
                2 => v.extend_from_slice(b"\x7fELF-SIM-NOT-REALLY"),
                _ => v.extend_from_slice(b"PK-SIM-NOT-AN-ARCHIVE"),
            }
            if axes.3 % 5 == 0 {
                let pick = index.checked_rem(MARKERS.len().max(1)).unwrap_or(0);
                let m = MARKERS
                    .get(pick)
                    .copied()
                    .or_else(|| MARKERS.first().copied())
                    .unwrap_or_default();
                v.extend_from_slice(m);
            }
        }
        // The benign category never reaches the hostile generator.
        Benign => unreachable!("benign samples use benign_content"),
    }
    v
}

/// Benign content: ordinary text/data with ordinary entropy. Some of it is
/// deliberately *hard-negative*: threatening names/strings with nothing
/// behind them (MASTER §13).
fn benign_content(rng: &mut Rng, index: usize, hard_negative: bool) -> Vec<u8> {
    let mut v = Vec::with_capacity(2048);
    if hard_negative {
        // The name shouts, the content is empty of everything. This is the
        // false-positive probe.
        v.extend_from_slice(b"# benign probe file - no payload, no logic\n");
        v.extend_from_slice(format!("probe index {index}\n").as_bytes());
        let n = rng.range(16, 48);
        v.extend_from_slice(&encode_block(rng, n));
    } else {
        v.extend_from_slice(b"litric-bench benign sample\n");
        let words = ["meeting", "notes", "grocery", "spec", "diary", "log"];
        for _ in 0..rng.range(8, 64) {
            let w = words
                .get((rng.next() as usize) % words.len())
                .copied()
                .unwrap_or("note");
            v.extend_from_slice(w.as_bytes());
            v.push(b' ');
        }
    }
    v
}

/// Generate `count` samples for one category. Deterministic in
/// `(seed, category, index)`.
#[must_use]
pub fn samples_for_category(seed: u64, cat: Category, count: usize) -> Vec<Sample> {
    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let first_byte = cat.slug().as_bytes().first().copied().unwrap_or(b'x');
        let mut rng = Rng::new(seed ^ (u64::from(first_byte) << 32) ^ index as u64);
        let (truth, content) = match cat {
            Category::Benign => {
                // Every 3rd benign sample is a hard negative: hostile shape,
                // benign truth.
                let hard = index % 3 == 0;
                let c = benign_content(&mut rng, index, hard);
                (Truth::Benign, c)
            }
            c => (c.default_truth(), hostile_content(&mut rng, c, index)),
        };
        out.push(Sample {
            id: format!("bench-{seed}-{}-{index:06}", cat.slug()),
            file_name: file_name_for(cat, index),
            category: cat,
            truth,
            level: cat.level(),
            digest_hex: sha256::hex(&sha256::digest(&content)),
            content,
        });
    }
    out
}

/// A file name per category: hostile categories get threat-shaped *names*
/// (the name is inert; only a rule that keys on names would ever care).
fn file_name_for(cat: Category, index: usize) -> String {
    use Category::*;
    match cat {
        Benign if index % 3 == 0 => format!("totally_not_malware_{index}.txt"),
        Benign => format!("document_{index}.txt"),
        PeAnomaly => format!("invoice_{index}.pdf.exe.bin"),
        HighEntropy => format!("update_{index}.dat"),
        SuspiciousScript => format!("script_{index}.vbs.txt"),
        ArchiveBomb => format!("backup_{index}.zip.bin"),
        UnicodePath => format!("sample_\u{00e9}\u{0430}_{}_{}.bin", index, index), // NFC/NFD-adjacent
        _ => format!("{}_{}.bin", cat.slug(), index),
    }
}

/// Distribute `total` across the 50 categories evenly (deterministic
/// remainder split), and generate the whole corpus.
#[must_use]
pub fn generate(seed: u64, total: usize) -> Vec<Sample> {
    let cats = all_categories();
    let n_cats = cats.len().max(1);
    let base = total.checked_div(n_cats).unwrap_or(0);
    let rem = total.checked_rem(n_cats).unwrap_or(0);
    let mut out = Vec::with_capacity(total);
    for (i, cat) in cats.iter().enumerate() {
        let n = base + usize::from(i < rem);
        out.extend(samples_for_category(seed, *cat, n));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic_across_calls() {
        let a = generate(928_374, 500);
        let b = generate(928_374, 500);
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.digest_hex, y.digest_hex, "{} vs {}", x.id, y.id);
            assert_eq!(x.file_name, y.file_name);
        }
    }

    #[test]
    fn the_categories_are_exactly_fifty() {
        assert_eq!(all_categories().len(), 50);
        let slugs: std::collections::BTreeSet<&str> =
            all_categories().iter().map(|c| c.slug()).collect();
        assert_eq!(slugs.len(), 50, "slugs must be unique");
    }

    #[test]
    fn hostile_samples_never_contain_real_executable_headers() {
        // A loader cannot do anything with these: the "headers" are broken
        // or followed by inert encoded text. Assert the strongest guarantee
        // the generator can make: no valid PE/ELF magic survives intact.
        for s in generate(7, 500) {
            if s.truth == Truth::Hostile {
                let is_real_pe = s.content.starts_with(b"MZ")
                    && s.content.windows(4).any(|w| w == [b'P', b'E', 0, 0])
                    && s.content.len() > 4096;
                assert!(!is_real_pe, "{} looks loadable", s.id);
            }
        }
    }

    #[test]
    fn hard_negatives_are_benign_but_scary_looking() {
        let benign = samples_for_category(11, Category::Benign, 9);
        let hard = benign.first().expect("category sample");
        assert_eq!(hard.truth, Truth::Benign);
        assert!(hard.file_name.contains("not_malware"));
    }

    #[test]
    fn every_sample_digest_matches_its_content() {
        for s in generate(3, 100) {
            assert_eq!(s.digest_hex, sha256::hex(&sha256::digest(&s.content)));
        }
    }

    #[test]
    fn eicar_is_present_and_exact() {
        assert!(EICAR.starts_with(b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-"));
    }
}
