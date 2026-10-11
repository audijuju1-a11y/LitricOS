//! Installer family detection and unattended-install flags.
//!
//! When a user double-clicks `setup.exe` in LitricOS, the goal is that a
//! progress bar appears and an application shows up in the launcher. That is
//! only possible if we know *which* installer we are looking at, because every
//! installer toolkit has its own switch for "do not ask me anything".
//!
//! # Why this is not just a nicety
//!
//! The alternative is showing the user a Windows installer wizard rendered by
//! Wine, which is exactly the experience that makes people say Linux is hard.
//! Detecting the family and driving it unattended is what turns
//! "install a Windows program" into an operating-system feature instead of a
//! hobby project.
//!
//! # Honesty requirement
//!
//! Detection is a heuristic over byte signatures. It can be wrong, and when it
//! is uncertain the correct behaviour is [`InstallerFamily::Unknown`] plus an
//! interactive install, **never** a guessed silent switch. Guessing wrong on a
//! silent switch can mean an installer that writes to the wrong place with no
//! visible feedback.

use core::fmt;

/// A recognised Windows installer toolkit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InstallerFamily {
    /// Nullsoft Scriptable Install System.
    Nsis,
    /// Inno Setup.
    InnoSetup,
    /// InstallShield.
    InstallShield,
    /// A WiX "Burn" bundle.
    WixBurn,
    /// A Microsoft Installer database (`.msi`), driven by `msiexec`.
    Msi,
    /// A 7-Zip self-extracting archive.
    SevenZipSfx,
    /// A WinRAR self-extracting archive.
    WinRarSfx,
    /// A Squirrel.Windows installer, typical of Electron applications.
    Squirrel,
    /// A PE image that is not an installer: a portable application.
    NotAnInstaller,
    /// Looks like an installer, family unrecognised.
    Unknown,
}

impl InstallerFamily {
    /// Arguments that make this installer run without user interaction.
    ///
    /// Returns `None` when we must not guess. A `None` here means the UI shows
    /// the installer window and lets the human drive it, which is a worse
    /// experience but a correct one.
    #[must_use]
    pub fn silent_args(self) -> Option<&'static [&'static str]> {
        match self {
            // /S is the documented NSIS switch. /D=, if used, must come last and
            // must be unquoted -- the install planner appends it, not us.
            Self::Nsis => Some(&["/S"]),
            // /SP- suppresses the "This will install..." prompt that otherwise
            // blocks even in silent mode.
            Self::InnoSetup => Some(&["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/SP-"]),
            // InstallShield wraps MSI; /v passes through to msiexec.
            Self::InstallShield => Some(&["/s", "/v\"/qn /norestart\""]),
            Self::WixBurn => Some(&["/quiet", "/norestart"]),
            // MSI is not self-executing; the planner invokes msiexec instead.
            Self::Msi => Some(&["/qn", "/norestart"]),
            Self::SevenZipSfx => Some(&["-y"]),
            Self::WinRarSfx => Some(&["/S"]),
            Self::Squirrel => Some(&["--silent"]),
            Self::NotAnInstaller | Self::Unknown => None,
        }
    }

    /// Whether this family needs an external driver program rather than being
    /// executed directly.
    #[must_use]
    pub const fn needs_msiexec(self) -> bool {
        matches!(self, Self::Msi)
    }

    /// Stable token for logs and telemetry-free diagnostics.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Nsis => "nsis",
            Self::InnoSetup => "inno",
            Self::InstallShield => "installshield",
            Self::WixBurn => "wix-burn",
            Self::Msi => "msi",
            Self::SevenZipSfx => "7z-sfx",
            Self::WinRarSfx => "winrar-sfx",
            Self::Squirrel => "squirrel",
            Self::NotAnInstaller => "portable",
            Self::Unknown => "unknown",
        }
    }

    /// A plain-language description for the install dialog.
    ///
    /// The user is about to let untrusted software write to their machine. They
    /// should be told what it is in words, not in a toolkit name they have no
    /// reason to recognise.
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Nsis | Self::InnoSetup | Self::InstallShield | Self::WixBurn => {
                "a Windows installer program"
            }
            Self::Msi => "a Windows Installer package",
            Self::SevenZipSfx | Self::WinRarSfx => "a self-extracting archive",
            Self::Squirrel => "a Windows installer for a web-technology application",
            Self::NotAnInstaller => "a Windows program that runs without installing",
            Self::Unknown => "a Windows program that appears to be an installer",
        }
    }
}

impl fmt::Display for InstallerFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// Byte signatures, most specific first.
///
/// Order matters: an InstallShield bundle can embed an MSI, and a WiX Burn
/// bundle contains a `.wixburn` section while also looking generically like a
/// bundle. First match wins, so the more specific marker is listed earlier.
const SIGNATURES: &[(&[u8], InstallerFamily)] = &[
    (b"Nullsoft.NSIS.exehead", InstallerFamily::Nsis),
    (b"NullsoftInst", InstallerFamily::Nsis),
    (b"Inno Setup Setup Data", InstallerFamily::InnoSetup),
    (b"JR.Inno.Setup", InstallerFamily::InnoSetup),
    (b".wixburn", InstallerFamily::WixBurn),
    (b"WixBundleOriginalSource", InstallerFamily::WixBurn),
    (b"ISSetupPrerequisites", InstallerFamily::InstallShield),
    (b"InstallShield", InstallerFamily::InstallShield),
    (b"Squirrel", InstallerFamily::Squirrel),
    (b"7-Zip SFX", InstallerFamily::SevenZipSfx),
    (b"SfxSetup", InstallerFamily::SevenZipSfx),
    (b"WinRAR SFX", InstallerFamily::WinRarSfx),
];

