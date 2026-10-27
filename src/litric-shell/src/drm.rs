//! The KMS output path: finished frames handed to the display controller.
//!
//! # Why there is a second output path
//!
//! ADR-0017 put the desktop on `/dev/fb0` because the workspace forbids
//! `unsafe` and an fbdev frame needs no ioctl. That was the right first stage
//! and it is no longer enough: fbdev is a compatibility shim, it cannot tell
//! the driver *when* to show a frame, and on QEMU's DRM bridge it accepts a
//! `write(2)` without publishing those pixels to the scanout at all. The
//! desktop could be composed perfectly and never appear.
//!
//! This module drives the real output path through [`HELPER`], a small C
//! program that owns every unsafe operation (see `drmpresent.c`). The shell
//! keeps what it already had: pixels, layout, policy, and safe Rust. What
//! changes is that a frame is now a set of damage bands written into a dumb
//! buffer the card is already scanning out, and `P` asks the driver for a
//! page flip at the next vertical blank. The flip is performed by the display
//! controller, not copied by the CPU, so a frame never tears and no 4 MB
//! travels through a pipe per repaint.
//!
//! # The contract
//!
//! ```text
//!   helper stdout   LITRIC-DRM/1 <width> <height> <pitch> <connector>
//!   helper stdin    B <y> <rows>\n <rows × pitch bytes, XRGB8888>
//!                   P\n            flip
//!                   Q\n            clean exit (EOF is also clean)
//! ```
//!
//! Every band is written into *both* buffers by the helper, which is what
//! makes a partial update safe: the buffer being scanned out can never show a
//! newer band over an older frame.
//!
//! # What this module does not claim
//!
//! The raster is still software. `P` moves *which* finished buffer is on
//! screen; it does not shade anything. A GPU rasteriser, dmabuf import and a
//! GL compositor are separate decisions, and ADR-0026 records them as not
//! made. The protocol is verified against a stand-in helper in this module's
//! tests; the ioctl sequence is compiled and has never run on a real card in
//! this repository's evidence.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use crate::canvas::{Canvas, Rect};

/// Where the helper is installed in the image.
pub const HELPER: &str = "/usr/bin/litric-drmpresent";

/// How long the helper gets to publish its mode before it is presumed stuck.
///
/// The helper prints its handshake before it reads a single frame, so this is
/// a bound on a broken card, not on normal startup. Without it a wedged
/// driver would hang the shell before the first pixel, which looks exactly
/// like a machine that did not boot.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

/// Longest handshake line accepted. A driver name cannot be longer than this,
/// and a helper that prints more is not speaking this protocol.
const MAX_HANDSHAKE: u64 = 128;

/// What the helper negotiated with the card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrmInfo {
    /// Mode width in pixels.
    pub width: u32,
    /// Mode height in pixels.
    pub height: u32,
    /// Bytes per row of the scanout buffer, as the driver chose it.
    pub pitch: u32,
    /// Connector the mode was set on, e.g. `eDP-1`.
    pub connector: String,
}

impl DrmInfo {
    /// Parse the helper's one-line handshake.
    ///
    /// Returns `None` for anything that is not exactly this protocol with
    /// sane geometry. A pitch narrower than one pixel row cannot describe a
    /// buffer this shell can write, and a zero-sized mode would produce a
    /// frame the helper would then refuse — better to fall back to fbdev than
    /// to half-start.
    #[must_use]
    pub fn parse(line: &str) -> Option<Self> {
        let mut parts = line.split_whitespace();
        if parts.next()? != "LITRIC-DRM/1" {
            return None;
        }
        let width: u32 = parts.next()?.parse().ok()?;
        let height: u32 = parts.next()?.parse().ok()?;
        let pitch: u32 = parts.next()?.parse().ok()?;
        let connector = parts.next()?.to_owned();
        if parts.next().is_some() {
            return None;
        }
        if width == 0 || height == 0 {
            return None;
        }
        let minimum = width.checked_mul(4)?;
        if pitch < minimum {
            return None;
        }
        let _ = (pitch as u64).checked_mul(u64::from(height))?;
        Some(Self {
            width,
            height,
            pitch,
            connector,
        })
    }

