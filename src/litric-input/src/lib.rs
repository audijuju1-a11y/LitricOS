//! LitricOS input.
//!
//! # Layer position
//!
//! **Layer 4 (Runtime).** It depends on nothing above the kernel's own
//! interfaces and knows nothing about rendering, which is the whole design:
//! the desktop reacts to *what the user did*, never to what a particular device
//! reported.
//!
//! ```text
//!   /dev/input/event*  ─►  evdev   raw records
//!                          state   modifiers, pointer, clicks
//!                          keys    layout: scancode -> key and text
//!                          event   what the desktop consumes
//!   /sys/class/input   ─►  device  discovery and classification
//!                          backend threads, channel, InputSource
//! ```
//!
//! # Backend independence
//!
//! Everything above [`event::InputEvent`] is free of evdev. The desktop holds
//! an [`backend::InputSource`], so replacing the backend — libinput, a Wayland
//! seat, a remote session, a replayed recording — is one implementation of one
//! trait. [`backend::ScriptedSource`] is that already: it drives the whole
//! stack from an array in a test, on a machine with no input devices.
//!
//! # No `unsafe`
//!
//! Devices are opened with `File::open`, read with `Read::read`, and classified
//! from text in sysfs. Reads stay thread-per-device so nothing here needs
//! non-blocking flags. There is exactly one exception to the no-unsafe rule:
//! asking an absolute pointer for its real coordinate range takes a single
//! `EVIOCGABS` ioctl — see `backend::absinfo` — because no sysfs file carries
//! it and guessing the range is precisely the bug this exists to prevent.

#![deny(unsafe_code)]

pub mod backend;
pub mod device;
pub mod evdev;
pub mod event;
pub mod keys;
pub mod state;

pub use backend::{BackendReport, EvdevSource, InputSource, ScriptedSource, Waker};
pub use event::{InputEvent, Point, PointerButton};
pub use keys::{Key, Layout, Modifiers, Side};
pub use state::InputState;
