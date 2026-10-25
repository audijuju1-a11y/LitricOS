//! Unit tests for the parts of the compositor that do not need a client.
//!
//! The protocol behaviour is covered by `tests/client.rs`, against a real
//! foreign client. What is left here is the arithmetic that protects the
//! desktop from a bad buffer, and the modifier mapping that decides what a
//! client thinks is held down.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{modifier_mask, opaque, read_frame, Format, Frame, Pool};

/// The four bytes at `at`, as the slice an assertion can compare against.
fn quad(pixels: &[u8], at: usize) -> Option<&[u8]> {
    pixels.get(at..at.saturating_add(4))
}

fn scratch_file(name: &str, bytes: &[u8]) -> (PathBuf, std::fs::File) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("litric-wl-unit-{name}-{nanos}"));
    // Read *and* write: the pool is read back with `pread`, and a write-only
    // descriptor cannot be.
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&path)
        .expect("file");
    file.write_all(bytes).expect("write");
    (path, file)
}

#[test]
fn shift_control_alt_and_super_are_the_xkb_bits_the_keymap_declares() {
    assert_eq!(modifier_mask(&[]), 0);
    assert_eq!(modifier_mask(&[42]), 1, "LeftShift is Shift");
    assert_eq!(modifier_mask(&[54]), 1, "RightShift is Shift");
    assert_eq!(modifier_mask(&[29]), 4, "LeftCtrl is Control");
    assert_eq!(modifier_mask(&[56]), 8, "LeftAlt is Mod1");
    assert_eq!(modifier_mask(&[125]), 64, "LeftMeta is Mod4");
    assert_eq!(
        modifier_mask(&[29, 42]),
        5,
        "Ctrl+Shift is both bits, not one"
    );
    assert_eq!(modifier_mask(&[30]), 0, "'a' is not a modifier");
}

#[test]
fn a_frame_is_read_row_by_row_through_a_stride() {
    // Two rows of two pixels, with four bytes of padding per row. The padding
    // must not reach the desktop: a compositor that copies `stride * height`
    // bytes hands the scene a sheared image.
    let mut bytes = Vec::new();
    for row in 0..2u8 {
        for px in 0..2u8 {
            bytes.extend_from_slice(&[px, row, 0x7F, 0xFF]);
        }
        bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    }
    let (path, file) = scratch_file("stride", &bytes);
    let pool = Pool::new(file, u64::try_from(bytes.len()).expect("len"));
    let frame = read_frame(&pool, 0, 2, 2, 12, Format::Argb8888, 1).expect("readable");
    assert_eq!(frame.width, 2);
    assert_eq!(frame.height, 2);
    assert_eq!(frame.pixels.len(), 16);
    assert_eq!(quad(&frame.pixels, 0), Some(&[0u8, 0, 0x7F, 0xFF][..]));
    assert_eq!(quad(&frame.pixels, 4), Some(&[1u8, 0, 0x7F, 0xFF][..]));
    // The second row starts after the padding, not at byte 8.
    assert_eq!(quad(&frame.pixels, 8), Some(&[0u8, 1, 0x7F, 0xFF][..]));
    assert_eq!(quad(&frame.pixels, 12), Some(&[1u8, 1, 0x7F, 0xFF][..]));
    let _ = std::fs::remove_file(path);
}

#[test]
fn an_offset_into_the_pool_is_honoured() {
    let mut bytes = vec![0u8; 8];
    bytes.extend_from_slice(&[9, 8, 7, 6]);
    let (path, file) = scratch_file("offset", &bytes);
    let pool = Pool::new(file, u64::try_from(bytes.len()).expect("len"));
    let frame = read_frame(&pool, 8, 1, 1, 4, Format::Argb8888, 1).expect("readable");
    assert_eq!(quad(&frame.pixels, 0), Some(&[9u8, 8, 7, 6][..]));
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_buffer_that_names_memory_outside_the_pool_is_refused() {
    let bytes = vec![0u8; 16];
    let (path, file) = scratch_file("short", &bytes);
    let pool = Pool::new(file, u64::try_from(bytes.len()).expect("len"));
    // 4x4 at 16 bytes a row needs 64 bytes; the pool has 16.
    let err = read_frame(&pool, 0, 4, 4, 16, Format::Argb8888, 1);
    assert!(err.is_err(), "a short pool must not be read past its end");
    let message = err.err().unwrap_or_default();
    assert!(message.contains("pool"), "{message}");
    // A stride narrower than a row is arithmetically impossible to honour.
    assert!(read_frame(&pool, 0, 4, 1, 8, Format::Argb8888, 1).is_err());
    // Zero and oversized dimensions are refused before any arithmetic.
    assert!(read_frame(&pool, 0, 0, 4, 16, Format::Argb8888, 1).is_err());
    assert!(read_frame(&pool, 0, 9000, 1, 36_000, Format::Argb8888, 1).is_err());
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_scaled_buffer_reports_its_logical_size() {
    let frame = Frame {
        width: 640,
        height: 480,
        format: Format::Argb8888,
        scale: 2,
        pixels: Arc::from(vec![0u8; 640 * 480 * 4]),
    };
    assert_eq!(frame.logical_width(), 320);
    assert_eq!(frame.logical_height(), 240);

    let unscaled = Frame {
        scale: 0,
        ..frame.clone()
    };
    assert_eq!(unscaled.logical_width(), 640, "a zero scale means one");
}

#[test]
fn an_xrgb_frame_becomes_opaque_without_touching_its_colour() {
    let mut frame = Frame {
        width: 1,
        height: 1,
        format: Format::Xrgb8888,
        scale: 1,
        pixels: Arc::from(vec![10u8, 20, 30, 0]),
    };
    opaque(&mut frame);
    assert_eq!(&*frame.pixels, &[10, 20, 30, 0xFF]);
    // An ARGB frame keeps the alpha the client asked for.
    let mut argb = Frame {
        format: Format::Argb8888,
        ..frame
    };
    opaque(&mut argb);
    assert_eq!(argb.pixels.get(3), Some(&0xFF));
}
