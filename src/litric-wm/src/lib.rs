//! LitricOS window manager.
//!
//! # Layer position
//!
//! **Layer 5 (Native ecosystem).** It sits between input and rendering and
//! touches neither: no devices, no framebuffer, no clock, no I/O of any kind.
//! It is told where the pointer is and what happened, and it decides what that
//! means for the windows.
//!
//! ```text
//!   input  ──►  WindowManager  ──►  scene  ──►  compositor  ──►  renderer
//!                    │
//!               take_damage() ──► only these rectangles are repainted
//! ```
//!
//! # Why it is a separate crate
//!
//! ADR-0009 puts the compositor and the shell in one process to save the memory
//! a separate window manager would cost. One process is not one module: keeping
//! the window model in its own crate with no dependencies is what stops "the
//! shell draws a rectangle and remembers where it put it" from becoming the
//! window system by default. Every behaviour here — focus following a close,
//! snapping, a resize hitting its minimum size, a drag that cannot push a title
//! bar under the top bar — is tested without a display.
//!
//! Windows carry a [`window::Content`] that distinguishes a panel the shell
//! draws from a client surface. There are no clients yet; the distinction is
//! here now so the manager is never written in a way that assumes there will
//! not be.

#![forbid(unsafe_code)]

pub mod geom;
pub mod manager;
pub mod window;

pub use geom::{Chrome, Rect};
pub use manager::{Hit, HitZone, ResizeEdge, WindowManager};
pub use window::{Content, SnapEdge, Window, WindowId, WindowState};