    /// Bytes in one complete frame.
    #[must_use]
    pub const fn frame_bytes(&self) -> u64 {
        (self.pitch as u64).saturating_mul(self.height as u64)
    }

    /// One line for the status file and the console.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{}x{} pitch={} connector={} format=XRGB8888 flip=pageflip",
            self.width, self.height, self.pitch, self.connector
        )
    }
}

/// Find the first card node under `root` (normally `/`).
///
/// `card0` is tried first, then a short range: a machine with two adapters can
/// expose the one driving the panel at a higher index.
#[must_use]
pub fn discover_card(root: &Path) -> Option<PathBuf> {
    for n in 0..4u32 {
        let dev = root.join(format!("dev/dri/card{n}"));
        if dev.exists() {
            return Some(dev);
        }
    }
    None
}

/// Turn damage rectangles into row bands, clipped to the screen and merged.
///
/// The helper patches whole rows into the scanout buffer, so a clip that
/// spans rows is the unit of work. Merging touching bands matters: a window
/// drag produces dozens of overlapping rectangles per frame, and sending
/// thirty bands of two rows each is thirty `write` calls for one repaint.
#[must_use]
pub fn bands(damage: &[Rect], height: u32) -> Vec<(u32, u32)> {
    let limit = height as f32;
    let mut rows: Vec<(u32, u32)> = Vec::new();
    for r in damage {
        let top = r.y.floor();
        let bottom = r.bottom().ceil();
        if !top.is_finite() || !bottom.is_finite() {
            continue;
        }
        // f32 as u32 saturates in Rust, so an off-screen rectangle lands on
        // the edge instead of wrapping to a huge row.
        let y0 = top.clamp(0.0, limit) as u32;
        let y1 = bottom.clamp(0.0, limit) as u32;
        if y1 <= y0 {
            continue;
        }
        rows.push((y0, y1));
    }
    rows.sort_unstable();
    let mut merged: Vec<(u32, u32)> = Vec::with_capacity(rows.len());
    for (y0, y1) in rows {
        match merged.last_mut() {
            Some(last) if y0 <= last.1 => last.1 = last.1.max(y1),
            _ => merged.push((y0, y1)),
        }
    }
    merged
}

/// Encode `rows` rows of `canvas` starting at `y0` into `out`, `pitch` bytes
/// per row, in the `XRGB8888` little-endian layout the card was told about.
///
/// Columns past the canvas (a canvas narrower than the mode) and the padding
/// after the last pixel of a padded row are zeroed, because those bytes are
/// part of the frame the card scans out.
fn encode_band(canvas: &Canvas, y0: u32, rows: u32, pitch: u32, out: &mut Vec<u8>) {
    let pitch = pitch as usize;
    let total = pitch.saturating_mul(rows as usize);
    out.clear();
    out.resize(total, 0);
    // Four bytes per pixel is the format this path declares; a padded row
    // holds whole pixels and some slack, so the pixel count is the pitch
    // divided by four, never the mode width alone.
    let columns = pitch.saturating_div(4).min(canvas.width() as usize);
    for row in 0..rows {
        let y = u64::from(y0).saturating_add(u64::from(row));
        let base = (row as usize).saturating_mul(pitch);
        for x in 0..columns {
            let (Ok(xi), Ok(yi)) = (i64::try_from(x), i64::try_from(y)) else {
                continue;
            };
            let Some(px) = canvas.get(xi, yi) else {
                continue;
            };
            let offset = base.saturating_add(x.saturating_mul(4));
            let Some(slot) = out.get_mut(offset..offset.saturating_add(4)) else {
                continue;
            };
            let r = ((px >> 16) & 0xFF) as u8;
            let g = ((px >> 8) & 0xFF) as u8;
            let b = (px & 0xFF) as u8;
            slot.copy_from_slice(&[b, g, r, 0xFF]);
        }
    }
}

