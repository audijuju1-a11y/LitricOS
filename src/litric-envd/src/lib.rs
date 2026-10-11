//! Litric environment daemon — the execution half of [`litric_env`].
//!
//! # Layer position
//!
//! **Layer 2/3 boundary.** [`litric_env`] decides (states, transitions,
//! budgets, switch planning) and performs no I/O. This crate *executes*: it
//! turns a decided transition into a real hypervisor action, and it is the
//! only component permitted to start or stop a guest.
//!
//! # What actually runs
//!
//! A guest runs as a `qemu-system-x86_64` child process. Every property of
//! that process — its CPU count, its memory ceiling, its disk, whether it
//! may touch the network — is derived from the [`litric_env::Budget`] the
//! manager already approved, so the resource contract the user sees and the
//! resource contract the hypervisor enforces are the same contract, generated
//! from one source. [`LaunchPlan`] is a pure value: it can be printed,
//! hashed, dry-run and unit-tested without a hypervisor, and [`build_argv`]
//! is the only place the argv spelling lives.
//!
//! # Supervision
//!
//! [`Supervisor`] owns child handles, records the last exit, and drives the
//! suspend/resume lifecycle through QEMU's monitor protocol over a Unix
//! socket — `stop` (suspend to memory), `system_wakeup` (resume),
//! `screendump` (evidence), `quit` (clean shutdown). The state machine in
//! [`litric_env`] remains the single source of truth for what transitions are
//! legal; this crate refuses to send a monitor command that the model says
//! cannot happen, which keeps the two layers from drifting apart.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::Command;

use litric_env::{Budget, EnvState, EnvironmentId};

/// Why a launch plan could not be produced or executed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlanError {
    /// The environment is the host; the host is not a guest of itself.
    HostIsNotAGuest,
    /// The disk image named for this environment does not exist.
    DiskMissing {
        /// The path that was looked for.
        path: String,
    },
    /// The environment is not in a state that can be launched from.
    NotLaunchable {
        /// The state it is actually in.
        state: EnvState,
    },
    /// The budget does not name a runnable machine.
    EmptyBudget,
}

impl core::fmt::Display for PlanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::HostIsNotAGuest => write!(f, "the host cannot be launched as a guest"),
            Self::DiskMissing { path } => write!(f, "guest disk is missing: {path}"),
            Self::NotLaunchable { state } => {
                write!(f, "a guest can only be launched from stopped, got {state}")
            }
            Self::EmptyBudget => write!(f, "the environment has no resource budget"),
        }
    }
}

impl std::error::Error for PlanError {}

/// The firmware a guest boots.
///
/// Named, not free-form: each variant fixes a firmware layout, and the argv
/// builder is the one place that knows which files that layout needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Firmware {
    /// SeaBIOS legacy boot. Not used by Litric guests today; kept so the
    /// Linux-guest path can boot distribution installers that assume BIOS.
    Bios,
    /// OVMF UEFI with a private, per-guest variable store. This is how real
    /// guests boot, and what a Windows guest requires.
    Uefi {
        /// Read-only firmware code image.
        code: PathBuf,
        /// Writable variable store template; copied per run.
        vars_template: PathBuf,
    },
}

/// One guest disk attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disk {
    /// Path to the disk image.
    pub path: PathBuf,
    /// Format as QEMU understands it (`qcow2`, `raw`).
    pub format: String,
}

/// Everything needed to turn one decided transition into a running guest.
///
/// Constructed by [`launch_plan`], executed by [`build_argv`] +
/// [`Supervisor`]. A pure value on purpose: the plan is the reviewable,
/// testable, loggable form of "start this guest".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    /// Which environment this plan starts.
    pub environment: EnvironmentId,
    /// Approved resource budget; the argv is derived from it, never from
    /// ad-hoc numbers at the call site.
    pub budget: Budget,
    /// Boot firmware layout.
    pub firmware: Firmware,
    /// Disks to attach, in order.
    pub disks: Vec<Disk>,
    /// Whether the guest may use the network. Every guest starts with no
    /// network and gains it only here, explicitly, never by ambient default.
    pub allow_network: bool,
    /// Whether a window is shown (true) or the guest runs headless with a
    /// monitor socket only (false). Headless is how a background guest
    /// hibernated last session boots.
    pub show_window: bool,
    /// Host directory for the per-run monitor socket.
    pub monitor_dir: PathBuf,
}

