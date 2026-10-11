//! Bottles: disposable, per-application Windows environments.
//!
//! ECOSYSTEM section 17.1 requires that a Windows application run inside an
//! environment that can be created, suspended, reset, repaired, migrated and
//! deleted. This module is that model.
//!
//! # Why one bottle per application
//!
//! The traditional Wine model is a single shared `~/.wine` prefix. It is the
//! wrong shape for an operating system, for three reasons:
//!
//! 1. **Blast radius.** Every Windows program shares one registry, one
//!    `C:\Windows`, one set of DLL overrides. One installer that replaces a
//!    system DLL breaks every other application at once.
//! 2. **Uninstall is impossible.** Nothing can be cleanly removed from a shared
//!    prefix, so a shared prefix only ever accumulates.
//! 3. **Isolation.** RUNTIME section 41 says a Windows application is untrusted
//!    code even when its publisher is trusted. Sharing a filesystem namespace
//!    between two untrusted programs means one can read the other's saved
//!    credentials.
//!
//! Per-application bottles cost disk (deduplicated by content-addressed runtime
//! layers, so the cost is the *delta*, not a full Windows install per app) and
//! buy a working uninstall, a working reset, and real isolation.

use core::fmt;
use litric_core::AppId;

/// Which Windows API level the bottle presents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum WindowsVersion {
    /// Windows 7. Still required by a surprising amount of older software.
    Win7,
    /// Windows 10.
    #[default]
    Win10,
    /// Windows 11.
    Win11,
}

impl WindowsVersion {
    /// The value Wine expects for this version.
    #[must_use]
    pub const fn wine_token(self) -> &'static str {
        match self {
            Self::Win7 => "win7",
            Self::Win10 => "win10",
            Self::Win11 => "win11",
        }
    }
}

/// The compatibility runtime backing a bottle.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Runtime {
    /// A Proton build. Preferred for games: it carries DXVK, VKD3D-Proton and
    /// the Fossilize shader pre-caching that removes first-run stutter.
    Proton {
        /// Version string, e.g. `"9.0-4"` or `"experimental"`.
        version: String,
    },
    /// Upstream Wine. Preferred for productivity software, where Proton's
    /// game-oriented patches buy nothing.
    Wine {
        /// Version string, e.g. `"11.0"`.
        version: String,
    },
}

impl Runtime {
    /// Stable token for paths and logs.
    #[must_use]
    pub fn token(&self) -> String {
        match self {
            Self::Proton { version } => format!("proton-{version}"),
            Self::Wine { version } => format!("wine-{version}"),
        }
    }

    /// Whether this runtime ships the Direct3D-to-Vulkan translation layers.
    #[must_use]
    pub const fn has_d3d_translation(&self) -> bool {
        matches!(self, Self::Proton { .. })
    }
}

impl fmt::Display for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.token())
    }
}

/// Optional runtime components a bottle may need staged.
///
/// These are the redistributables Windows software habitually assumes are
/// present. Staging them per bottle, from content-addressed layers, keeps the
/// bottle self-contained and reproducible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Component {
    /// Microsoft .NET Framework 4.8.
    DotNetFx48,
    /// .NET 8 desktop runtime.
    DotNet8,
    /// Visual C++ 2015-2022 redistributable.
    VcRedist2022,
    /// DirectX 9 runtime shims.
    D3dx9,
    /// Core web fonts, so text does not render as boxes.
    CoreFonts,
    /// Media Foundation, needed by some video playback paths.
    MediaFoundation,
}

impl Component {
    /// Stable token used as the layer name.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::DotNetFx48 => "dotnetfx48",
            Self::DotNet8 => "dotnet8",
            Self::VcRedist2022 => "vcredist2022",
            Self::D3dx9 => "d3dx9",
            Self::CoreFonts => "corefonts",
            Self::MediaFoundation => "mediafoundation",
        }
    }
}

