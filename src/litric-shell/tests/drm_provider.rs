//! Exercise the real C2 provider binary, not a shell stand-in.
//!
//! The Rust protocol tests in `src/drm.rs` use a tiny script that prints a
//! handshake and copies stdin to a file. That is the right fixture for the
//! wire protocol, but it does not notice when `drmpresent.c` stops compiling,
//! starts hanging, or loses its "a regular file is not a card" refusal. This
//! test builds the actual helper and runs its `--check` path against a regular
//! file.
//!
//! This host has `CONFIG_DRM` unset and no `/dev/dri`, so the success path
//! (connector, mode, SETCRTC, page flip) cannot run here. The test asserts the
//! refusal and its exact reason instead of inventing a card. When a real card
//! is present on another host, the helper's `--check` is still the first thing
//! to read; the evidence is in `docs/adr/0026-kms-output-path.md`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litric-shell-drm-helper-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_real_kms_helper_refuses_a_non_card_and_exits_cleanly() {
    let dir = scratch();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/drmpresent.c");
    let binary = dir.join("litric-drmpresent");
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());

    let compiled = Command::new(&compiler)
        .args(["-std=c17", "-O2", "-Wall", "-Wextra", "-Werror"])
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output();
    let Ok(compiled) = compiled else {
        eprintln!(
            "SKIP: no C compiler ({}) is available to build the real KMS helper",
            compiler.to_string_lossy()
        );
        cleanup(&dir);
        return;
    };
    if !compiled.status.success() {
        let stderr = String::from_utf8_lossy(&compiled.stderr);
        // A host without the kernel DRM uapi headers cannot build the image
        // helper either. Say so loudly instead of treating that as a provider
        // success or a Rust failure.
        if stderr.contains("drm/drm.h") || stderr.contains("drm_mode.h") {
            eprintln!("SKIP: kernel DRM headers are unavailable: {stderr}");
            cleanup(&dir);
            return;
        }
        panic!("the real KMS helper did not compile: {stderr}");
    }

    let not_a_card = dir.join("not-a-card");
    std::fs::write(&not_a_card, b"this is a file, not a DRM node").expect("fixture file");
    let checked = Command::new(&binary)
        .arg("--check")
        .arg(&not_a_card)
        .output()
        .expect("run the real helper");

    assert!(
        !checked.status.success(),
        "a regular file must not be accepted as a KMS card"
    );
    assert_eq!(
        checked.status.code(),
        Some(1),
        "the helper exits 1 for a card it cannot query"
    );
    let stderr = String::from_utf8_lossy(&checked.stderr);
    assert!(
        stderr.contains("get resources: Inappropriate ioctl for device"),
        "the real helper must name the ioctl failure, got: {stderr}"
    );
    assert!(
        !stderr.contains("SETCRTC") && !stderr.contains("become DRM master"),
        "refusing a file must happen before mode setting: {stderr}"
    );

    cleanup(&dir);
}