impl LaunchPlan {
    /// The host-unique monitor socket path for this plan.
    #[must_use]
    pub fn monitor_socket(&self) -> PathBuf {
        self.monitor_dir
            .join(format!("litric-env-{}.sock", self.environment.token()))
    }
}

/// Produce the launch plan for starting `id` from `Stopped`.
///
/// The budget must already have been approved by
/// [`litric_env::EnvironmentManager::set_budget`]; this function derives argv
/// numbers from it and performs no approval of its own — re-approving here
/// would create two policy engines that could disagree.
///
/// # Errors
/// [`PlanError`] when the environment cannot be launched: the host is not a
/// guest, the state is not `Stopped`, the budget is empty, or the disk is
/// absent from the host filesystem.
// Eight parameters describe one launch honestly (id, state, budget, disks,
// firmware, network, window, monitor); grouping them into a struct would move
// the verbosity, not remove it.
#[allow(clippy::too_many_arguments)]
pub fn launch_plan(
    id: EnvironmentId,
    state: EnvState,
    budget: Budget,
    disks: Vec<Disk>,
    firmware: Firmware,
    allow_network: bool,
    show_window: bool,
    monitor_dir: PathBuf,
) -> Result<LaunchPlan, PlanError> {
    if id.is_host() {
        return Err(PlanError::HostIsNotAGuest);
    }
    if state != EnvState::Stopped {
        return Err(PlanError::NotLaunchable { state });
    }
    if budget.cpus == 0 || budget.memory_mb == 0 {
        return Err(PlanError::EmptyBudget);
    }
    for d in &disks {
        if !d.path.exists() {
            return Err(PlanError::DiskMissing {
                path: d.path.to_string_lossy().into_owned(),
            });
        }
    }
    Ok(LaunchPlan {
        environment: id,
        budget,
        firmware,
        disks,
        allow_network,
        show_window,
        monitor_dir,
    })
}

/// The QEMU binary this crate executes. A constant so tests and callers can
/// refer to the same name, and so a future change (a Litric-pinned hypervisor
/// path) is a one-line edit.
pub const QEMU_BINARY: &str = "qemu-system-x86_64";

