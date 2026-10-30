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
//! * [`providers`] is the write half of audio, network, Bluetooth and
//!   authentication: validated arguments, a base tool per provider, and a plan
//!   that a test can read without running anything.
//! * [`facts`] measures the running machine. Real, and tested against fixtures.
//! * [`platform`] reports what the machine *is*: interfaces, sound cards,
//!   power, Bluetooth radios, sessions. Real, and tested against fixtures.
//! * [`action`] is the one thing a session can *do* to the machine: a power
//!   transition, through a runner the tests substitute, gated by the broker.
//! * [`proto`] is the wire format. Real, and tested.
//! * [`state`] answers requests from the live subsystems. Real, and tested.
//! * [`server`] is the Unix socket transport. Real on Unix; absent elsewhere.
//!
//! Not real yet, and not pretended anywhere in the replies: the trash is in
//! memory rather than read from disk, and an environment switch moves the state
//! machine without starting a virtual machine.

#![forbid(unsafe_code)]

pub mod action;
pub mod auth;
pub mod facts;
pub mod grants;
pub mod launch;
pub mod lockout;
pub mod platform;
pub mod proto;
pub mod providers;
pub mod state;
pub mod update;
mod update_health;
pub mod update_lock;

#[cfg(unix)]
pub mod server;
