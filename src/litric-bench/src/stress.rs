//! Bounded stress: catastrophic pressure with hard ceilings (MASTER §5–§7).
//!
//! "Catastrophic-equivalent" does not mean unbounded. Every generator here
//! takes an explicit budget and refuses to exceed it — the framework must
//! never become the outage it is trying to simulate. Filesystem stress
//! shapes (deep trees, long names, many entries) are produced as *sample
//! content* plus the plan to materialize them inside the TEST_VOLUME only;
//! the host itself is never touched.

//! Budget math is checked_div/saturating at call sites; the PRNG module
//! performs masked modular arithmetic by contract.
#![allow(clippy::arithmetic_side_effects)]

use crate::severity::Level;

// NOTE: Rng is crate-private in corpus; expose a thin local PRNG here so
// stress stays self-contained and the corpus module need not widen its API.
mod rng {
    //! SplitMix64, identical constants to corpus's private generator.
    pub struct Rng(u64);
    impl Rng {
        pub fn new(seed: u64) -> Self {
            Self(seed)
        }
        pub fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
    }
}
use rng::Rng;

/// A hard ceiling every stress plan must respect. The CLI's
/// --cpu-limit/--memory-limit/--disk-limit map onto these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// Maximum bytes the stress may write, total.
    pub max_bytes: u64,
    /// Maximum files the stress may create, total.
    pub max_files: u64,
    /// Maximum wall-clock seconds the stress may run.
    pub max_seconds: u64,
}

impl Budget {
    /// The conservative default: enough to prove the point on any host.
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            max_bytes: 64 * 1024 * 1024,
            max_files: 10_000,
            max_seconds: 60,
        }
    }

    /// True when `files` files of `bytes_each` bytes fit inside this budget.
    #[must_use]
    pub const fn allows(self, files: u64, bytes_each: u64) -> bool {
        files <= self.max_files && files.saturating_mul(bytes_each) <= self.max_bytes
    }
}

/// One planned filesystem stress action (MASTER §5), materialized only
/// inside the test volume by the CLI layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsAction {
    /// Create `count` files of `bytes_each` under one directory.
    MassiveCreate {
        /// How many files to create.
        count: u64,
        /// Bytes per file.
        bytes_each: u64,
    },
    /// Rename `count` files in one directory pass.
    MassiveRename {
        /// How many files to rename.
        count: u64,
    },
    /// Delete `count` files created earlier in the same run.
    MassiveDelete {
        /// How many files to delete.
        count: u64,
    },
    /// A directory tree `depth` levels deep, `fanout` per level.
    DeepTree {
        /// Levels below the volume root.
        depth: u32,
        /// Directories per level.
        fanout: u32,
    },
    /// Long-path stress: name of `name_len` bytes inside depth-4 tree.
    LongPath {
        /// File-name length in bytes.
        name_len: u64,
    },
    /// Unicode normalization + case collision names.
    UnicodeCollision {
        /// How many colliding names to create.
        count: u64,
    },
}

/// One planned memory stress action (MASTER §6). Allocation happens in the
/// sandbox harness, bounded by `bytes`, never on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemAction {
    /// One large allocation of `bytes`.
    LargeObject {
        /// Allocation size in bytes.
        bytes: u64,
    },
    /// `count` allocations of `bytes_each`, held simultaneously.
    ManySmall {
        /// How many allocations.
        count: u64,
        /// Bytes per allocation.
        bytes_each: u64,
    },
    /// Allocate/free `rounds` times to fragment the arena.
    AllocFreeChurn {
        /// How many allocate/free rounds.
        rounds: u64,
        /// Bytes per round.
        bytes_each: u64,
    },
}

/// One planned CPU stress action (MASTER §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuAction {
    /// Burn one core for `millis`.
    SingleCore {
        /// Duration in milliseconds.
        millis: u64,
    },
    /// Burn `cores` cores for `millis`.
    MultiCore {
        /// How many cores.
        cores: u32,
        /// Duration in milliseconds.
        millis: u64,
    },
    /// Spawn/exit `count` short-lived threads.
    ThreadChurn {
        /// How many thread lifecycles.
        count: u64,
    },
    /// Spawn/exit `count` no-op child *simulations* (bookkeeping only in
    /// Phase 0: process churn is measured through the engine scan loop).
    ProcessChurn {
        /// How many simulated lifecycles.
        count: u64,
    },
}