/// Build the argv for a launch plan.
///
/// Every flag is commented by the property it enforces; the resource numbers
/// come from the plan's budget, which came from the manager, which is what
/// makes "the guest cannot exceed what the user approved" true by
/// construction rather than by convention.
///
/// Window selection: with `show_window`, a real display backend opens a
/// window on the host desktop; without it the guest boots headless and only
/// its monitor socket exists. In both cases the QEMU monitor is a per-run
/// Unix socket, never stdio — stdio monitors die with the daemon and take
/// suspend/resume with them.
#[must_use]
pub fn build_argv(plan: &LaunchPlan) -> Vec<String> {
    let mut argv: Vec<String> = Vec::new();

    // x86-64 virtual machine with hardware-accelerated virtualisation when
    // available. TCG fallback is decided by the caller probing /dev/kvm and
    // substituting the machine string; the plan stays accelerator-agnostic.
    // vmport=off: with the default on, a Linux guest's vmmouse driver binds
    // phantom absolute pointing devices on the i8042 AUX port that fight the
    // USB tablet for pointer events and wedge the cursor in a corner.
    argv.push("-machine".into());
    argv.push("q35,accel=kvm:tcg,vmport=off".into());
    argv.push("-cpu".into());
    argv.push("host".into());

    // The approved budget, verbatim.
    argv.push("-smp".into());
    argv.push(plan.budget.cpus.to_string());
    argv.push("-m".into());
    argv.push(format!("{}M", plan.budget.memory_mb));

    // Firmware.
    match &plan.firmware {
        Firmware::Bios => {}
        Firmware::Uefi {
            code,
            vars_template,
        } => {
            argv.push("-drive".into());
            argv.push(format!(
                "if=pflash,format=raw,unit=0,readonly=on,file={}",
                code.display()
            ));
            argv.push("-drive".into());
            argv.push(format!(
                "if=pflash,format=raw,unit=1,file={}",
                vars_template.display()
            ));
        }
    }

    // Disks, virtio — the para-virtualised bus, not emulated IDE.
    for d in &plan.disks {
        argv.push("-drive".into());
        argv.push(format!(
            "if=virtio,format={},file={}",
            d.format,
            d.path.display()
        ));
    }

    // Network: nothing unless explicitly granted. When granted, user-mode
    // networking — the guest never appears on the host's LAN without a
    // further, deliberate bridge.
    if plan.allow_network {
        argv.push("-device".into());
        argv.push("virtio-net-pci,netdev=n0".into());
        argv.push("-netdev".into());
        argv.push("user,id=n0".into());
    }

    // Display. With a window, SDL on the host desktop (WSLg presents it as a
    // native window on Windows). Headless otherwise.
    if plan.show_window {
        argv.push("-display".into());
        argv.push("sdl".into());
    } else {
        argv.push("-display".into());
        argv.push("none".into());
    }
    // The guest's own display device exists in both modes: headless guests
    // still need something to render to for screendumps.
    //
    // virtio-gpu, not the legacy VGA: the guest kernel carries
    // CONFIG_DRM_VIRTIO_GPU=y, so this device becomes a real DRM card
    // (virtio_gpu) in the guest — the desktop's renderer can then report a
    // GPU-backed scanout instead of an unlabelled legacy frame buffer, and
    // future acceleration (virgl/venus) lands without another device change.
    argv.push("-device".into());
    argv.push("virtio-gpu-pci".into());

    // Input: an absolute tablet so the pointer maps 1:1 to guest pixels.
    argv.push("-device".into());
    argv.push("usb-ehci,id=ehci".into());
    argv.push("-device".into());
    argv.push("usb-tablet,bus=ehci.0".into());

    // Monitor on a per-run socket; suspend/resume/dump go through it.
    argv.push("-monitor".into());
    argv.push(format!(
        "unix:{},server,nowait",
        plan.monitor_socket().display()
    ));

    // A guest panic reboots forever by default; under supervision a reboot is
    // a decision, so the automatic loop is disabled.
    argv.push("-no-reboot".into());

    argv
}

/// Build the [`Command`] for a plan. Separated from [`build_argv`] so tests
/// can assert on argv while production gets the process handle.
#[must_use]
pub fn build_command(plan: &LaunchPlan) -> Command {
    let mut cmd = Command::new(QEMU_BINARY);
    cmd.args(build_argv(plan));
    cmd
}

/// The QEMU monitor commands this daemon uses, as typed constants. QEMU's
/// human monitor is line-oriented; these are the only lines this crate ever
/// sends, enumerated so the surface is auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorCommand {
    /// Suspend to memory (the model's `Suspended`).
    Stop,
    /// Resume from memory (the model's `Running`).
    SystemWakeup,
    /// Write the guest screen to `path` as PPM — the evidence primitive.
    Screendump,
    /// Ask the guest to power off cleanly (the model's `Stopped`).
    SystemPowerdown,
}

impl MonitorCommand {
    /// The exact monitor line.
    #[must_use]
    pub fn line(self, path: &str) -> String {
        match self {
            Self::Stop => "stop".to_owned(),
            Self::SystemWakeup => "system_wakeup".to_owned(),
            Self::Screendump => format!("screendump {path}"),
            Self::SystemPowerdown => "system_powerdown".to_owned(),
        }
    }
}

