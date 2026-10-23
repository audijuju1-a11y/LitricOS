//! The `.trashinfo` record: the `Path=` field, and the body around it.
//!
//! # Why this is not inside `trash.rs`
//!
//! The trash directory on disk has **two** writers in this repository — the
//! `litric-filesd` daemon ([`crate::trash::DiskTrash`]) and the shell's own
//! bridge (`litric_shell::fsbridge`). Before this module existed each one
//! formatted the record itself, and the same defect had to be found and fixed
//! twice (BUG-0011), twice differently. One definition, used by both, is the
//! only arrangement in which "the two writers agree" is a property of the code
//! rather than a promise in a comment.
//!
//! It is also deliberately **platform-independent**. `trash.rs` and `ops.rs`
//! are `#[cfg(unix)]`, so on the Windows development host nothing about the
//! trash format ever compiled, let alone ran. The format is string handling
//! and belongs where every host can test it.
//!
//! # What the specification requires (Freedesktop Trash 1.0)
//!
//! > The key "Path" contains the original location of the file/directory, as
//! > either an absolute pathname (starting with the slash character "/") or a
//! > relative pathname (starting with any other character). A relative
//! > pathname is to be from the directory in which the trash directory resides
//! > … it MUST not include a ".." directory.
//!
//! For the home trash the trash directory lives at `<root>/.local/share/Trash`,
//! whose parent is `<root>/.local/share`, so the only legal relative spelling
//! of `<root>/Documents/x` is `../../Documents/x` — which the same clause
//! forbids. **Only the absolute form is available**, which is what
//! [`record`] writes.
//!
//! This is BUG-0009: the previous writers stored `Path=file:///Documents/x`,
//! which starts with `f`, so a conforming reader (KDE, `gio trash`, anything
//! that is not Litric) treated it as a *relative* path and restored the file to
//! `<trash parent>/file:/Documents/x`. Litric's own readers stripped the
//! `file://` and so never noticed.

use std::path::Path;

/// Percent-encode a `.trashinfo` field.
///
/// `/` is left literal, unlike the wire-protocol encoder in `litric-core::pct`:
/// the specification's own examples and GLib's implementation keep the
/// separator, so this is what another tool's `.trashinfo` will look like and
/// what it expects to read back. Everything outside the RFC 3986 unreserved
/// set (`A-Z a-z 0-9 - . _ ~`) becomes `%XX`, so a space, a newline or a byte
/// that is not ASCII cannot break the line-oriented file.
#[must_use]
pub fn pct_encode(text: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(text.len());
    for &b in text.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            out.push('%');
            out.push(char::from(
                HEX.get(usize::from(b >> 4)).copied().unwrap_or(b'0'),
            ));
            out.push(char::from(
                HEX.get(usize::from(b & 0x0F)).copied().unwrap_or(b'0'),
            ));
        }
    }
    out
}

