//! LitricOS shared foundation types.
//!
//! Everything in this crate is part of the **public system contract**. Changing
//! a type here is an ABI event, not a refactor — see `docs/adr/` and
//! `LITRIC_NATIVE_RUNTIME_ABI_SECURITY_v1.md` section 14.
//!
//! Design rules this crate follows, from RUNTIME section 42:
//!
//! * Structured errors, never strings. Applications must not parse English to
//!   decide what to do next.
//! * Opaque handles for system-owned objects, so internal layout can change
//!   without breaking compiled binaries.
//! * Explicit validation at construction. A value of type [`AppId`] is a
//!   *proof* that the string was checked; downstream code never re-validates.

#![forbid(unsafe_code)]

pub mod error;
pub mod handle;
pub mod id;
pub mod pct;
pub mod version;

pub use error::{Error, ErrorKind, Result};
pub use handle::{Handle, HandleKind};
pub use id::{AppId, IdError};
pub use version::{AbiVersion, Version, VersionError};
