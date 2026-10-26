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
//! It **is** a Wayland compositor, in the sense that matters: a real server —
//! the litric-compositor crate, ADR-0024 — listens on the session's
//! wayland-0 socket, and a foreign process that connects gets a LitricOS
//! window: the title bar is the shell's, the pixels are the client's, and input
//! reaches it through the window manager's z-order. What it is not is XWayland,
//! clipboard or drag-and-drop, an input method, or a GPU import path; those are
//! listed in the ADR rather than implied here.
//!
//! On top of that it has its own desktop: input decoding, a tested window
//! manager with focus, workspaces, snapping and multi-window cycling, a
//! system-wide search, a lock surface, built-in applications and a
//! mission-control overview.
//!
//! ```text
//!   fb       find the framebuffer, encode a frame, write it
//!   canvas   anti-aliased software raster: shapes, gradients, blur
//!   font     LitricOS Sans, a stroke typeface rendered at any size
//!   clients  a client's shared-memory surface as a window, frame by frame
//!   desktop  the composition, filled with measured system state
//! ```

#![forbid(unsafe_code)]

pub mod activity;
pub mod anim;
pub mod appicons;
pub mod apps;
pub mod canvas;
pub mod chrome;
pub mod clients;
pub mod desktop;
pub mod domkey;
pub mod fb;
pub mod font;
pub mod fsbridge;
#[cfg(unix)]
pub mod fsclient;
pub mod i18n;
pub mod icons;
pub mod inbox;
pub mod link;
pub mod lock;
pub mod monitor;
pub mod overview;
pub mod prefstore;
pub mod registry;
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
