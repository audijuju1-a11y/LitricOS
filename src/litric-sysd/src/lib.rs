//! LitricOS system daemon, as a library.
//!
//! # Layer position
//!
//! **Layer 4 (Runtime).** Everything below — the capability engine, the storage
//! broker, the trash policy, the environment manager — is real code with
//! passing tests and no way to run. This is what runs it, and the binary in
//! `main.rs` is a thin shell around it.
//!
//! It is a library and not only a binary for two reasons. The client half of
//! the protocol is what the shell and the settings application will use to talk
//! to the daemon, and they should link it rather than reimplement it. And a
//! binary-only crate hides its own API from the compiler's dead-code analysis
//! on any platform where part of it is `cfg`-ed out, which turns a real "this
//! is never called" warning into noise that gets ignored.
//!
//! # What is real, and what is not
//!
//! * [`facts`] measures the running machine. Real, and tested against fixtures.
//! * [`proto`] is the wire format. Real, and tested.
//! * [`state`] answers requests from the live subsystems. Real, and tested.
//! * [`server`] is the Unix socket transport. Real on Unix; absent elsewhere.
//!
//! Not real yet, and not pretended anywhere in the replies: the trash is in
//! memory rather than read from disk, and an environment switch moves the state
//! machine without starting a virtual machine.

#![forbid(unsafe_code)]

pub mod facts;
pub mod proto;
pub mod state;

#[cfg(unix)]
pub mod server;
