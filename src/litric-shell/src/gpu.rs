//! Discovery of the DRM device behind the KMS output path.
//!
//! The shell can present through a card without knowing what the card is.
//! That is enough to put pixels on screen, but not enough to tell a user (or a
//! proof) whether the machine is using a real GPU, a virtual display adapter,
//! or a compatibility framebuffer. This module reads the kernel's own DRM
//! sysfs nodes and names the card, its driver, its PCI identifiers and any
//! render node that is available.
//!
//! It is deliberately descriptive. Discovering a render node does not mean the
//! shell rasterises on the GPU: the shell's canvas remains software (ADR-0026).
//! The information is here so the presenter and the browser can make an
//! explicit choice from the hardware that is actually present, rather than
//! assuming every machine has the same device.

use std::path::{Path, PathBuf};

/// One DRM card and the useful facts the kernel exposes for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInfo {
    /// Card node, e.g. `/dev/dri/card0`.
    pub card: PathBuf,
    /// Driver directory name, e.g. `bochs-drm` or `i915`.
    pub driver: String,
    /// PCI vendor id as sysfs spells it, or `unknown` when the card is not PCI.
    pub vendor: String,
    /// PCI device id as sysfs spells it, or `unknown`.
    pub device: String,
    /// First render node the card exposes, when it has one.
    pub render: Option<PathBuf>,
}

impl GpuInfo {
    /// One line suitable for the shell status file.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "card={} driver={} vendor={} device={} render={}",
            self.card.display(),
            self.driver,
            self.vendor,
            self.device,
            self.render
                .as_ref()
                .map_or_else(|| "none".to_owned(), |path| path.display().to_string())
        )
    }
}

fn read_trimmed(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn driver_name(path: &Path) -> String {
    std::fs::read_link(path)
        .ok()
        .and_then(|target| {
            target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Find the first `cardN` under `root` and describe it.
///
/// `root` is normally `/`. It is a parameter so the discovery rules can be
/// tested against a directory tree without a real DRM device. `card0` is tried
/// first because that is the kernel's usual primary card; a machine whose panel
/// is driven by card1..card3 is still described.
#[must_use]
pub fn discover(root: &Path) -> Option<GpuInfo> {
    for card_number in 0..4u32 {
        let card = root.join(format!("dev/dri/card{card_number}"));
        if !card.exists() {
            continue;
        }
        let device = root
            .join("sys/class/drm")
            .join(format!("card{card_number}"))
            .join("device");
        let render = (128..132u32)
            .map(|number| root.join(format!("dev/dri/renderD{number}")))
            .find(|path| path.exists());
        return Some(GpuInfo {
            card,
            driver: driver_name(&device.join("driver")),
            vendor: read_trimmed(&device.join("vendor")).unwrap_or_else(|| "unknown".to_owned()),
            device: read_trimmed(&device.join("device")).unwrap_or_else(|| "unknown".to_owned()),
            render,
        });
    }
    None
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("litric-shell-gpu-{name}"));
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).expect("fixture dir");
        dir
    }

    fn card_tree(root: &Path, number: u32, driver: &str, render: bool) {
        let device = root
            .join("sys/class/drm")
            .join(format!("card{number}"))
            .join("device");
        fs::create_dir_all(&device).expect("device dir");
        fs::create_dir_all(root.join("dev/dri")).expect("dri dir");
        std::os::unix::fs::symlink(
            format!("../../../bus/pci/drivers/{driver}"),
            device.join("driver"),
        )
        .expect("driver symlink");
        fs::write(device.join("vendor"), "0x1af4\n").expect("vendor");
        fs::write(device.join("device"), "0x1050\n").expect("device id");
        fs::write(root.join(format!("dev/dri/card{number}")), b"").expect("card node");
        if render {
            fs::write(root.join("dev/dri/renderD128"), b"").expect("render node");
        }
    }

    #[test]
    fn the_card_is_named_with_its_driver_and_render_node() {
        let root = fixture("card");
        card_tree(&root, 0, "virtio_gpu", true);
        let gpu = discover(&root).expect("card");
        assert_eq!(gpu.card, root.join("dev/dri/card0"));
        assert_eq!(gpu.driver, "virtio_gpu");
        assert_eq!(gpu.vendor, "0x1af4");
        assert_eq!(gpu.device, "0x1050");
        assert_eq!(gpu.render, Some(root.join("dev/dri/renderD128")));
        assert!(gpu.summary().contains("driver=virtio_gpu"));
        assert!(gpu.summary().contains("render="));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_card_without_sysfs_details_is_still_reported() {
        let root = fixture("bare");
        fs::create_dir_all(root.join("dev/dri")).expect("dri dir");
        fs::write(root.join("dev/dri/card0"), b"").expect("card node");
        let gpu = discover(&root).expect("card");
        assert_eq!(gpu.driver, "unknown");
        assert_eq!(gpu.vendor, "unknown");
        assert_eq!(gpu.render, None);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn there_is_no_gpu_without_a_card_node() {
        let root = fixture("none");
        assert!(discover(&root).is_none());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_later_card_is_found_when_card_zero_is_absent() {
        let root = fixture("later");
        card_tree(&root, 2, "i915", false);
        let gpu = discover(&root).expect("card2");
        assert_eq!(gpu.card, root.join("dev/dri/card2"));
        assert_eq!(gpu.driver, "i915");
        assert_eq!(gpu.render, None);
        fs::remove_dir_all(&root).ok();
    }
}