/// Inverse of [`pct_encode`].
///
/// Deliberately tolerant where the wire decoder in `litric-core::pct` is
/// strict: a `.trashinfo` is a file a user or another program may have edited,
/// and one malformed escape must not make a whole trash entry unreadable. A
/// `%` that is not followed by two hexadecimal digits is kept literally.
#[must_use]
pub fn pct_decode(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while let Some(&b) = bytes.get(i) {
        if b == b'%' {
            let hex = field.get(i.saturating_add(1)..i.saturating_add(3));
            if let Some(hex) = hex {
                if let Ok(decoded) = u8::from_str_radix(hex, 16) {
                    out.push(decoded);
                    i = i.saturating_add(3);
                    continue;
                }
            }
        }
        out.push(b);
        i = i.saturating_add(1);
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The whole `[Trash Info]` body for an item trashed from `host_path`.
///
/// `host_path` is the item's **absolute** host location — `<root>/Documents/x`
/// — not a logical path and not a URI. `deleted_iso` is the already-rendered
/// `YYYY-MM-DDThh:mm:ss` local time.
///
/// This is the single place a record is formatted, so both writers produce
/// byte-identical files and [`is_not_a_uri`] cannot be true of one and false of
/// the other.
#[must_use]
pub fn record(host_path: &Path, deleted_iso: &str) -> String {
    format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        pct_encode(&host_path.to_string_lossy()),
        deleted_iso
    )
}

/// Whether a recorded `Path=` value is the pre-BUG-0009 `file://…` URI form.
///
/// Kept as a named predicate because it is the shape a **legacy** entry has,
/// and every reader must still recover those.
#[must_use]
pub fn is_not_a_uri(recorded: &str) -> bool {
    !recorded.starts_with("file://")
}

/// The logical path, relative to `host_root`, that `absolute` names.
///
/// `None` when `absolute` is not inside `host_root`. The check is on a path
/// boundary, not on a string prefix: `/var/home/litric2/x` is **not** under
/// `/var/home/litric`, even though its text begins with the root's.
#[must_use]
pub fn strip_root(host_root: &Path, absolute: &str) -> Option<String> {
    let root_text = host_root.to_string_lossy();
    let root = root_text.trim_end_matches('/');
    let root = if root.is_empty() { "/" } else { root };
    let rest = absolute.strip_prefix(root)?;
    if rest.is_empty() {
        // The root itself: the only thing that can be trashed from there is
        // the root, whose logical location is `/`.
        return Some("/".to_owned());
    }
    if let Some(tail) = rest.strip_prefix('/') {
        return Some(format!("/{tail}"));
    }
    if root == "/" {
        // `root` is the separator itself, so `rest` already lost its leading
        // slash: "/Documents/x" became "Documents/x".
        return Some(format!("/{rest}"));
    }
    // A sibling that merely shares a text prefix with the root.
    None
}

/// Map a decoded `Path=` value back to the logical path it names.
///
/// Three shapes are understood, in this order:
///
/// 1. the current form — an absolute host path under `host_root`, which is what
///    [`record`] writes for this root;
/// 2. the legacy form — `file://<logical>` from a build before BUG-0009, kept so
///    an entry written by an older LitricOS still restores to the right place;
/// 3. anything else — returned **unchanged**, so the entry still lists with the
///    location it actually records instead of a location nobody asked for.
///
/// Case 3 is why [`maps_into_root`] exists. This function used to claim that a
/// verbatim value "fails the caller's `LogicalPath::parse`". For `Path=/dev/sda`
/// that is true — it is not rooted, so parsing refuses it — but for a *rooted*
/// path from another system (`Path=/somewhere/else/stray.txt`, a trash directory
/// shared with a host whose root is elsewhere) parsing succeeds, because
/// syntactically it is a perfectly good logical path. A restore then resolved it
/// under this root and wrote the item to a place the entry never named. The
/// syntax cannot tell the two apart; the mapping can, and the caller must ask.
#[must_use]
pub fn logical_location(host_root: &Path, recorded: &str) -> String {
    if let Some(logical) = strip_root(host_root, recorded) {
        return logical;
    }
    if let Some(legacy) = recorded.strip_prefix("file://") {
        return legacy.to_owned();
    }
    recorded.to_owned()
}

/// Whether a recorded `Path=` names a location this root can actually own.
///
/// False means the entry came from somewhere else and there is no honest
/// destination for it here: `logical_location` returns it verbatim, and a
/// restore of it has to be refused rather than guessed. Callers that restore
/// must consult this; callers that only list can ignore it.
#[must_use]
pub fn maps_into_root(host_root: &Path, recorded: &str) -> bool {
    strip_root(host_root, recorded).is_some() || recorded.starts_with("file://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    // ---- the writer ------------------------------------------------------

    #[test]
    fn the_record_carries_the_absolute_location_and_the_date() {
        let rec = record(
            Path::new("/var/home/litric/Documents/report.odt"),
            "2026-10-15T09:30:00",
        );
        assert_eq!(
            rec,
            "[Trash Info]\nPath=/var/home/litric/Documents/report.odt\n\
             DeletionDate=2026-10-15T09:30:00\n"
        );
    }

    #[test]
    fn the_record_never_writes_a_uri() {
        // The regression guard for BUG-0009. The old writers built
        // `file://<logical>` and percent-encoded that, so the stored value
        // began `file%3A%2F%2F%2F` — a *relative* pathname to every conforming
        // reader, which then restored the item under `<trash parent>/file:/…`.
        let rec = record(Path::new("/Documents/x"), "2026-10-15T09:30:00");
        let path_line = rec.lines().nth(1).expect("a Path line");
        let value = path_line.strip_prefix("Path=").expect("Path= prefix");
        assert!(
            is_not_a_uri(value),
            "Path= must be the location, not a URI: {value:?}"
        );
        assert!(
            !value.contains("file"),
            "no `file://` scheme, literal or encoded: {value:?}"
        );
        assert!(value.starts_with('/'), "an absolute pathname: {value:?}");
    }

    #[test]
    fn spaces_and_non_ascii_are_percent_encoded_and_round_trip() {
        let host = Path::new("/var/home/litric/My Documents/ไฟล์ ใหม่ (2).txt");
        let rec = record(host, "2026-10-15T09:30:00");
        let value = rec
            .lines()
            .nth(1)
            .and_then(|l| l.strip_prefix("Path="))
            .expect("Path=");
        assert!(
            value.is_ascii(),
            "the file stays a line protocol: {value:?}"
        );
        assert!(
            value.contains("/My%20Documents/"),
            "spaces escaped: {value:?}"
        );
        assert_eq!(
            pct_decode(value),
            "/var/home/litric/My Documents/ไฟล์ ใหม่ (2).txt"
        );
    }

    // ---- the reader ------------------------------------------------------

    #[test]
    fn an_absolute_location_under_the_root_maps_back_to_a_logical_path() {
        let rec = record(
            Path::new("/var/home/litric/Documents/x.txt"),
            "2026-10-15T09:30:00",
        );
        let value = rec.lines().nth(1).expect("line");
        let decoded = pct_decode(value.strip_prefix("Path=").expect("Path="));
        assert_eq!(
            logical_location(&root("/var/home/litric"), &decoded),
            "/Documents/x.txt"
        );
    }

    #[test]
    fn a_legacy_uri_entry_still_restores_to_the_right_place() {
        // A `.trashinfo` written by a build before the fix. Both readers must
        // keep recovering these or the change strands the user's old trash.
        for root_text in ["/var/home/litric", "/"] {
            assert_eq!(
                logical_location(&root(root_text), "file:///Documents/tax/2026.ods"),
                "/Documents/tax/2026.ods",
                "root {root_text}"
            );
        }
    }

    #[test]
    fn a_legacy_uri_entry_that_is_still_percent_encoded_is_decoded_first() {
        // The legacy writer percent-encoded the whole URI, so `file://` itself
        // arrives as `file%3A%2F%2F` and only `pct_decode` turns it back.
        let stored = "file%3A%2F%2F%2FDocuments%2Fmemo.txt";
        let decoded = pct_decode(stored);
        assert_eq!(decoded, "file:///Documents/memo.txt");
        assert_eq!(
            logical_location(&root("/var/home/litric"), &decoded),
            "/Documents/memo.txt"
        );
    }

    #[test]
    fn the_root_itself_is_the_logical_root() {
        assert_eq!(
            logical_location(&root("/var/home/litric"), "/var/home/litric"),
            "/"
        );
        assert_eq!(logical_location(&root("/"), "/"), "/");
    }

    #[test]
    fn a_host_root_of_slash_keeps_the_leading_separator() {
        // On a booted image the shell may be rooted at `/`, where the root text
        // is the separator and the remainder has lost its leading slash.
        assert_eq!(logical_location(&root("/"), "/Documents/x"), "/Documents/x");
        assert_eq!(strip_root(&root("/"), "/a/b"), Some("/a/b".to_owned()));
    }

    #[test]
    fn a_trailing_slash_on_the_root_does_not_change_the_answer() {
        for r in ["/var/home/litric", "/var/home/litric/"] {
            assert_eq!(
                logical_location(&root(r), "/var/home/litric/Documents/x"),
                "/Documents/x",
                "root {r:?}"
            );
        }
    }

    #[test]
    fn a_sibling_that_shares_the_roots_text_prefix_is_not_inside_it() {
        // `/var/home/litric2/x` starts with `/var/home/litric`, but a plain
        // string prefix check would call it "inside the root" and hand the
        // caller a logical path `/2/x` — a restore into the wrong directory.
        assert_eq!(
            strip_root(&root("/var/home/litric"), "/var/home/litric2/x"),
            None
        );
        assert_eq!(
            logical_location(&root("/var/home/litric"), "/var/home/litric2/x"),
            "/var/home/litric2/x",
            "unknown values are left for the parser to refuse"
        );
    }

    #[test]
    fn an_unrecognised_value_is_returned_unchanged_so_it_can_be_refused() {
        // Not absolute, not a URI, not under the root. The caller parses it as
        // a logical path, which fails, which is the honest outcome: no restore
        // rather than a restore to a guessed location.
        assert_eq!(logical_location(&root("/h"), "Documents/x"), "Documents/x");
        assert_eq!(logical_location(&root("/h"), ""), "");
        assert!(!is_not_a_uri("file:///x"));
        assert!(is_not_a_uri("/Documents/x"));
    }

    #[test]
    fn encode_and_decode_are_inverses_for_awkward_names() {
        for name in [
            "/Documents/a b.txt",
            "/Documents/a#b?c%d.txt",
            "/Documents/ไฟล์.txt",
            "/Documents/new\nline.txt",
            "/Documents/tilde~dash-dot_dot",
        ] {
            assert_eq!(pct_decode(&pct_encode(name)), name, "{name:?}");
        }
    }
}