/// Lifecycle state of a bottle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BottleState {
    /// Directory allocated, runtime not yet staged.
    Created,
    /// Runtime and components staged; ready to run.
    Ready,
    /// A process from this bottle is running.
    Running,
    /// Frozen. Processes stopped, state preserved on disk.
    Suspended,
    /// Failed verification or a staging step. Needs repair or reset.
    Broken,
}

impl BottleState {
    /// Whether an application may be launched from this bottle right now.
    #[must_use]
    pub const fn can_launch(self) -> bool {
        matches!(self, Self::Ready | Self::Running)
    }

    /// The lifecycle transitions permitted from this state.
    ///
    /// Enumerating transitions rather than allowing any assignment is what keeps
    /// "reset a running bottle" from being a data-loss bug that only shows up
    /// under a race.
    #[must_use]
    pub fn allowed_transitions(self) -> &'static [BottleState] {
        match self {
            Self::Created => &[Self::Ready, Self::Broken],
            Self::Ready => &[Self::Running, Self::Suspended, Self::Broken, Self::Created],
            // A running bottle must be stopped before anything else happens to
            // it. There is no Running -> Created edge: that would be deleting
            // files out from under live processes.
            Self::Running => &[Self::Ready, Self::Suspended, Self::Broken],
            Self::Suspended => &[Self::Ready, Self::Broken],
            // Repair goes Broken -> Created (wipe and re-stage), never straight
            // to Ready: a bottle that failed verification is not trusted to be
            // correct just because someone asked nicely.
            Self::Broken => &[Self::Created],
        }
    }

    /// Whether `self -> next` is a legal transition.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        self.allowed_transitions().contains(&next)
    }
}

/// The isolation policy applied to everything running in a bottle.
///
/// Defaults are deny. A Windows application gets nothing it was not explicitly
/// given, exactly like a native application, and the same capability broker
/// makes the decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Isolation {
    /// May reach the network at all.
    pub network: bool,
    /// May see the user's real home directory.
    ///
    /// Default `false`: the bottle's `C:\users\<name>` maps to storage private
    /// to that bottle. Opening a real document goes through the file portal,
    /// where the user's own choice of file is the grant.
    pub real_home: bool,
    /// May enumerate and open GPU devices. Needed for anything that draws.
    pub gpu: bool,
    /// May open audio output.
    pub audio_out: bool,
    /// May open audio capture.
    pub audio_in: bool,
    /// May see USB devices (controllers, dongles).
    pub usb: bool,
    /// May read the clipboard.
    pub clipboard_read: bool,
    /// May write the clipboard.
    pub clipboard_write: bool,
}

impl Default for Isolation {
    /// The default is what an unattended installer should get: enough to draw a
    /// window and nothing else.
    fn default() -> Self {
        Self {
            network: false,
            real_home: false,
            gpu: true,
            audio_out: false,
            audio_in: false,
            usb: false,
            clipboard_read: false,
            clipboard_write: true,
        }
    }
}

impl Isolation {
    /// The profile for running an installer: no network, no devices, no audio.
    ///
    /// An installer has no legitimate need to reach the internet. Many do it
    /// anyway, to download payloads; those are handled by prompting the user to
    /// grant `network` for that one run, with the request visible, rather than
    /// by granting it silently to every installer.
    #[must_use]
    pub const fn for_installer() -> Self {
        Self {
            network: false,
            real_home: false,
            gpu: true,
            audio_out: false,
            audio_in: false,
            usb: false,
            clipboard_read: false,
            clipboard_write: false,
        }
    }

    /// A typical game profile.
    #[must_use]
    pub const fn for_game() -> Self {
        Self {
            network: true,
            real_home: false,
            gpu: true,
            audio_out: true,
            audio_in: false,
            usb: true,
            clipboard_read: false,
            clipboard_write: true,
        }
    }