/// A running `litric-drmpresent`, with the mode it negotiated.
#[derive(Debug)]
pub struct DrmPresenter {
    info: DrmInfo,
    child: Child,
    stdin: Option<ChildStdin>,
    band: Vec<u8>,
}

impl DrmPresenter {
    /// Start the helper on `card` and wait for its handshake.
    ///
    /// # Errors
    /// The helper is missing, cannot be started, exits before publishing a
    /// mode, or publishes something that is not this protocol.
    pub fn open(card: &Path) -> std::io::Result<Self> {
        Self::open_with(Path::new(HELPER), card)
    }

    /// Same as [`Self::open`] but with an explicit helper path, so the
    /// protocol can be tested against a stand-in process.
    ///
    /// # Errors
    /// As [`Self::open`].
    pub fn open_with(helper: &Path, card: &Path) -> std::io::Result<Self> {
        let mut child = Command::new(helper)
            .arg(card)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("drm helper did not provide stdout"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("drm helper did not provide stdin"))?;

        // Read the handshake on its own thread so a helper that never speaks
        // is a timeout instead of a hung shell. The thread owns stdout and
        // dies with the pipe.
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let read = BufReader::new(stdout)
                .take(MAX_HANDSHAKE)
                .read_line(&mut line)
                .map(|n| (n, line));
            let _ = tx.send(read);
        });
        let read = match rx.recv_timeout(HANDSHAKE_TIMEOUT) {
            Ok(read) => read,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "{} did not publish a mode within {} s",
                        helper.display(),
                        HANDSHAKE_TIMEOUT.as_secs()
                    ),
                ));
            }
        };
        let line = match read {
            Ok((n, line)) if n > 0 => line,
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(std::io::Error::other(format!(
                    "{} exited before publishing a mode on {}",
                    helper.display(),
                    card.display()
                )));
            }
        };
        let Some(info) = DrmInfo::parse(line.trim_end()) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::other(format!(
                "{} published an unreadable mode: {}",
                helper.display(),
                line.trim_end()
            )));
        };
        Ok(Self {
            info,
            child,
            stdin: Some(stdin),
            band: Vec::new(),
        })
    }

    /// The negotiated mode.
    #[must_use]
    pub const fn info(&self) -> &DrmInfo {
        &self.info
    }

    /// Send a whole frame and flip.
    ///
    /// # Errors
    /// Any I/O error from the helper's pipe, which means the helper is gone.
    pub fn present(&mut self, canvas: &Canvas) -> std::io::Result<u32> {
        self.send_bands(canvas, &[(0, self.info.height)])
    }

    /// Send only the rows covered by `damage`, then flip.
    ///
    /// Returns the number of rows actually sent, so the status file states
    /// what was done rather than what was asked for.
    ///
    /// # Errors
    /// Any I/O error from the helper's pipe.
    pub fn present_damage(&mut self, canvas: &Canvas, damage: &[Rect]) -> std::io::Result<u32> {
        let bands = bands(damage, self.info.height);
        if bands.is_empty() {
            return Ok(0);
        }
        self.send_bands(canvas, &bands)
    }

    fn send_bands(&mut self, canvas: &Canvas, bands: &[(u32, u32)]) -> std::io::Result<u32> {
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "drm helper is already closed",
            ));
        };
        let mut sent = 0u32;
        for (y0, y1) in bands {
            let rows = y1.saturating_sub(*y0);
            if rows == 0 {
                continue;
            }
            encode_band(canvas, *y0, rows, self.info.pitch, &mut self.band);
            stdin.write_all(format!("B {} {}\n", y0, rows).as_bytes())?;
            stdin.write_all(&self.band)?;
            sent = sent.saturating_add(rows);
        }
        stdin.write_all(b"P\n")?;
        stdin.flush()?;
        Ok(sent)
    }

    /// Ask the helper to exit and reap it.
    ///
    /// Called from `Drop` as well, so a shell that is being killed still gives
    /// the card back rather than leaving a dead master behind.
    ///
    /// # Errors
    /// Any I/O error from closing the helper's stdin.
    pub fn close(&mut self) -> std::io::Result<()> {
        let mut result = Ok(());
        if let Some(mut stdin) = self.stdin.take() {
            result = stdin.write_all(b"Q\n").and_then(|()| stdin.flush());
            // Dropping stdin closes the pipe; the helper exits on either the
            // command or EOF, so both are clean.
            drop(stdin);
        }
        // Give it a moment to drop DRM master on its own, then insist. A
        // helper stuck in a driver call must not keep the shell from exiting.
        for _ in 0..30 {
            match self.child.try_wait() {
                Ok(Some(_)) => return result,
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(e) => return Err(e),
            }
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        result
    }
}

