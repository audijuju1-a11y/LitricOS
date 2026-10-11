//! LitricOS Windows compatibility: inspection and install planning.
//!
//! # Layer position
//!
//! This crate sits at **Layer 6 (Native Ecosystem / Applications)**. It depends
//! downward on `litric-core` and on nothing else. It is subordinate to the
//! native platform by construction: nothing in LitricOS Core references it, and
//! removing it entirely leaves a working operating system. That is the
//! architectural invariant from ECOSYSTEM section 22, rule 15, and it is
//! enforced here by the dependency graph rather than by intent.
//!
//! # What this crate does and does not do
//!
//! It **decides**. Given the bytes of a file, it produces a concrete, ordered
//! [`InstallPlan`]: which runtime, which components, which isolation profile,
//! which arguments, in which order, and which capabilities the user is being
//! asked to grant.
//!
//! It **does not execute**. No process spawning, no filesystem writes, no
//! network. Execution belongs to `litric-compatd`, which runs the plan under
//! the capability broker's supervision. Splitting decision from execution is
//! what makes the decision testable on any host, which is why every assertion
//! in this crate is backed by a test that runs today.

#![forbid(unsafe_code)]

pub mod bottle;
pub mod compat;
pub mod installer;
pub mod pe;

pub use bottle::{Bottle, BottleState, Component, Isolation, Runtime, WindowsVersion};
pub use installer::InstallerFamily;
pub use pe::{Machine, PeError, PeInfo, Subsystem};

use litric_core::AppId;

/// How many leading bytes of a file the planner needs.
///
/// Installer signatures live in the stub near the start of the file. Reading 1
/// MiB of a 4 GiB installer is the difference between an instant decision and a
/// visible pause in the file manager.
pub const INSPECT_WINDOW: usize = 1024 * 1024;

/// Why a file cannot be run or installed at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    /// The bytes are not a Windows executable or installer.
    NotWindowsSoftware {
        /// The structural problem found while parsing.
        reason: PeError,
    },
    /// It is a Windows binary, but one that cannot run here.
    CannotRun {
        /// Plain-language explanation suitable for showing to a user.
        reason: &'static str,
    },
}

impl Rejection {
    /// A message suitable for display to a person.
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::NotWindowsSoftware { reason } => {
                format!("This file is not a Windows program ({reason}).")
            }
            Self::CannotRun { reason } => format!("This Windows program cannot run: {reason}."),
        }
    }
}

/// One step of an install plan.
///
/// Steps are ordered and each is independently describable to the user, so the
/// progress UI reports what is actually happening rather than an indeterminate
/// spinner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Create the bottle directory and register its identity.
    CreateBottle {
        /// The bottle to create.
        bottle: Box<Bottle>,
    },
    /// Stage a content-addressed runtime layer into the bottle.
    StageComponent {
        /// The component to stage.
        component: Component,
    },
    /// Run the installer inside the bottle.
    RunInstaller {
        /// Arguments passed to the installer, in order.
        args: Vec<String>,
        /// True when the installer must be driven by `msiexec` rather than
        /// executed directly.
        via_msiexec: bool,
        /// False when the family was not recognised and a human must drive the
        /// installer's own window.
        unattended: bool,
    },
    /// Copy a portable program into the bottle without running an installer.
    InstallPortable,
    /// Scan the bottle for installed applications and register launcher entries.
    RegisterApplications,
    /// Verify the bottle launches, then mark it ready.
    VerifyAndCommit,
}

impl Step {
    /// A short plain-language label for the progress UI.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::CreateBottle { .. } => "Preparing an isolated Windows environment".into(),
            Self::StageComponent { component } => {
                format!("Adding required Windows component: {}", component.token())
            }
            Self::RunInstaller {
                unattended: true, ..
            } => "Installing".into(),
            Self::RunInstaller {
                unattended: false, ..
            } => "Starting the installer (it will ask you questions)".into(),
            Self::InstallPortable => "Setting up the program".into(),
            Self::RegisterApplications => "Adding to your applications".into(),
            Self::VerifyAndCommit => "Checking that it starts".into(),
        }
    }
}