/// Weak hints that a PE is *an* installer even when the family is unrecognised.
const GENERIC_INSTALLER_HINTS: &[&[u8]] = &[
    b"setup.exe",
    b"Setup.exe",
    b"InstallLocation",
    b"UninstallString",
    b"Installer",
];

/// Identify the installer family from the leading bytes of a file.
///
/// `head` should be a prefix of the file. 1 MiB is a good size: installer
/// signatures live in the stub near the start, and reading more costs I/O for
/// nothing. `is_pe` comes from [`crate::pe::inspect`] succeeding.
///
/// The function never allocates and never reads outside `head`.
#[must_use]
pub fn detect(head: &[u8], is_pe: bool) -> InstallerFamily {
    if crate::pe::is_msi(head) {
        return InstallerFamily::Msi;
    }
    if !is_pe {
        return InstallerFamily::Unknown;
    }

    for (needle, family) in SIGNATURES {
        if contains(head, needle) {
            return *family;
        }
    }

    if GENERIC_INSTALLER_HINTS.iter().any(|h| contains(head, h)) {
        return InstallerFamily::Unknown;
    }

    InstallerFamily::NotAnInstaller
}

/// Substring search over bytes.
///
/// Written out rather than pulled from a crate because this module is on the
/// zero-dependency list, and because a naive scan over a 1 MiB prefix is far
/// below the cost of the read that produced it.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    let last = haystack.len().saturating_sub(needle.len());
    (0..=last).any(|i| haystack.get(i..i.saturating_add(needle.len())) == Some(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf_with(marker: &[u8]) -> Vec<u8> {
        let mut v = vec![0u8; 512];
        v.extend_from_slice(marker);
        v.extend_from_slice(&[0u8; 512]);
        v
    }

    #[test]
    fn detects_each_known_family() {
        let cases: &[(&[u8], InstallerFamily)] = &[
            (b"Nullsoft.NSIS.exehead", InstallerFamily::Nsis),
            (b"Inno Setup Setup Data (5.5.9)", InstallerFamily::InnoSetup),
            (b"ISSetupPrerequisites", InstallerFamily::InstallShield),
            (b".wixburn", InstallerFamily::WixBurn),
            (b"7-Zip SFX 1.6", InstallerFamily::SevenZipSfx),
            (b"WinRAR SFX", InstallerFamily::WinRarSfx),
            (b"Squirrel", InstallerFamily::Squirrel),
        ];
        for (marker, expected) in cases {
            assert_eq!(
                detect(&buf_with(marker), true),
                *expected,
                "marker {marker:?}"
            );
        }
    }

    #[test]
    fn msi_is_detected_from_ole_magic_even_without_pe() {
        let mut v = vec![0xd0u8, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];
        v.extend_from_slice(&[0u8; 64]);
        assert_eq!(detect(&v, false), InstallerFamily::Msi);
        assert!(InstallerFamily::Msi.needs_msiexec());
    }

    #[test]
    fn plain_program_is_not_an_installer() {
        let v = buf_with(b"just an ordinary program with no markers");
        assert_eq!(detect(&v, true), InstallerFamily::NotAnInstaller);
        assert_eq!(InstallerFamily::NotAnInstaller.silent_args(), None);
    }

    #[test]
    fn generic_hints_yield_unknown_not_a_guess() {
        // The critical behaviour: something that smells like an installer but
        // is not recognised must NOT be handed a guessed silent switch.
        let v = buf_with(b"UninstallString");
        let family = detect(&v, true);
        assert_eq!(family, InstallerFamily::Unknown);
        assert_eq!(
            family.silent_args(),
            None,
            "an unrecognised installer must never be driven with a guessed switch"
        );
    }

    #[test]
    fn every_recognised_family_has_silent_args() {
        // Inverse property: if we claim to recognise a family, we must know how
        // to drive it. A recognised family with no switch is a detection that
        // buys nothing.
        for f in [
            InstallerFamily::Nsis,
            InstallerFamily::InnoSetup,
            InstallerFamily::InstallShield,
            InstallerFamily::WixBurn,
            InstallerFamily::Msi,
            InstallerFamily::SevenZipSfx,
            InstallerFamily::WinRarSfx,
            InstallerFamily::Squirrel,
        ] {
            assert!(
                f.silent_args().is_some_and(|a| !a.is_empty()),
                "{f} has no silent args"
            );
            assert!(!f.describe().is_empty());
        }
    }

    #[test]
    fn specific_signature_wins_over_generic() {
        // An InstallShield bundle that also mentions "Installer" must be
        // detected as InstallShield, not as Unknown.
        let mut v = buf_with(b"ISSetupPrerequisites");
        v.extend_from_slice(b"Installer");
        assert_eq!(detect(&v, true), InstallerFamily::InstallShield);
    }

    #[test]
    fn detection_is_bounded_and_panic_free() {
        assert_eq!(detect(&[], false), InstallerFamily::Unknown);
        assert_eq!(detect(&[], true), InstallerFamily::NotAnInstaller);
        assert_eq!(detect(&[0u8; 1], true), InstallerFamily::NotAnInstaller);
        // A signature split across the end of the read window must not match
        // and must not read past the buffer.
        assert_eq!(
            detect(b"Nullsoft.NSIS.exehe", true),
            InstallerFamily::NotAnInstaller
        );
    }

    #[test]
    fn contains_handles_edges() {
        assert!(contains(b"abcdef", b"abc"));
        assert!(contains(b"abcdef", b"def"));
        assert!(contains(b"abcdef", b"cde"));
        assert!(!contains(b"abcdef", b"xyz"));
        assert!(!contains(b"abc", b"abcdef"));
        assert!(!contains(b"", b"a"));
        assert!(!contains(b"abc", b""));
    }
}