impl Drop for DrmPresenter {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::canvas::rgb;
    use std::fs;

    fn fixture(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("litric-shell-drm-{name}"));
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).expect("fixture dir");
        dir
    }

    /// A stand-in helper: prints one handshake, then copies stdin to a file.
    /// It exercises this module's protocol exactly as the real helper would,
    /// on a machine with no card node.
    fn stub(dir: &Path, handshake: &str) -> PathBuf {
        let path = dir.join("stub-drmpresent.sh");
        let script = format!(
            "#!/bin/sh\nprintf '{}\\n'\nexec cat > \"$(dirname \"$0\")/out.bin\"\n",
            handshake
        );
        fs::write(&path, script).expect("stub script");
        let status = Command::new("chmod")
            .arg("0755")
            .arg(&path)
            .status()
            .expect("chmod");
        assert!(status.success(), "chmod failed");
        path
    }

    #[test]
    fn a_handshake_parses_into_geometry() {
        let info = DrmInfo::parse("LITRIC-DRM/1 1280 800 5120 eDP-1").expect("parses");
        assert_eq!((info.width, info.height), (1280, 800));
        assert_eq!(info.pitch, 5120);
        assert_eq!(info.connector, "eDP-1");
        assert_eq!(info.frame_bytes(), 4_096_000);
        assert!(info.summary().contains("XRGB8888"));
    }

    #[test]
    fn anything_that_is_not_the_protocol_is_refused() {
        for line in [
            "",
            "LITRIC-DRM/1",
            "LITRIC-DRM/1 1280 800",
            "LITRIC-DRM/2 1280 800 5120 eDP-1",
            "LITRIC-DRM/1 0 800 5120 eDP-1",
            "LITRIC-DRM/1 1280 0 5120 eDP-1",
            "LITRIC-DRM/1 1280 800 4096 eDP-1",
            "LITRIC-DRM/1 1280 800 5120",
            "LITRIC-DRM/1 1280 800 5120 eDP-1 extra",
            "LITRIC-DRM/1 -1280 800 5120 eDP-1",
            "nonsense",
        ] {
            assert!(DrmInfo::parse(line).is_none(), "accepted {line:?}");
        }
    }

    #[test]
    fn damage_becomes_merged_row_bands() {
        // Two rectangles on the same rows are one band, not two.
        let merged = bands(
            &[
                Rect::new(0.0, 10.0, 100.0, 20.0),
                Rect::new(200.0, 10.0, 50.0, 20.0),
            ],
            800,
        );
        assert_eq!(merged, vec![(10, 30)]);
    }

    #[test]
    fn bands_touch_merge_and_gaps_stay_separate() {
        let touching = bands(
            &[
                Rect::new(0.0, 0.0, 10.0, 5.0),
                Rect::new(0.0, 5.0, 10.0, 5.0),
            ],
            800,
        );
        assert_eq!(touching, vec![(0, 10)], "touching bands merge");
        let apart = bands(
            &[
                Rect::new(0.0, 0.0, 10.0, 5.0),
                Rect::new(0.0, 6.0, 10.0, 2.0),
            ],
            800,
        );
        assert_eq!(apart, vec![(0, 5), (6, 8)], "a gap is two bands");
    }

    #[test]
    fn off_screen_and_empty_damage_is_clipped_away() {
        assert!(bands(&[], 800).is_empty());
        assert!(bands(&[Rect::new(0.0, 900.0, 10.0, 10.0)], 800).is_empty());
        assert!(bands(&[Rect::new(0.0, -20.0, 10.0, 10.0)], 800).is_empty());
        assert_eq!(
            bands(&[Rect::new(0.0, -20.0, 10.0, 40.0)], 800),
            vec![(0, 20)],
            "clipped to the top edge"
        );
        assert_eq!(
            bands(&[Rect::new(0.0, 790.0, 10.0, 40.0)], 800),
            vec![(790, 800)],
            "clipped to the bottom edge"
        );
    }

    #[test]
    fn a_band_is_pitch_bytes_per_row_with_the_canvas_inside_it() {
        let mut canvas = Canvas::new(2, 2, rgb(0x102030));
        canvas.put(0, 1, rgb(0xFF0000));
        let mut out = Vec::new();
        // A padded row: 2 pixels need 8 bytes, the driver says 12.
        encode_band(&canvas, 0, 2, 12, &mut out);
        assert_eq!(out.len(), 24, "two rows of 12 bytes");
        assert_eq!(&out[0..4], &[0x30, 0x20, 0x10, 0xFF], "first pixel");
        assert_eq!(&out[8..12], &[0, 0, 0, 0], "row padding is zeroed");
        assert_eq!(&out[12..16], &[0x00, 0x00, 0xFF, 0xFF], "row 1 pixel");
    }

    #[test]
    fn a_canvas_narrower_than_the_mode_leaves_the_rest_black() {
        let canvas = Canvas::new(1, 1, rgb(0xFFFFFF));
        let mut out = Vec::new();
        encode_band(&canvas, 0, 1, 16, &mut out);
        assert_eq!(out.len(), 16);
        assert_eq!(&out[0..4], &[0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(
            out[4..].iter().all(|b| *b == 0),
            "uncovered columns are zero"
        );
    }

    #[test]
    fn a_full_frame_is_one_band_then_a_flip() {
        let dir = fixture("full");
        let helper = stub(&dir, "LITRIC-DRM/1 2 2 8 eDP-1");
        let card = dir.join("card0");
        fs::write(&card, b"").expect("card fixture");
        let mut presenter = DrmPresenter::open_with(&helper, &card).expect("handshake");
        assert_eq!(presenter.info().width, 2);

        let canvas = Canvas::new(2, 2, rgb(0x102030));
        assert_eq!(presenter.present(&canvas).expect("present"), 2);
        presenter.close().expect("close");

        let bytes = fs::read(dir.join("out.bin")).expect("protocol capture");
        assert_eq!(&bytes[..6], b"B 0 2\n", "header names the band");
        // header, payload, flip, then the quit that close() sends
        assert_eq!(bytes.len(), 6 + 16 + 4, "header, payload, flip, quit");
        assert_eq!(&bytes[6..10], &[0x30, 0x20, 0x10, 0xFF]);
        assert_eq!(&bytes[22..], b"P\nQ\n");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn damage_sends_only_its_bands_and_still_flips() {
        let dir = fixture("damage");
        let helper = stub(&dir, "LITRIC-DRM/1 2 4 8 eDP-1");
        let card = dir.join("card0");
        fs::write(&card, b"").expect("card fixture");
        let mut presenter = DrmPresenter::open_with(&helper, &card).expect("handshake");
        let canvas = Canvas::new(2, 4, rgb(0xFFFFFF));
        let sent = presenter
            .present_damage(&canvas, &[Rect::new(0.0, 2.0, 2.0, 1.0)])
            .expect("present");
        assert_eq!(sent, 1, "one row was sent");
        presenter.close().expect("close");

        let bytes = fs::read(dir.join("out.bin")).expect("protocol capture");
        assert_eq!(&bytes[..6], b"B 2 1\n");
        assert_eq!(bytes.len(), 6 + 8 + 4);
        assert_eq!(&bytes[14..16], b"P\n");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn damage_that_covers_nothing_writes_nothing_at_all() {
        let dir = fixture("nodamage");
        let helper = stub(&dir, "LITRIC-DRM/1 2 4 8 eDP-1");
        let card = dir.join("card0");
        fs::write(&card, b"").expect("card fixture");
        let mut presenter = DrmPresenter::open_with(&helper, &card).expect("handshake");
        let canvas = Canvas::new(2, 4, rgb(0xFFFFFF));
        assert_eq!(presenter.present_damage(&canvas, &[]).expect("present"), 0);
        presenter.close().expect("close");
        let bytes = fs::read(dir.join("out.bin")).unwrap_or_default();
        assert_eq!(bytes, b"Q\n", "no frame was sent, only the quit");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_helper_that_never_speaks_is_an_error_not_a_hang() {
        let dir = fixture("silent");
        let helper = dir.join("silent.sh");
        fs::write(&helper, "#!/bin/sh\nexec sleep 30\n").expect("script");
        Command::new("chmod")
            .arg("0755")
            .arg(&helper)
            .status()
            .expect("chmod");
        let card = dir.join("card0");
        fs::write(&card, b"").expect("card fixture");
        let started = std::time::Instant::now();
        let err = DrmPresenter::open_with(&helper, &card).expect_err("must refuse");
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
        assert!(
            started.elapsed() < HANDSHAKE_TIMEOUT + Duration::from_secs(2),
            "took {:?}",
            started.elapsed()
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_helper_that_exits_is_an_error_that_names_the_card() {
        let dir = fixture("exits");
        let helper = dir.join("exit.sh");
        fs::write(&helper, "#!/bin/sh\nexit 1\n").expect("script");
        Command::new("chmod")
            .arg("0755")
            .arg(&helper)
            .status()
            .expect("chmod");
        let card = dir.join("card0");
        fs::write(&card, b"").expect("card fixture");
        let err = DrmPresenter::open_with(&helper, &card).expect_err("must refuse");
        assert!(
            err.to_string().contains("before publishing a mode"),
            "{err}"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_nonsense_handshake_is_refused_with_what_was_said() {
        let dir = fixture("nonsense");
        let helper = stub(&dir, "LITRIC-DRM/1 2 4 4 eDP-1");
        let card = dir.join("card0");
        fs::write(&card, b"").expect("card fixture");
        let err = DrmPresenter::open_with(&helper, &card).expect_err("must refuse");
        assert!(err.to_string().contains("unreadable mode"), "{err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn card_discovery_prefers_the_lowest_index() {
        let dir = fixture("cards");
        fs::create_dir_all(dir.join("dev/dri")).expect("dri dir");
        assert!(discover_card(&dir).is_none(), "no card, no guess");
        fs::write(dir.join("dev/dri/card1"), b"").expect("card1");
        assert_eq!(
            discover_card(&dir),
            Some(dir.join("dev/dri/card1")),
            "finds the only card"
        );
        fs::write(dir.join("dev/dri/card0"), b"").expect("card0");
        assert_eq!(
            discover_card(&dir),
            Some(dir.join("dev/dri/card0")),
            "card0 wins"
        );
        fs::remove_dir_all(&dir).ok();
    }
}
