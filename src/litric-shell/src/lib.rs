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
//! It is **not** yet a Wayland compositor: there are no client surfaces, no
//! protocol and therefore no third-party applications drawing into it. What it
//! does have is its own desktop: input decoding, a tested window manager with
//! focus, workspaces, snapping and multi-window cycling, a system-wide search,
//! a lock surface, built-in applications and a mission-control overview. The
//! compositor protocol is the next layer, and the shell is where it plugs in.
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
pub mod chrome;
pub mod desktop;
pub mod domkey;
pub mod fb;
pub mod font;
pub mod fsbridge;
pub mod i18n;
pub mod icons;
pub mod link;
pub mod lock;
pub mod monitor;
pub mod overview;
pub mod seek;
pub mod settings;
pub mod shell;
pub mod shortcuts;
pub mod taskmgr;
pub mod textaudit;
pub mod textedit;
pub mod ui;
pub mod views;
pub mod wordmark;

#[cfg(test)]
mod e2e;
