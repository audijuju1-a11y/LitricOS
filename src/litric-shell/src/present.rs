//! Which surface the desktop is presented to, and why that one.
//!
//! Two output paths exist and they are not equal. `/dev/dri/cardN` through
//! [`crate::drm`] is the real one: hardware page flips, damage-only transfers,
//! and a connector whose mode the driver actually accepted. `/dev/fbN`
//! through [`crate::fb`] is the compatibility path kept for machines whose
//! card has no KMS driver and for fixture hosts that have no card at all.
//!
//! The rule this module enforces is that the choice is **stated**, never
//! inferred silently. The status file always contains one `present=` line that
//! says which path was used, and when the preferred path was not available it
//! says what the machine reported instead. A desktop that quietly fell back
//! looks identical on screen to one that worked, right up to the point where
//! it is asked for a frame rate.

use std::io;
use std::path::{Path, PathBuf};

use crate::canvas::{Canvas, Rect};
use crate::drm::{self, DrmPresenter};
use crate::fb::{self, Framebuffer, PixelFormat};

/// What the operator asked for on the command line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Choice {
    /// DRM/KMS when the machine offers it, framebuffer otherwise.
    #[default]
    Auto,
    /// DRM/KMS only; a machine without it is an error rather than a silent
    /// downgrade.
    Drm,
    /// The framebuffer even where a card exists. Used by fixtures and by a
    /// machine whose KMS driver is suspected of lying.
    Fbdev,
}

impl Choice {
    /// Parse the `--presenter` value.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "auto" => Some(Self::Auto),
            "drm" | "kms" => Some(Self::Drm),
            "fbdev" | "fb" => Some(Self::Fbdev),
            _ => None,
        }
    }
}

/// An open display surface.
#[derive(Debug)]
pub enum Output {
    /// KMS scanout through the native helper.
    Drm(DrmPresenter),
    /// fbdev writes or the mapped presenter pipe.
    Fb(Framebuffer),
}

impl Output {
    /// Present a whole frame.
    ///
    /// Returns the number of rows that actually reached the screen, which is
    /// the number the status file reports.
    ///
    /// # Errors
    /// Any I/O error from the underlying surface.
    pub fn present(&mut self, canvas: &Canvas) -> io::Result<u32> {
        match self {
            Self::Drm(p) => p.present(canvas),
            Self::Fb(f) => f.present(canvas).map(|()| f.info().height),
        }
    }

    /// Present only the damaged regions.
    ///
    /// # Errors
    /// Any I/O error from the underlying surface.
    pub fn present_damage(&mut self, canvas: &Canvas, damage: &[Rect]) -> io::Result<u32> {
        match self {
            Self::Drm(p) => p.present_damage(canvas, damage),
            Self::Fb(f) => f.present_damage(canvas, damage),
        }
    }

    /// Screen width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        match self {
            Self::Drm(p) => p.info().width,
            Self::Fb(f) => f.info().width,
        }
    }

    /// Screen height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        match self {
            Self::Drm(p) => p.info().height,
            Self::Fb(f) => f.info().height,
        }
    }

    /// `drm` or `fbdev`, for tests and diagnostics.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Drm(_) => "drm",
            Self::Fb(_) => "fbdev",
        }
    }

    /// One line describing the negotiated surface.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Drm(p) => p.info().summary(),
            Self::Fb(f) => f.info().summary(),
        }
    }
}

/// A surface plus the status lines that describe how it was chosen.
#[derive(Debug)]
pub struct Opened {
    /// The surface to draw onto.
    pub output: Output,
    /// Lines for the console and the status file, in the order they happened.
    pub lines: Vec<String>,
}

impl Opened {
    /// Screen width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.output.width()
    }

    /// Screen height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.output.height()
    }
}

fn open_drm(root: &Path, helper: &Path) -> Result<(DrmPresenter, PathBuf), String> {
    let Some(card) = drm::discover_card(root) else {
        return Err("no card node under /dev/dri".to_owned());
    };
    if !helper.is_file() {
        return Err(format!("{} is not installed", helper.display()));
    }
    let presenter = DrmPresenter::open_with(helper, &card).map_err(|e| e.to_string())?;
    Ok((presenter, card))
}

