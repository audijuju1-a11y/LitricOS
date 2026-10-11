//! LitricOS Defender — policy core (ADR-0022 Phase 0).
//!
//! This crate is the detection brain without the body: hash verdicts,
//! byte-pattern matching, a bounded verdict cache and the fail-open policy
//! ladder. It contains **no I/O, no process execution and no
//! third-party dependencies** — by the same supply-chain rule as the other
//! core crates, because this code decides what the machine may execute.
//!
//! Phase roadmap (ADR-0022): Phase 0 is this pure core, testable on any
//! host. Phase 2 attaches a Linux fanotify transport; Phase 3 moves the
//! hook points into the Litric kernel. The core's API is designed so those
//! phases change the transport, never the decision logic.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
#![deny(missing_docs)]

pub mod cache;
pub mod denylist;
pub mod engine;
pub mod patterns;
pub mod sha256;
pub mod verdict;