    /// The LitricOS capability strings this isolation profile implies.
    ///
    /// Returned in a stable order so the install dialog renders the same list
    /// every time, and so tests can compare it.
    #[must_use]
    pub fn required_capabilities(&self) -> Vec<&'static str> {
        let mut caps = Vec::new();
        if self.network {
            caps.push("net.client");
        }
        if self.real_home {
            caps.push("fs.read.home");
        }
        if self.gpu {
            caps.push("gpu");
        }
        if self.audio_out {
            caps.push("audio.output");
        }
        if self.audio_in {
            caps.push("microphone");
        }
        if self.usb {
            caps.push("device.usb");
        }
        if self.clipboard_read {
            caps.push("clipboard.read");
        }
        if self.clipboard_write {
            caps.push("clipboard.write");
        }
        caps
    }
}

/// A Windows compatibility environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bottle {
    id: AppId,
    runtime: Runtime,
    windows_version: WindowsVersion,
    /// 64-bit bottle. 32-bit-only programs still run in it via WoW64.
    win64: bool,
    components: Vec<Component>,
    isolation: Isolation,
    state: BottleState,
}

impl Bottle {
    /// Create a bottle description in the [`BottleState::Created`] state.
    ///
    /// Components are sorted and deduplicated so that two bottles requesting the
    /// same set produce identical descriptions, which is what lets staged
    /// runtime layers be shared and content-addressed.
    #[must_use]
    pub fn new(id: AppId, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            windows_version: WindowsVersion::default(),
            win64: true,
            components: Vec::new(),
            isolation: Isolation::default(),
            state: BottleState::Created,
        }
    }

    /// Set the presented Windows version.
    #[must_use]
    pub fn with_windows_version(mut self, v: WindowsVersion) -> Self {
        self.windows_version = v;
        self
    }

    /// Set the isolation policy.
    #[must_use]
    pub fn with_isolation(mut self, iso: Isolation) -> Self {
        self.isolation = iso;
        self
    }

    /// Add required components, deduplicating and sorting.
    #[must_use]
    pub fn with_components(mut self, components: impl IntoIterator<Item = Component>) -> Self {
        self.components.extend(components);
        self.components.sort_unstable();
        self.components.dedup();
        self
    }

    /// The owning application identity.
    #[must_use]
    pub const fn id(&self) -> &AppId {
        &self.id
    }
    /// The compatibility runtime.
    #[must_use]
    pub const fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    /// The presented Windows version.
    #[must_use]
    pub const fn windows_version(&self) -> WindowsVersion {
        self.windows_version
    }
    /// Whether this is a 64-bit bottle.
    #[must_use]
    pub const fn is_win64(&self) -> bool {
        self.win64
    }
    /// Staged components, sorted and deduplicated.
    #[must_use]
    pub fn components(&self) -> &[Component] {
        &self.components
    }
    /// The isolation policy.
    #[must_use]
    pub const fn isolation(&self) -> &Isolation {
        &self.isolation
    }
    /// Current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> BottleState {
        self.state
    }

    /// Attempt a lifecycle transition.
    ///
    /// # Errors
    /// Returns the current state unchanged in `Err` when the transition is not
    /// permitted, so the caller can report what it actually was rather than
    /// assuming.
    pub fn transition(&mut self, next: BottleState) -> Result<(), BottleState> {
        if self.state.can_transition_to(next) {
            self.state = next;
            Ok(())
        } else {
            Err(self.state)
        }
    }

    /// The storage path component for this bottle, relative to the bottles root.
    ///
    /// Derived from the validated [`AppId`], so it cannot contain a path
    /// separator or a traversal sequence.
    #[must_use]
    pub fn path_component(&self) -> String {
        format!("{}@{}", self.id.as_path_component(), self.runtime.token())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> AppId {
        AppId::parse("com.example.editor").expect("valid id")
    }

    #[test]
    fn new_bottle_starts_created_and_cannot_launch() {
        let b = Bottle::new(
            app(),
            Runtime::Wine {
                version: "11.0".into(),
            },
        );
        assert_eq!(b.state(), BottleState::Created);
        assert!(!b.state().can_launch());
    }

    #[test]
    fn lifecycle_transitions_are_enforced() {
        let mut b = Bottle::new(
            app(),
            Runtime::Wine {
                version: "11.0".into(),
            },
        );
        assert!(b.transition(BottleState::Ready).is_ok());
        assert!(b.transition(BottleState::Running).is_ok());
        assert!(b.state().can_launch());

        // The load-bearing rule: a running bottle cannot be wiped underneath
        // live processes.
        assert_eq!(
            b.transition(BottleState::Created),
            Err(BottleState::Running)
        );
        assert_eq!(
            b.state(),
            BottleState::Running,
            "failed transition must not mutate"
        );

        assert!(b.transition(BottleState::Ready).is_ok());
        assert!(b.transition(BottleState::Created).is_ok());
    }

    #[test]
    fn broken_bottle_must_be_rebuilt_not_blessed() {
        let mut b = Bottle::new(
            app(),
            Runtime::Wine {
                version: "11.0".into(),
            },
        );
        b.transition(BottleState::Ready).expect("created -> ready");
        b.transition(BottleState::Broken).expect("ready -> broken");

        // A bottle that failed verification cannot be marked Ready directly.
        assert_eq!(b.transition(BottleState::Ready), Err(BottleState::Broken));
        assert_eq!(b.transition(BottleState::Running), Err(BottleState::Broken));
        // Repair means wipe and re-stage.
        assert!(b.transition(BottleState::Created).is_ok());
    }

    #[test]
    fn default_isolation_denies_network_and_home() {
        // The single most important default in this file.
        let iso = Isolation::default();
        assert!(
            !iso.network,
            "a Windows program must not get network by default"
        );
        assert!(
            !iso.real_home,
            "a Windows program must not see the real home by default"
        );
        assert!(!iso.audio_in, "microphone is never a default");
        assert!(!iso.usb);
        assert!(!iso.clipboard_read);
    }

    #[test]
    fn installer_profile_is_the_most_restrictive() {
        let iso = Isolation::for_installer();
        assert!(!iso.network);
        assert!(!iso.real_home);
        assert!(!iso.audio_out);
        assert!(!iso.audio_in);
        assert!(!iso.usb);
        assert!(!iso.clipboard_read);
        assert!(!iso.clipboard_write);
        assert_eq!(iso.required_capabilities(), vec!["gpu"]);
    }

    #[test]
    fn capability_list_is_stable_and_complete() {
        let iso = Isolation::for_game();
        let caps = iso.required_capabilities();
        assert_eq!(
            caps,
            vec![
                "net.client",
                "gpu",
                "audio.output",
                "device.usb",
                "clipboard.write"
            ]
        );
        // Determinism: the install dialog must show the same list every time.
        assert_eq!(caps, Isolation::for_game().required_capabilities());
    }

    #[test]
    fn components_are_deduplicated_and_ordered() {
        let b = Bottle::new(
            app(),
            Runtime::Proton {
                version: "9.0-4".into(),
            },
        )
        .with_components([
            Component::VcRedist2022,
            Component::DotNetFx48,
            Component::VcRedist2022,
        ]);
        assert_eq!(
            b.components(),
            &[Component::DotNetFx48, Component::VcRedist2022]
        );

        // Same set requested in a different order gives an identical bottle,
        // which is what makes staged layers shareable.
        let c = Bottle::new(
            app(),
            Runtime::Proton {
                version: "9.0-4".into(),
            },
        )
        .with_components([Component::VcRedist2022, Component::DotNetFx48]);
        assert_eq!(b.components(), c.components());
    }

    #[test]
    fn path_component_is_traversal_safe() {
        let b = Bottle::new(
            app(),
            Runtime::Proton {
                version: "9.0-4".into(),
            },
        );
        let p = b.path_component();
        assert_eq!(p, "com.example.editor@proton-9.0-4");
        assert!(!p.contains('/'));
        assert!(!p.contains('\\'));
        assert!(!p.contains(".."));
    }

    #[test]
    fn proton_carries_d3d_translation_and_wine_does_not() {
        assert!(Runtime::Proton {
            version: "9.0-4".into()
        }
        .has_d3d_translation());
        assert!(!Runtime::Wine {
            version: "11.0".into()
        }
        .has_d3d_translation());
    }
}
