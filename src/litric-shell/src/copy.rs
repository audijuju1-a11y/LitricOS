//! The words a person reads.
//!
//! One module, because a label written in two places drifts. The network row,
//! the process table and the storage list used to spell the machine's own words
//! -- `wlan0`, `erofs`, `/dev/input/...` -- into surfaces a user reads. The rule
//! is simple: the interface says what a thing *means*, never what the kernel
//! calls it. The vocabulary lives here, next to the list the guard test forbids.

/// Tokens that belong to the machine, never to a row in the interface.
///
/// The guard journey in `e2e` renders the real frames and fails if any of these
/// reaches `Canvas::text_runs`. This is the single list: a token added here is
/// forbidden everywhere at once, which is what stops the rule drifting back out
/// into the code.
pub const BANNED_TOKENS: &[&str] = &[
    "erofs", "ext4", "btrfs", "tmpfs", "squashfs", "/dev/", "/proc/", "wlan0", "eth0", "enp0s2",
    "hci0", "card0",
];

/// A link's state in the words a person uses.
#[must_use]
pub fn connection(state: &str) -> &'static str {
    match state {
        "up" => "Connected",
        "down" => "Not connected",
        _ => "Connecting",
    }
}

/// What kind of link an interface is, without its kernel name.
#[must_use]
pub fn interface_kind(name: &str, wireless: bool) -> &'static str {
    if name == "lo" {
        "This device"
    } else if wireless {
        "Wi-Fi"
    } else {
        "Wired network"
    }
}

/// Whether a volume can be written, in words.
#[must_use]
pub fn volume_access(read_only: bool) -> &'static str {
    if read_only {
        "Read-only"
    } else {
        "Read and write"
    }
}

/// Why a protected volume cannot be written, in a sentence with no token in it.
#[must_use]
pub fn readonly_note(read_only: bool) -> Option<&'static str> {
    read_only.then_some(
        "This is the protected system storage: nothing can change it while Litric is running, \
         and every block is checked as it is read.",
    )
}

/// The name of a running program as the process table shows it.
///
/// The kernel's own names (`litric-shell`, `litric-filesd`) are accurate and
/// unreadable. These are the names of the things a person opened; anything the
/// table does not recognise keeps the name it was given, because inventing a
/// friendly name for an unknown process would hide what is running.
#[must_use]
pub fn process_label(comm: &str) -> String {
    match comm {
        "litric-shell" => "Desktop",
        "litric-compositor" => "Display",
        "litric-sysd" => "System services",
        "litric-filesd" => "Files",
        "litric-browserd" => "Browser",
        "litric-notifyd" => "Notifications",
        "litric-defender" => "Security",
        _ => comm,
    }
    .to_owned()
}

/// The name a person reads for a place or an item: the last path segment,
/// with the home root named the way the Files pane names it.
#[must_use]
pub fn place_name(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return "This Litric".to_owned();
    }
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .to_owned()
}

/// True when `text` carries no banned token.
#[must_use]
pub fn is_plain(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    !BANNED_TOKENS.iter().any(|t| lower.contains(t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_connection_states_are_the_three_plain_ones() {
        assert_eq!(connection("up"), "Connected");
        assert_eq!(connection("down"), "Not connected");
        assert_eq!(connection("unknown"), "Connecting");
        for state in ["up", "down", "unknown"] {
            assert!(is_plain(connection(state)));
        }
    }

    #[test]
    fn interface_kinds_do_not_carry_the_kernel_name() {
        assert_eq!(interface_kind("wlan0", true), "Wi-Fi");
        assert_eq!(interface_kind("enp0s2", false), "Wired network");
        assert_eq!(interface_kind("lo", false), "This device");
        for name in ["wlan0", "enp0s2", "lo"] {
            assert!(is_plain(interface_kind(name, false)));
        }
    }

    #[test]
    fn storage_says_what_can_be_done_not_what_the_disk_is_formatted_as() {
        assert_eq!(volume_access(true), "Read-only");
        assert_eq!(volume_access(false), "Read and write");
        let note = readonly_note(true).unwrap_or_default();
        assert!(note.contains("protected system storage"), "{note}");
        assert!(is_plain(note), "no token in the explanation: {note}");
        assert!(readonly_note(false).is_none());
    }

    #[test]
    fn programs_a_person_opened_are_called_what_they_are() {
        assert_eq!(process_label("litric-shell"), "Desktop");
        assert_eq!(process_label("litric-filesd"), "Files");
        assert_eq!(process_label("litric-sysd"), "System services");
        // An unrecognised program keeps its own name: renaming it would hide
        // what is running.
        assert_eq!(process_label("bash"), "bash");
        for comm in ["litric-shell", "litric-sysd", "litric-defender"] {
            assert!(is_plain(&process_label(comm)));
        }
    }

    #[test]
    fn places_are_named_the_way_the_window_names_them() {
        assert_eq!(place_name("/"), "This Litric");
        assert_eq!(place_name("/Documents"), "Documents");
        assert_eq!(place_name("/Documents/Journey"), "Journey");
        assert_eq!(place_name("Journey"), "Journey");
    }

    #[test]
    fn the_guard_list_rejects_every_machine_token() {
        for token in BANNED_TOKENS {
            let drawn = format!("volume {token} mounted");
            assert!(!is_plain(&drawn), "{token} must be rejected");
        }
        assert!(is_plain("Connected over Wi-Fi"));
    }
}
