//! The keyboard keymap a client needs to make sense of `wl_keyboard.key`.
//!
//! Wayland sends key *codes*, not characters: the client turns 24 into "q"
//! using a keymap the compositor hands it as a file descriptor. That keymap is
//! a compiled XKB description. Generating one at runtime would need libxkbcommon
//! and a set of XKB rules on the image, so this crate ships a resolved map,
//! produced once from the standard `us` layout, and documents the gap: a user
//! who switches to another layout switches the *shell's* decoding, not the
//! client's keymap. Closing that properly means generating the map when the
//! layout changes; it is listed as a known limitation in ADR-0024 rather than
//! hidden behind a layout that the client half-applies.
//!
//! `src/keymap.xkb` is that map, `xkb_keymap_get_as_string` output of
//! `xkb_keymap_new_from_names(layout="us")`, with no includes left to resolve.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

use crate::StartError;

/// The shipped map.
const KEYMAP: &str = include_str!("keymap.xkb");

/// The name the map is written under inside the runtime directory.
const FILE_NAME: &str = "litric-keymap.xkb";

/// A resolved keymap, ready to hand to any number of keyboards.
#[derive(Debug)]
pub struct Keymap {
    file: File,
    size: u32,
}

impl Keymap {
    /// Write the map into the runtime directory and keep it open.
    ///
    /// The file is unlinked immediately: a client may keep the descriptor for
    /// as long as it likes, and no other process should be able to rewrite what
    /// a keyboard is already using.
    pub fn load(runtime: &Path) -> io::Result<Self> {
        let path = runtime.join(FILE_NAME);
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)?;
        file.write_all(KEYMAP.as_bytes())?;
        // The map is a C string: the size a client maps includes the NUL.
        file.write_all(b"\0")?;
        file.flush()?;
        let size = u32::try_from(KEYMAP.len().saturating_add(1)).unwrap_or(u32::MAX);
        file.seek(SeekFrom::Start(0))?;
        // Nothing else needs the name; the descriptor is the keymap now.
        let _ = fs::remove_file(&path);
        Ok(Self { file, size })
    }

    /// A fresh descriptor over the same bytes. Wayland transfers ownership of
    /// the descriptor, so every keyboard gets its own.
    pub fn dup(&self) -> io::Result<File> {
        self.file.try_clone()
    }

    /// Length including the terminating NUL, which is what the protocol wants.
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }
}

/// Bind the listening socket at `socket`.
///
/// `wayland-server` owns the lock file and the unlink-if-stale dance, which is
/// exactly the part that is easy to get subtly wrong by hand.
pub fn listen(socket: &Path) -> Result<wayland_server::ListeningSocket, StartError> {
    wayland_server::ListeningSocket::bind_absolute(socket.to_path_buf())
        .map_err(|e| StartError::Bind(e.to_string()))
}