/// One planned telemetry/log flood (MASTER §45–§47 in the corpus list and
/// §14's TELEMETRY_EVENT/SECURITY_EVENT classes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloodAction {
    /// `count` log lines in one burst.
    LogFlood {
        /// How many lines.
        count: u64,
    },
    /// `count` synthetic security events in one burst.
    SecurityEventFlood {
        /// How many events.
        count: u64,
    },
    /// `count` telemetry batches with oversized payloads.
    TelemetryOverload {
        /// How many batches.
        count: u64,
        /// Bytes per batch.
        bytes_each: u64,
    },
}

/// A deterministic stress plan: what the chaos-free path executes in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressPlan {
    /// Filesystem actions, in execution order.
    pub fs: Vec<FsAction>,
    /// Memory actions, in execution order.
    pub mem: Vec<MemAction>,
    /// CPU actions, in execution order.
    pub cpu: Vec<CpuAction>,
    /// Flood actions, in execution order.
    pub floods: Vec<FloodAction>,
    /// The budget every action must fit inside.
    pub budget: Budget,
    /// The severity level this plan exercises.
    pub level: Level,
}

/// Build a deterministic plan for `level` under `budget`. Same seed in,
/// same plan out — MASTER §14's reproducibility requirement applies to
/// stress exactly as it applies to the corpus.
#[must_use]
pub fn plan(seed: u64, level: Level, budget: Budget) -> StressPlan {
    let mut rng = Rng::new(seed);
    let churn_rounds = rng.next() % 512 + 64;
    let mut plan = StressPlan {
        fs: Vec::new(),
        mem: Vec::new(),
        cpu: Vec::new(),
        floods: Vec::new(),
        budget,
        level,
    };

    // Filesystem: the fs family gets 60% of the byte budget, leaving room
    // for the memory and telemetry families under the same hard ceiling.
    let files = (budget.max_files.checked_div(4).unwrap_or(0)).max(16);
    let fs_bytes = budget
        .max_bytes
        .checked_div(5)
        .unwrap_or(0)
        .saturating_mul(3);
    let bytes_each = fs_bytes.checked_div(files).unwrap_or(1).clamp(1, 64 * 1024);
    plan.fs.push(FsAction::MassiveCreate {
        count: files,
        bytes_each,
    });
    plan.fs.push(FsAction::MassiveRename { count: files });
    plan.fs.push(FsAction::MassiveDelete { count: files });
    plan.fs.push(FsAction::DeepTree {
        depth: 6,
        fanout: 3,
    });
    plan.fs.push(FsAction::LongPath { name_len: 255 });
    plan.fs.push(FsAction::UnicodeCollision { count: 32 });

    // Memory: hard ceiling = the budget's byte cap, each family bounded.
    let mem_each = budget.max_bytes.checked_div(8).unwrap_or(0);
    let small_count = mem_each.checked_div(4096).unwrap_or(0).clamp(1, 1024);
    plan.mem.push(MemAction::LargeObject { bytes: mem_each });
    plan.mem.push(MemAction::ManySmall {
        count: small_count,
        bytes_each: 4096,
    });
    plan.mem.push(MemAction::AllocFreeChurn {
        rounds: churn_rounds,
        bytes_each: 8192,
    });

    // CPU: one core always; multi-core only from Level Three upward.
    plan.cpu.push(CpuAction::SingleCore { millis: 250 });
    if level.number() >= 3 {
        plan.cpu.push(CpuAction::MultiCore {
            cores: 2,
            millis: 250,
        });
    }
    plan.cpu.push(CpuAction::ThreadChurn { count: 128 });

    // Floods: telemetry pressure rises with level.
    plan.floods.push(FloodAction::LogFlood {
        count: 1_000 * u64::from(level.number().max(1)),
    });
    if level.number() >= 2 {
        plan.floods.push(FloodAction::SecurityEventFlood {
            count: 500 * u64::from(level.number()),
        });
        let tel_count = budget
            .max_bytes
            .checked_div(32)
            .unwrap_or(0)
            .checked_div(16 * 1024)
            .unwrap_or(0)
            .clamp(1, 64);
        plan.floods.push(FloodAction::TelemetryOverload {
            count: tel_count,
            bytes_each: 16 * 1024,
        });
    }

    plan
}