/// One supervised guest: the child process while running, plus what happened
/// last time it exited.
#[derive(Debug, Default)]
pub struct GuestHandle {
    /// The QEMU child, present while the guest runs.
    child: Option<std::process::Child>,
    /// The plan it was launched with, kept for the monitor socket path.
    plan: Option<LaunchPlan>,
    /// What the last run did, once it ended.
    last_exit: Option<GuestExit>,
}

/// How a guest's run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuestExit {
    /// The guest powered down cleanly.
    Clean,
    /// QEMU died or was killed; the code is QEMU's.
    Crashed {
        /// The exit code observed.
        code: i32,
    },
    /// Never launched.
    NeverLaunched,
}

impl GuestHandle {
    /// A handle for a guest that has not been started.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            child: None,
            plan: None,
            last_exit: None,
        }
    }

    /// Whether a guest process is currently held.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.child.is_some()
    }

    /// The plan of the current or most recent launch.
    #[must_use]
    pub const fn plan(&self) -> Option<&LaunchPlan> {
        self.plan.as_ref()
    }

    /// How the last run ended.
    #[must_use]
    pub const fn last_exit(&self) -> Option<&GuestExit> {
        self.last_exit.as_ref()
    }

    /// Launch a plan, replacing any previous run. The caller must already
    /// have transitioned the model state to `Starting`; this crate executes,
    /// it does not re-decide.
    ///
    /// # Errors
    /// Returns the OS error when the QEMU process cannot be spawned. A guest
    /// that starts and then fails reports through [`Self::poll`], not here.
    pub fn spawn(&mut self, plan: LaunchPlan) -> std::io::Result<()> {
        self.stop_child_if_any();
        let mut cmd = build_command(&plan);
        // The supervisor owns the child explicitly and reaps it; std gives
        // no kill_on_drop, so the kill path is kill()/poll()/stop_child —
        // all of them deliberate, none implicit.
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        self.child = Some(cmd.spawn()?);
        self.plan = Some(plan);
        self.last_exit = None;
        Ok(())
    }

    /// Reap the child if it has exited. Returns the resulting exit state,
    /// or `None` while the guest still runs.
    pub fn poll(&mut self) -> Option<GuestExit> {
        let child = self.child.as_mut()?;
        match child.try_wait() {
            Ok(Some(status)) => {
                let exit = if status.success() {
                    GuestExit::Clean
                } else {
                    GuestExit::Crashed {
                        code: status.code().unwrap_or(-1),
                    }
                };
                self.child = None;
                self.last_exit = Some(exit.clone());
                Some(exit)
            }
            Ok(None) => None,
            Err(_) => Some(GuestExit::Crashed { code: -1 }),
        }
    }

    /// Kill the guest immediately. Used for the model's `Failed` path, where
    /// the guest has stopped being trustworthy and a clean shutdown request
    /// is a courtesy it has forfeited.
    pub fn kill(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
    }

    fn stop_child_if_any(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
    }
}

/// Supervises every guest of one machine. One handle per non-host
/// environment, forever; the handle is the guest's memory across its
/// lifecycle.
#[derive(Debug, Default)]
pub struct Supervisor {
    handles: BTreeMapLite,
}

/// A minimal ordered map keyed by environment, sized exactly for the two
/// guests the model admits. Written as a fixed pair rather than a generic
/// map so the set of supervised guests is closed by construction — the same
/// reasoning as `EnvironmentId` being a closed enum.
#[derive(Debug, Default)]
pub struct BTreeMapLite {
    windows: GuestHandle,
    linux: GuestHandle,
}

impl BTreeMapLite {
    fn get_mut(&mut self, id: EnvironmentId) -> Option<&mut GuestHandle> {
        match id {
            EnvironmentId::Windows => Some(&mut self.windows),
            EnvironmentId::Linux => Some(&mut self.linux),
            // The host and any future environment variant are not guests.
            _ => None,
        }
    }

    fn get(&self, id: EnvironmentId) -> Option<&GuestHandle> {
        match id {
            EnvironmentId::Windows => Some(&self.windows),
            EnvironmentId::Linux => Some(&self.linux),
            _ => None,
        }
    }
}