/// Open the best surface the machine offers.
///
/// # Errors
/// No usable surface at all, or a specific path was required and missing.
pub fn open(root: &Path, choice: Choice, format: Option<PixelFormat>) -> Result<Opened, String> {
    open_with(root, choice, format, Path::new(drm::HELPER))
}

/// [`open`] with an explicit helper path, so the selection rules can be
/// tested on a machine that has neither a card nor the installed helper.
///
/// # Errors
/// As [`open`].
pub fn open_with(
    root: &Path,
    choice: Choice,
    format: Option<PixelFormat>,
    helper: &Path,
) -> Result<Opened, String> {
    let mut lines: Vec<String> = Vec::new();
    if choice != Choice::Fbdev {
        match open_drm(root, helper) {
            Ok((presenter, card)) => {
                lines.push("present=drm".to_owned());
                lines.push(format!("drm.card={}", card.display()));
                lines.push(format!("drm={}", presenter.info().summary()));
                lines.push("target=drm-kms".to_owned());
                return Ok(Opened {
                    output: Output::Drm(presenter),
                    lines,
                });
            }
            Err(why) => {
                // A required path that is missing is an error; an automatic
                // one degrades, but says so with the reason the machine gave.
                if choice == Choice::Drm {
                    return Err(format!("drm output requested but unavailable: {why}"));
                }
                lines.push(format!("present=drm-skipped({why})"));
            }
        }
    }

    let Some((sys, dev, mut info)) = fb::discover(root) else {
        let reason = if lines.is_empty() {
            "no framebuffer found".to_owned()
        } else {
            format!("no framebuffer found after {}", lines.join(", "))
        };
        return Err(reason);
    };
    if let Some(f) = format {
        info.format = f;
    }
    let framebuffer =
        Framebuffer::open(&dev, info).map_err(|e| format!("cannot open {}: {e}", dev.display()))?;
    lines.push("present=fbdev".to_owned());
    lines.push(format!("fb={}", framebuffer.info().summary()));
    lines.push(format!("fb.sysfs={}", sys.display()));
    lines.push("target=framebuffer".to_owned());
    Ok(Opened {
        output: Output::Fb(framebuffer),
        lines,
    })
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::canvas::rgb;
    use std::fs;

    fn fixture(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("litric-shell-present-{name}"));
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).expect("fixture dir");
        dir
    }

    fn framebuffer_at(root: &Path) {
        let sys = root.join("sys/class/graphics/fb0");
        fs::create_dir_all(&sys).expect("sysfs fixture");
        fs::create_dir_all(root.join("dev")).expect("dev fixture");
        fs::write(sys.join("virtual_size"), "8,4\n").expect("size");
        fs::write(sys.join("bits_per_pixel"), "32\n").expect("bpp");
        fs::write(sys.join("stride"), "32\n").expect("stride");
        fs::write(sys.join("name"), "fixturefb\n").expect("name");
        fs::write(root.join("dev/fb0"), vec![0u8; 32 * 4]).expect("device");
    }

    fn card_node(root: &Path) {
        fs::create_dir_all(root.join("dev/dri")).expect("dri fixture");
        fs::write(root.join("dev/dri/card0"), b"").expect("card fixture");
    }

    fn stub(dir: &Path, handshake: &str) -> PathBuf {
        let path = dir.join("drmpresent.sh");
        fs::write(
            &path,
            format!(
                "#!/bin/sh\nprintf '{}\\n'\nexec cat > \"$(dirname \"$0\")/out.bin\"\n",
                handshake
            ),
        )
        .expect("stub");
        assert!(std::process::Command::new("chmod")
            .arg("0755")
            .arg(&path)
            .status()
            .expect("chmod")
            .success());
        path
    }

    #[test]
    fn the_flag_accepts_the_three_choices_and_nothing_else() {
        assert_eq!(Choice::parse("auto"), Some(Choice::Auto));
        assert_eq!(Choice::parse("drm"), Some(Choice::Drm));
        assert_eq!(Choice::parse("kms"), Some(Choice::Drm));
        assert_eq!(Choice::parse("fbdev"), Some(Choice::Fbdev));
        assert_eq!(Choice::parse("gpu"), None);
        assert_eq!(Choice::parse(""), None);
    }

    #[test]
    fn a_card_and_an_installed_helper_are_preferred_over_the_framebuffer() {
        let dir = fixture("drm");
        framebuffer_at(&dir);
        card_node(&dir);
        let helper = stub(&dir, "LITRIC-DRM/1 64 48 256 eDP-1");
        let mut opened = open_with(&dir, Choice::Auto, None, &helper).expect("opens DRM");
        assert_eq!(opened.output.kind(), "drm");
        assert_eq!((opened.width(), opened.height()), (64, 48));
        assert_eq!(opened.lines[0], "present=drm");
        assert!(opened.lines.iter().any(|l| l.contains("connector=eDP-1")));
        assert!(
            !opened.lines.iter().any(|l| l.starts_with("present=fbdev")),
            "a fallback line must not appear when nothing fell back"
        );
        // And the surface really presents: one band, one flip.
        let canvas = Canvas::new(64, 48, rgb(0x102030));
        assert_eq!(opened.output.present(&canvas).expect("present"), 48);
        // Drop before cleanup: the helper writes into this directory.
        drop(opened);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn without_a_card_the_framebuffer_is_used_and_the_reason_is_recorded() {
        let dir = fixture("nocard");
        framebuffer_at(&dir);
        let helper = dir.join("absent-drmpresent");
        let mut opened = open_with(&dir, Choice::Auto, None, &helper).expect("falls back to fb");
        assert_eq!(opened.output.kind(), "fbdev");
        assert_eq!(
            opened.lines[0],
            "present=drm-skipped(no card node under /dev/dri)"
        );
        assert_eq!(opened.lines[1], "present=fbdev");
        let canvas = Canvas::new(8, 4, rgb(0xFFFFFF));
        assert_eq!(
            opened
                .output
                .present_damage(&canvas, &[Rect::new(0.0, 1.0, 8.0, 2.0)])
                .expect("present"),
            2
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn requiring_drm_on_a_machine_without_it_is_an_error_naming_the_cause() {
        let dir = fixture("required");
        framebuffer_at(&dir);
        let helper = dir.join("absent-drmpresent");
        let err = open_with(&dir, Choice::Drm, None, &helper).expect_err("must refuse");
        assert!(err.contains("no card node"), "{err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_broken_helper_degrades_and_still_presents() {
        let dir = fixture("broken");
        framebuffer_at(&dir);
        card_node(&dir);
        let helper = dir.join("broken.sh");
        fs::write(&helper, "#!/bin/sh\nexit 3\n").expect("script");
        assert!(std::process::Command::new("chmod")
            .arg("0755")
            .arg(&helper)
            .status()
            .expect("chmod")
            .success());
        let opened = open_with(&dir, Choice::Auto, None, &helper).expect("degrades");
        assert_eq!(opened.output.kind(), "fbdev");
        assert!(
            opened.lines[0].starts_with("present=drm-skipped(")
                && opened.lines[0].contains("before publishing a mode"),
            "{}",
            opened.lines[0]
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fbdev_can_be_forced_where_a_card_exists() {
        let dir = fixture("forced");
        framebuffer_at(&dir);
        card_node(&dir);
        let helper = stub(&dir, "LITRIC-DRM/1 64 48 256 eDP-1");
        let opened = open_with(&dir, Choice::Fbdev, None, &helper).expect("opens fb");
        assert_eq!(opened.output.kind(), "fbdev");
        assert!(
            opened.lines.iter().all(|l| !l.starts_with("present=drm")),
            "a forced framebuffer must not start the card"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_machine_with_neither_surface_reports_what_it_looked_for() {
        let dir = fixture("nothing");
        let helper = dir.join("absent-drmpresent");
        let err = open_with(&dir, Choice::Auto, None, &helper).expect_err("must refuse");
        assert!(err.contains("no framebuffer"), "{err}");
        assert!(err.contains("no card node"), "{err}");
        fs::remove_dir_all(&dir).ok();
    }
}
