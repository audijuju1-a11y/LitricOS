//! LitricOS shell: the desktop that appears at boot.
//!
//! # Layer position
//!
//! **Layer 5 (Native ecosystem).** It sits on `litric-sysd` for what is true
//! about the machine and on `litric-design` for what the interface should look
//! like, and it turns both into pixels.
//!
//! # What this is, and what it is not
//!
//! It **is** a real compositor output path: it discovers the framebuffer,
//! renders the whole desktop in software with anti-aliased geometry and a real
//! typeface, solves the glass alpha against the pixels actually behind each
//! panel, and writes the frame to the display. Everything on screen is drawn by
//! this code from numbers measured by the system.
//!
//! It is **not** yet a Wayland compositor. There is no input handling, no
//! client applications and no window management, because those need a protocol
//! implementation and clients to talk it. What is here is the half that has to
//! exist first and the half that proves the design works on real hardware: a
//! machine that boots to this is a machine with a desktop on it, and the next
//! layer has somewhere to draw into.
//!
//! ```text
//!   fb       find the framebuffer, encode a frame, write it
//!   canvas   anti-aliased software raster: shapes, gradients, blur
//!   font     LitricOS Sans, a stroke typeface rendered at any size
//!   desktop  the composition, filled with measured system state
//! ```

#![forbid(unsafe_code)]

pub mod activity;
pub mod anim;
pub mod apps;
pub mod canvas;
pub mod desktop;
pub mod domkey;
pub mod fb;
pub mod font;
pub mod fsbridge;
pub mod icons;
pub mod link;
pub mod lock;
pub mod monitor;
pub mod shell;
pub mod shortcuts;
pub mod taskmgr;
pub mod textaudit;
pub mod textedit;
pub mod ui;
pub mod views;

#[cfg(test)]
mod e2e;