/// Total bytes the plan would write if executed (the CLI prints this and
/// refuses to run when it exceeds the budget — a plan cannot lie).
#[must_use]
pub fn planned_bytes(plan: &StressPlan) -> u64 {
    let mut total = 0u64;
    for a in &plan.fs {
        total = total.saturating_add(match *a {
            FsAction::MassiveCreate { count, bytes_each } => count.saturating_mul(bytes_each),
            FsAction::LongPath { name_len } => name_len,
            FsAction::UnicodeCollision { count } => count.saturating_mul(64),
            _ => 0,
        });
    }
    for m in &plan.mem {
        total = total.saturating_add(match *m {
            MemAction::LargeObject { bytes } => bytes,
            MemAction::ManySmall { count, bytes_each } => count.saturating_mul(bytes_each),
            MemAction::AllocFreeChurn { bytes_each, .. } => bytes_each,
        });
    }
    for f in &plan.floods {
        total = total.saturating_add(match *f {
            FloodAction::TelemetryOverload { count, bytes_each } => {
                count.saturating_mul(bytes_each)
            }
            _ => 0,
        });
    }
    total
}

/// Total files the plan would touch.
#[must_use]
pub fn planned_files(plan: &StressPlan) -> u64 {
    let mut total = 0u64;
    for a in &plan.fs {
        total = total.saturating_add(match *a {
            FsAction::MassiveCreate { count, .. }
            | FsAction::MassiveRename { count }
            | FsAction::MassiveDelete { count }
            | FsAction::UnicodeCollision { count } => count,
            FsAction::DeepTree { depth, fanout } => {
                // Geometric sum, saturating: fanout^depth can explode.
                let mut n = 0u64;
                let mut term = 1u64;
                for _ in 0..depth {
                    term = term.saturating_mul(u64::from(fanout));
                    n = n.saturating_add(term);
                }
                n
            }
            FsAction::LongPath { .. } => 1,
        });
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_is_deterministic_per_seed() {
        let b = Budget::conservative();
        let p1 = plan(928_374, Level::L2HighRisk, b);
        let p2 = plan(928_374, Level::L2HighRisk, b);
        assert_eq!(p1, p2, "same seed must give the same plan");
        let p3 = plan(928_375, Level::L2HighRisk, b);
        assert_ne!(p1, p3, "different seeds must differ somewhere");
    }

    #[test]
    fn plan_never_exceeds_budget() {
        let b = Budget::conservative();
        let p = plan(7, Level::L3SystemImpact, b);
        assert!(planned_bytes(&p) <= b.max_bytes, "byte ceiling is hard");
        assert!(planned_files(&p) <= b.max_files, "file ceiling is hard");
        for a in &p.fs {
            if let FsAction::MassiveCreate { count, bytes_each } = *a {
                assert!(b.allows(count, bytes_each));
            }
        }
    }

    #[test]
    fn level_gates_escalation() {
        let b = Budget::conservative();
        let l1 = plan(5, Level::L1Suspicious, b);
        let l5 = plan(5, Level::L5Catastrophic, b);
        assert!(l1
            .cpu
            .iter()
            .all(|c| !matches!(c, CpuAction::MultiCore { .. })));
        assert!(l5
            .cpu
            .iter()
            .any(|c| matches!(c, CpuAction::MultiCore { .. })));
        assert!(l1.floods.len() < l5.floods.len());
    }

    #[test]
    fn deep_tree_file_count_is_geometric_and_saturating() {
        let p = StressPlan {
            fs: vec![FsAction::DeepTree {
                depth: 4,
                fanout: 2,
            }],
            mem: vec![],
            cpu: vec![],
            floods: vec![],
            budget: Budget::conservative(),
            level: Level::L1Suspicious,
        };
        // 2 + 4 + 8 + 16 = 30
        assert_eq!(planned_files(&p), 30);
        let huge = StressPlan {
            fs: vec![FsAction::DeepTree {
                depth: 200,
                fanout: 4,
            }],
            mem: vec![],
            cpu: vec![],
            floods: vec![],
            budget: Budget::conservative(),
            level: Level::L1Suspicious,
        };
        // Must saturate, not panic.
        assert!(planned_files(&huge) >= 1);
    }
}