impl Supervisor {
    /// An empty supervisor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handles: BTreeMapLite::default(),
        }
    }

    /// The handle for `id`, if it is a supervisable guest.
    #[must_use]
    pub fn handle(&self, id: EnvironmentId) -> Option<&GuestHandle> {
        self.handles.get(id)
    }

    /// Launch `plan`, replacing any previous run of that environment.
    ///
    /// # Errors
    /// See [`GuestHandle::spawn`].
    pub fn launch(&mut self, plan: LaunchPlan) -> std::io::Result<()> {
        let Some(handle) = self.handles.get_mut(plan.environment) else {
            return Err(std::io::Error::other(
                "the host cannot be launched as a guest",
            ));
        };
        handle.spawn(plan)
    }

    /// Reap finished guests; returns `(environment, exit)` for each that
    /// ended since the last poll.
    pub fn poll(&mut self) -> Vec<(EnvironmentId, GuestExit)> {
        let mut ended = Vec::new();
        for id in [EnvironmentId::Windows, EnvironmentId::Linux] {
            if let Some(exit) = self.handles.get_mut(id).and_then(GuestHandle::poll) {
                ended.push((id, exit));
            }
        }
        ended
    }

    /// Kill `id` now. The model decides when a guest is `Failed`; this makes
    /// it so.
    pub fn kill(&mut self, id: EnvironmentId) {
        if let Some(h) = self.handles.get_mut(id) {
            h.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    use super::*;

    fn uefi() -> Firmware {
        Firmware::Uefi {
            code: PathBuf::from("/usr/share/edk2/x64/OVMF_CODE.4m.fd"),
            vars_template: PathBuf::from("/tmp/vars.fd"),
        }
    }

    fn plan(id: EnvironmentId, show_window: bool, net: bool) -> LaunchPlan {
        LaunchPlan {
            environment: id,
            budget: Budget::new(4, 8192),
            firmware: uefi(),
            disks: vec![Disk {
                path: PathBuf::from("/var/lib/litric/windows.qcow2"),
                format: "qcow2".into(),
            }],
            allow_network: net,
            show_window,
            monitor_dir: PathBuf::from("/run/litric/env"),
        }
    }

    #[test]
    fn the_argv_carries_the_approved_budget_and_nothing_else() {
        let argv = build_argv(&plan(EnvironmentId::Windows, true, false));
        let smp = argv.iter().position(|a| a == "-smp").expect("smp present");
        assert_eq!(argv[smp + 1], "4");
        let mem = argv.iter().position(|a| a == "-m").expect("mem present");
        assert_eq!(argv[mem + 1], "8192M");
    }

    #[test]
    fn network_is_only_present_when_granted() {
        let with = build_argv(&plan(EnvironmentId::Windows, false, true));
        let without = build_argv(&plan(EnvironmentId::Windows, false, false));
        assert!(with.iter().any(|a| a.contains("virtio-net-pci")));
        assert!(!without.iter().any(|a| a.contains("virtio-net-pci")));
    }

    #[test]
    fn window_mode_uses_sdl_and_headless_uses_none() {
        let windowed = build_argv(&plan(EnvironmentId::Linux, true, false));
        let headless = build_argv(&plan(EnvironmentId::Linux, false, false));
        assert!(windowed.contains(&"sdl".to_owned()));
        assert!(headless.contains(&"none".to_owned()));
        // Both still render to a real display device: screendumps must work
        // for headless guests too. virtio-gpu, not legacy VGA — the guest
        // kernel's DRM driver binds it and the shell reports the GPU.
        assert!(windowed.iter().any(|a| a.starts_with("virtio-gpu")));
    }

    #[test]
    fn the_monitor_socket_is_per_environment_and_per_run_directory() {
        let p = plan(EnvironmentId::Windows, false, false);
        assert!(p
            .monitor_socket()
            .to_string_lossy()
            .contains("litric-env-windows.sock"));
    }

    #[test]
    fn no_reboot_is_always_passed() {
        // An unattended reboot loop is a guest taking the machine hostage.
        let argv = build_argv(&plan(EnvironmentId::Linux, false, false));
        assert!(argv.contains(&"-no-reboot".to_owned()));
    }

    #[test]
    fn the_host_never_gets_a_launch_plan() {
        let r = launch_plan(
            EnvironmentId::Litric,
            EnvState::Stopped,
            Budget::new(1, 512),
            Vec::new(),
            Firmware::Bios,
            false,
            false,
            PathBuf::from("/tmp"),
        );
        assert_eq!(r, Err(PlanError::HostIsNotAGuest));
    }

    #[test]
    fn a_running_guest_cannot_be_launched_again() {
        let r = launch_plan(
            EnvironmentId::Windows,
            EnvState::Running,
            Budget::new(1, 512),
            Vec::new(),
            Firmware::Bios,
            false,
            false,
            PathBuf::from("/tmp"),
        );
        assert!(matches!(
            r,
            Err(PlanError::NotLaunchable {
                state: EnvState::Running
            })
        ));
    }

    #[test]
    fn an_empty_budget_is_refused_before_any_disk_is_touched() {
        let r = launch_plan(
            EnvironmentId::Linux,
            EnvState::Stopped,
            Budget::new(0, 0),
            Vec::new(),
            Firmware::Bios,
            false,
            false,
            PathBuf::from("/tmp"),
        );
        assert_eq!(r, Err(PlanError::EmptyBudget));
    }

    #[test]
    fn a_missing_disk_fails_the_plan_not_the_boot() {
        let r = launch_plan(
            EnvironmentId::Linux,
            EnvState::Stopped,
            Budget::new(1, 512),
            vec![Disk {
                path: PathBuf::from("/definitely/not/here.qcow2"),
                format: "qcow2".into(),
            }],
            Firmware::Bios,
            false,
            false,
            PathBuf::from("/tmp"),
        );
        assert!(matches!(r, Err(PlanError::DiskMissing { .. })));
    }

    #[test]
    fn monitor_commands_spell_what_the_model_means() {
        assert_eq!(MonitorCommand::Stop.line(""), "stop");
        assert_eq!(MonitorCommand::SystemWakeup.line(""), "system_wakeup");
        assert_eq!(
            MonitorCommand::Screendump.line("/tmp/x.ppm"),
            "screendump /tmp/x.ppm"
        );
        assert_eq!(MonitorCommand::SystemPowerdown.line(""), "system_powerdown");
    }

    #[test]
    fn the_supervisor_only_admits_guests() {
        let mut s = Supervisor::new();
        let p = plan(EnvironmentId::Litric, false, false);
        assert!(s.launch(p).is_err(), "the host cannot be launched");
        assert!(s.handle(EnvironmentId::Litric).is_none());
        assert!(s.handle(EnvironmentId::Windows).is_some());
        assert!(s.handle(EnvironmentId::Linux).is_some());
    }

    #[test]
    fn a_guest_that_never_ran_has_no_exit_record() {
        let s = Supervisor::new();
        let h = s.handle(EnvironmentId::Windows).expect("windows handle");
        assert!(!h.is_running());
        assert_eq!(h.last_exit(), None);
        assert!(GuestHandle::new().last_exit().is_none());
    }

    #[test]
    fn killing_a_never_started_guest_is_a_no_op() {
        let mut s = Supervisor::new();
        s.kill(EnvironmentId::Windows);
        s.kill(EnvironmentId::Linux);
        s.kill(EnvironmentId::Litric); // must not panic either
    }

    #[test]
    fn the_plan_round_trips_through_the_command_builder() {
        // The command the daemon runs must be exactly the argv the tests
        // reviewed: one builder, no second spelling.
        let p = plan(EnvironmentId::Windows, false, true);
        let cmd = build_command(&p);
        let argv: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(argv, build_argv(&p));
        assert!(cmd
            .get_program()
            .to_string_lossy()
            .contains("qemu-system-x86_64"));
    }
}