/// A complete, ordered plan for making a piece of Windows software usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    /// What was learned about the binary.
    pub pe: PeInfo,
    /// Which installer toolkit, if any.
    pub family: InstallerFamily,
    /// The environment the software will live in.
    pub bottle: Bottle,
    /// Ordered steps.
    pub steps: Vec<Step>,
    /// Capability strings the user is being asked to grant, in a stable order.
    pub requested_capabilities: Vec<&'static str>,
    /// Things the user should know before agreeing. Never suppressed.
    pub warnings: Vec<String>,
}

impl InstallPlan {
    /// Whether the plan can complete without asking the user anything after the
    /// initial consent.
    #[must_use]
    pub fn is_unattended(&self) -> bool {
        self.steps.iter().all(|s| {
            !matches!(
                s,
                Step::RunInstaller {
                    unattended: false,
                    ..
                }
            )
        })
    }
}

/// Decide how to install or run a piece of Windows software.
///
/// `head` is the first [`INSPECT_WINDOW`] bytes of the file (fewer is fine for
/// small files). `app_id` is the identity the software will be registered
/// under, chosen by the caller from the file name and publisher metadata.
///
/// # Errors
/// [`Rejection`] when the file is not Windows software, or is Windows software
/// that categorically cannot run here. The distinction matters: the first is
/// "you clicked the wrong file", the second is "this will never work", and the
/// UI says different things for each.
pub fn plan(head: &[u8], app_id: AppId) -> Result<InstallPlan, Rejection> {
    let is_msi = pe::is_msi(head);

    // MSI databases are OLE compound files, not PE images, so they are handled
    // before PE inspection rather than being reported as a malformed PE.
    let info = if is_msi {
        PeInfo {
            machine: Machine::Amd64,
            subsystem: Subsystem::Gui,
            is_64bit: true,
            is_dll: false,
            is_dotnet: false,
            min_os: (10, 0),
            section_count: 0,
            hardening: pe::Hardening::default(),
            has_certificate_table: false,
        }
    } else {
        pe::inspect(head).map_err(|reason| Rejection::NotWindowsSoftware { reason })?
    };

    if let Some(reason) = info.blocking_reason() {
        return Err(Rejection::CannotRun { reason });
    }

    let family = installer::detect(head, !is_msi);

    // --- runtime selection -------------------------------------------------
    // Decided on evidence, not on a guess. A .NET application gains nothing from
    // Proton's game patches and is better served by upstream Wine; anything else
    // gets Proton, because its D3D-to-Vulkan translation and shader pre-caching
    // are strictly additive for software that does not use them.
    let runtime = if info.is_dotnet {
        Runtime::Wine {
            version: "11.0".into(),
        }
    } else {
        Runtime::Proton {
            version: "9.0-4".into(),
        }
    };

    // --- components --------------------------------------------------------
    let mut components = Vec::new();
    if info.is_dotnet {
        // Distinguishing .NET Framework from modern .NET needs the CLR header's
        // version string, which is beyond the header window. Staging the
        // Framework covers the large majority of desktop software that ships as
        // a managed binary, and a missing modern-.NET runtime surfaces as a
        // clear launch failure rather than silent breakage.
        components.push(Component::DotNetFx48);
    }
    if matches!(family, InstallerFamily::Squirrel) {
        components.push(Component::VcRedist2022);
    }
    // Core fonts are cheap and prevent the single most common cosmetic failure
    // in Wine: text rendering as boxes.
    components.push(Component::CoreFonts);

    let isolation = Isolation::for_installer();

    let bottle = Bottle::new(app_id, runtime)
        .with_windows_version(WindowsVersion::Win10)
        .with_components(components.iter().copied())
        .with_isolation(isolation.clone());

    // --- steps -------------------------------------------------------------
    let mut steps = vec![Step::CreateBottle {
        bottle: Box::new(bottle.clone()),
    }];
    for component in bottle.components() {
        steps.push(Step::StageComponent {
            component: *component,
        });
    }

    match family {
        InstallerFamily::NotAnInstaller => steps.push(Step::InstallPortable),
        other => {
            let silent = other.silent_args();
            steps.push(Step::RunInstaller {
                args: silent
                    .unwrap_or(&[])
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect(),
                via_msiexec: other.needs_msiexec(),
                unattended: silent.is_some(),
            });
        }
    }
    steps.push(Step::RegisterApplications);
    steps.push(Step::VerifyAndCommit);

    // --- warnings ----------------------------------------------------------
    // These are shown, never suppressed. The user is about to run untrusted
    // code; withholding what we know about it to keep the dialog tidy is not a
    // trade we make.
    let mut warnings = Vec::new();

    if matches!(family, InstallerFamily::Unknown) {
        warnings.push(
            "This installer is not one LitricOS recognises, so it will show its own \
             windows and ask you questions."
                .into(),
        );
    }
    if info.has_certificate_table {
        warnings.push(
            "This program carries a digital signature. LitricOS has not verified it: \
             a signature being present does not mean it is valid or that the \
             publisher is trustworthy."
                .into(),
        );
    } else {
        warnings.push("This program is not digitally signed.".into());
    }
    if info.hardening.score() < 2 {
        warnings.push(
            "This program was built without common security protections \
             (address randomisation or data-execution prevention). It will run \
             inside a sandbox regardless."
                .into(),
        );
    }
    if !info.is_64bit {
        warnings.push("This is a 32-bit program. It will run, using compatibility support.".into());
    }

    Ok(InstallPlan {
        pe: info,
        family,
        requested_capabilities: isolation.required_capabilities(),
        bottle,
        steps,
        warnings,
    })
}

#[cfg(test)]
// Offset arithmetic in test scaffolding builds synthetic headers from small
// constants. The production rule stays absolute -- this exemption is scoped to
// the test module and visible in review, rather than weakening the workspace
// lint for everyone.
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn id() -> AppId {
        AppId::parse("com.example.thing").expect("valid id")
    }

    /// A minimal structurally valid 64-bit GUI PE, plus an optional marker.
    fn exe(marker: &[u8], dotnet: bool) -> Vec<u8> {
        let mut v = build_pe(dotnet);
        v.extend_from_slice(marker);
        v
    }

    fn build_pe(dotnet: bool) -> Vec<u8> {
        // Mirrors pe::tests::synth_pe for a 64-bit GUI executable. Kept local so
        // this crate's public behaviour is tested through its public surface.
        let opt_len: usize = 240;
        let pe_off: usize = 0x80;
        let mut b = vec![0u8; pe_off + 24 + opt_len];
        let put16 = |b: &mut Vec<u8>, off: usize, v: u16| {
            if let Some(s) = b.get_mut(off..off + 2) {
                s.copy_from_slice(&v.to_le_bytes());
            }
        };
        let put32 = |b: &mut Vec<u8>, off: usize, v: u32| {
            if let Some(s) = b.get_mut(off..off + 4) {
                s.copy_from_slice(&v.to_le_bytes());
            }
        };
        put16(&mut b, 0, 0x5a4d);
        put32(&mut b, 0x3c, pe_off as u32);
        put32(&mut b, pe_off, 0x0000_4550);
        put16(&mut b, pe_off + 4, 0x8664);
        put16(&mut b, pe_off + 6, 4);
        put16(&mut b, pe_off + 20, opt_len as u16);
        put16(&mut b, pe_off + 22, 0x0002);
        let opt = pe_off + 24;
        put16(&mut b, opt, 0x020b);
        put16(&mut b, opt + 40, 10);
        put16(&mut b, opt + 68, 2);
        put16(&mut b, opt + 70, 0x0040 | 0x0100); // ASLR + NX
        if dotnet {
            put32(&mut b, opt + 112 + 14 * 8, 0x2000);
        }
        b
    }

    #[test]
    fn plans_an_nsis_install_unattended() {
        let bytes = exe(b"Nullsoft.NSIS.exehead", false);
        let p = plan(&bytes, id()).expect("should plan");
        assert_eq!(p.family, InstallerFamily::Nsis);
        assert!(p.is_unattended());
        assert!(matches!(p.steps.first(), Some(Step::CreateBottle { .. })));
        assert!(matches!(p.steps.last(), Some(Step::VerifyAndCommit)));
        let ran = p.steps.iter().any(
            |s| matches!(s, Step::RunInstaller { args, unattended: true, .. } if args == &["/S"]),
        );
        assert!(ran, "NSIS plan must drive the installer with /S");
    }

    #[test]
    fn unrecognised_installer_is_attended_and_says_so() {
        // The root-cause behaviour: never guess a silent switch.
        let bytes = exe(b"UninstallString", false);
        let p = plan(&bytes, id()).expect("should plan");
        assert_eq!(p.family, InstallerFamily::Unknown);
        assert!(!p.is_unattended());
        assert!(
            p.warnings
                .iter()
                .any(|w| w.contains("not one LitricOS recognises")),
            "the user must be told the install will be interactive"
        );
    }

    #[test]
    fn dotnet_binaries_get_wine_and_the_framework() {
        let bytes = exe(b"", true);
        let p = plan(&bytes, id()).expect("should plan");
        assert!(p.pe.is_dotnet);
        assert!(matches!(p.bottle.runtime(), Runtime::Wine { .. }));
        assert!(p.bottle.components().contains(&Component::DotNetFx48));
    }

    #[test]
    fn native_binaries_get_proton() {
        let bytes = exe(b"", false);
        let p = plan(&bytes, id()).expect("should plan");
        assert!(matches!(p.bottle.runtime(), Runtime::Proton { .. }));
        assert!(p.bottle.runtime().has_d3d_translation());
    }

    #[test]
    fn portable_programs_skip_the_installer_step() {
        let bytes = exe(b"an ordinary program", false);
        let p = plan(&bytes, id()).expect("should plan");
        assert_eq!(p.family, InstallerFamily::NotAnInstaller);
        assert!(p.steps.iter().any(|s| matches!(s, Step::InstallPortable)));
        assert!(!p
            .steps
            .iter()
            .any(|s| matches!(s, Step::RunInstaller { .. })));
    }

    #[test]
    fn msi_is_planned_via_msiexec() {
        let mut v = vec![0xd0u8, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];
        v.extend_from_slice(&[0u8; 256]);
        let p = plan(&v, id()).expect("should plan");
        assert_eq!(p.family, InstallerFamily::Msi);
        assert!(p.steps.iter().any(|s| matches!(
            s,
            Step::RunInstaller {
                via_msiexec: true,
                ..
            }
        )));
    }

    #[test]
    fn installs_request_only_gpu_by_default() {
        // The security-critical assertion in this crate. An installer must not
        // silently receive network or the user's home directory.
        let bytes = exe(b"Nullsoft.NSIS.exehead", false);
        let p = plan(&bytes, id()).expect("should plan");
        assert_eq!(p.requested_capabilities, vec!["gpu"]);
        assert!(!p.requested_capabilities.contains(&"net.client"));
        assert!(!p.requested_capabilities.contains(&"fs.read.home"));
        assert!(!p.requested_capabilities.contains(&"microphone"));
    }

    #[test]
    fn signature_presence_is_never_reported_as_validity() {
        let bytes = exe(b"", false);
        let p = plan(&bytes, id()).expect("should plan");
        // No certificate table in the synthetic image.
        assert!(p
            .warnings
            .iter()
            .any(|w| w.contains("not digitally signed")));
        // And no warning anywhere may assert that something is verified.
        assert!(
            !p.warnings
                .iter()
                .any(|w| w.to_lowercase().contains("verified publisher")),
            "must never claim verification we did not perform"
        );
    }

    #[test]
    fn rejects_non_windows_files_distinctly_from_unrunnable_ones() {
        let garbage = vec![0u8; 4096];
        assert!(matches!(
            plan(&garbage, id()),
            Err(Rejection::NotWindowsSoftware { .. })
        ));

        // A DLL is Windows software, but not something you "run".
        let mut dll = build_pe(false);
        if let Some(s) = dll.get_mut(0x80 + 22..0x80 + 24) {
            s.copy_from_slice(&0x2000u16.to_le_bytes());
        }
        let r = plan(&dll, id());
        assert!(matches!(r, Err(Rejection::CannotRun { .. })));
        assert!(r.unwrap_err().user_message().contains("DLL"));
    }

    #[test]
    fn every_step_has_a_human_label() {
        let bytes = exe(b"Inno Setup Setup Data", false);
        let p = plan(&bytes, id()).expect("should plan");
        for step in &p.steps {
            assert!(!step.label().is_empty(), "step {step:?} has no label");
        }
    }

    #[test]
    fn planning_never_panics_on_arbitrary_input() {
        // Untrusted input, again. The file manager calls this on whatever the
        // user double-clicked.
        let full = exe(b"Nullsoft.NSIS.exehead", true);
        for cut in 0..full.len() {
            let _ = plan(full.get(..cut).unwrap_or(&[]), id());
        }
        for pattern in [&[0xffu8; 300][..], &[0u8; 300][..], b"MZ", b""] {
            let _ = plan(pattern, id());
        }
    }
}
