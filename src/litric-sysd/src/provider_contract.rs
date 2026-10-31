//! C1 — the provider contract the shell and the daemon share.
//!
//! The shell must never grow a D-Bus client. It already reaches the machine
//! through `litric-sysd`'s socket, but that was an accident of the transport
//! rather than a contract: the action dispatch in [`crate::state`] called the
//! concrete [`crate::providers`] functions directly. This module names the seam
//! for the three families C1 covers — audio, network and Bluetooth — and adds
//! the authentication family that shares the same action path.
//!
//! [`ToolProvider`] is the production implementation: it delegates to the base
//! tools (or, for audio level and Bluetooth power, the tools' own clients)
//! through the existing [`Runner`], so behaviour and reply lines are unchanged.
//! [`FakeProvider`] is the contract fake used by daemon tests: it answers with
//! scripted values and records the redacted action rendering, without spawning
//! a process or touching a sound server, a radio or a network.
//!
//! The contract deliberately exposes the existing [`Outcome`] rather than a new
//! reply language. A second reply format would be a second source of truth for
//! what `applied=` means, and the protocol tests already hold the first one.

use std::collections::VecDeque;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use crate::action::{Outcome, Runner};
use crate::providers::{self, AudioAction, AuthAction, BtAction, NetAction, SystemAction};
use crate::services::ServiceAction;

/// A measured default-sink volume and mute flag.
///
/// `None` is still represented outside this type: a provider that cannot answer
/// returns `Err`, and the daemon leaves its previous measurement in place rather
/// than inventing zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioLevel {
    /// The sink's volume, 0 to 100.
    pub volume: u8,
    /// Whether the sink is muted.
    pub muted: bool,
}

/// The four provider families the daemon can act on.
///
/// The trait is object-safe and is what [`crate::state::Sysd`] holds. It exists
/// so a test can substitute behaviour that would otherwise need a booted machine,
/// and so the shell's only route to these operations is the daemon contract,
/// never a client library in the desktop process.
pub trait Provider: fmt::Debug + Send + Sync {
    /// Apply an audio change.
    fn audio(&self, action: &AudioAction, runner: &dyn Runner, root: &Path) -> Outcome;

    /// Apply a network change.
    fn network(&self, action: &NetAction, runner: &dyn Runner, root: &Path) -> Outcome;

    /// Apply a Bluetooth change.
    fn bluetooth(&self, action: &BtAction, runner: &dyn Runner, root: &Path) -> Outcome;

    /// Apply an authentication action that is a single plan (verification).
    fn auth(&self, action: &AuthAction, runner: &dyn Runner, root: &Path) -> Outcome;

    /// Apply a bounded reload of an installed system service.
    fn service(&self, action: &ServiceAction, runner: &dyn Runner, root: &Path) -> Outcome {
        providers::perform(&SystemAction::Service(*action), runner, root)
    }

    /// Ask the sound server for the default sink's level.
    ///
    /// # Errors
    /// A sentence naming the missing tool, failed tool or unreadable answer.
    fn audio_level(&self, runner: &dyn Runner) -> Result<AudioLevel, String>;

    /// Ask BlueZ whether the default controller is powered on.
    ///
    /// # Errors
    /// A sentence naming the missing tool, failed tool or unreadable answer.
    fn bt_powered(&self, runner: &dyn Runner) -> Result<bool, String>;
}

/// The production provider: the base tools through the supplied runner.
#[derive(Debug, Clone, Copy, Default)]
pub struct ToolProvider;

impl Provider for ToolProvider {
    fn audio(&self, action: &AudioAction, runner: &dyn Runner, root: &Path) -> Outcome {
        providers::perform(&SystemAction::Audio(*action), runner, root)
    }

    fn network(&self, action: &NetAction, runner: &dyn Runner, root: &Path) -> Outcome {
        providers::perform(&SystemAction::Net(action.clone()), runner, root)
    }

    fn bluetooth(&self, action: &BtAction, runner: &dyn Runner, root: &Path) -> Outcome {
        providers::perform(&SystemAction::Bluetooth(action.clone()), runner, root)
    }

    fn auth(&self, action: &AuthAction, runner: &dyn Runner, root: &Path) -> Outcome {
        providers::perform(&SystemAction::Auth(action.clone()), runner, root)
    }

    fn audio_level(&self, runner: &dyn Runner) -> Result<AudioLevel, String> {
        providers::audio_level(runner).map(|(volume, muted)| AudioLevel { volume, muted })
    }

    fn bt_powered(&self, runner: &dyn Runner) -> Result<bool, String> {
        providers::bt_powered(runner)
    }
}

/// The state shared by every clone of a [`FakeProvider`].
#[derive(Debug, Default)]
struct FakeState {
    audio: Mutex<Option<AudioLevel>>,
    bluetooth: Mutex<Option<bool>>,
    outcomes: Mutex<VecDeque<Outcome>>,
    calls: Mutex<Vec<String>>,
}

/// A provider that answers without running a tool.
///
/// Cloning is cheap and shares the call record, which is what lets a test hand
/// one clone to [`crate::state::Sysd`] and inspect the other afterwards.
#[derive(Debug, Clone, Default)]
pub struct FakeProvider {
    state: Arc<FakeState>,
}

impl FakeProvider {
    /// A fake with no scripted outcome and no measured values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Script the next audio measurement.
    #[must_use]
    pub fn with_audio_level(self, volume: u8, muted: bool) -> Self {
        *self
            .state
            .audio
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(AudioLevel { volume, muted });
        self
    }

    /// Script the next Bluetooth power answer.
    #[must_use]
    pub fn with_bt_powered(self, powered: bool) -> Self {
        *self
            .state
            .bluetooth
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(powered);
        self
    }

    /// Queue one outcome for the next action, whatever its family.
    #[must_use]
    pub fn with_outcome(self, outcome: Outcome) -> Self {
        self.state
            .outcomes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push_back(outcome);
        self
    }

    /// Every action this fake was asked to perform, rendered with the same
    /// secret-redacting `Debug` the production types use.
    #[must_use]
    pub fn calls(&self) -> Vec<String> {
        self.state
            .calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn record(&self, action: &SystemAction) -> Outcome {
        // The record is a log seam, so it uses the type's own redacting
        // rendering. A password or Wi-Fi passphrase must never be copied into a
        // provider call record by hand.
        self.state
            .calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(format!("{action:?}"));
        self.state
            .outcomes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
            .unwrap_or_else(|| Outcome {
                applied: true,
                lines: vec![
                    format!("action={}.{}", action.family(), action.token()),
                    "applied=true".to_owned(),
                ],
            })
    }
}

impl Provider for FakeProvider {
    fn audio(&self, action: &AudioAction, _runner: &dyn Runner, _root: &Path) -> Outcome {
        self.record(&SystemAction::Audio(*action))
    }

    fn network(&self, action: &NetAction, _runner: &dyn Runner, _root: &Path) -> Outcome {
        self.record(&SystemAction::Net(action.clone()))
    }

    fn bluetooth(&self, action: &BtAction, _runner: &dyn Runner, _root: &Path) -> Outcome {
        self.record(&SystemAction::Bluetooth(action.clone()))
    }

    fn auth(&self, action: &AuthAction, _runner: &dyn Runner, _root: &Path) -> Outcome {
        self.record(&SystemAction::Auth(action.clone()))
    }

    fn service(&self, action: &ServiceAction, _runner: &dyn Runner, _root: &Path) -> Outcome {
        self.record(&SystemAction::Service(*action))
    }

    fn audio_level(&self, _runner: &dyn Runner) -> Result<AudioLevel, String> {
        self.state
            .audio
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .ok_or_else(|| "the fake has no audio measurement".to_owned())
    }

    fn bt_powered(&self, _runner: &dyn Runner) -> Result<bool, String> {
        self.state
            .bluetooth
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .ok_or_else(|| "the fake has no Bluetooth state".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::testing::RecordingRunner;

    #[test]
    fn the_fake_answers_audio_and_bluetooth_without_running_anything() {
        let provider = FakeProvider::new()
            .with_audio_level(42, true)
            .with_bt_powered(true);
        let runner = RecordingRunner::succeeding(0);

        assert_eq!(
            provider.audio_level(&runner),
            Ok(AudioLevel {
                volume: 42,
                muted: true
            })
        );
        assert_eq!(provider.bt_powered(&runner), Ok(true));
        assert!(
            runner.calls().is_empty(),
            "a fake provider must not spawn a tool: {:?}",
            runner.calls()
        );
    }

    #[test]
    fn the_fake_covers_the_three_c1_families() {
        let provider = FakeProvider::new();
        let runner = RecordingRunner::succeeding(0);
        let root = Path::new("/tmp/litric-provider-contract");

        let audio = provider.audio(&AudioAction::Mute(true), &runner, root);
        let network = provider.network(&NetAction::Reconfigure("eth0".to_owned()), &runner, root);
        let bluetooth = provider.bluetooth(&BtAction::Power(true), &runner, root);

        for outcome in [audio, network, bluetooth] {
            assert!(outcome.applied, "{:?}", outcome.lines);
            assert!(
                outcome.lines.iter().any(|l| l == "applied=true"),
                "{:?}",
                outcome.lines
            );
        }
        assert_eq!(provider.calls().len(), 3);
    }

    #[test]
    fn a_provider_call_record_never_contains_a_password() {
        const SENTINEL: &str = "S3NT1NEL-provider-password";
        let provider = FakeProvider::new();
        let runner = RecordingRunner::succeeding(0);
        let outcome = provider.auth(
            &AuthAction::Verify {
                user: "alice".to_owned(),
                password: SENTINEL.to_owned(),
            },
            &runner,
            Path::new("/tmp/litric-provider-contract"),
        );

        assert!(outcome.applied, "{:?}", outcome.lines);
        let recorded = provider.calls().join("\n");
        assert!(
            !recorded.contains(SENTINEL),
            "the provider call record leaked the password: {recorded}"
        );
        assert!(
            recorded.contains("<redacted>"),
            "the record must show that a secret was removed: {recorded}"
        );
    }

    #[test]
    fn the_tool_provider_still_runs_the_audited_plan() {
        let provider = ToolProvider;
        let runner = RecordingRunner::succeeding(0);
        let outcome = provider.audio(
            &AudioAction::Volume(40),
            &runner,
            Path::new("/tmp/litric-provider-contract"),
        );

        assert!(outcome.applied, "{:?}", outcome.lines);
        assert_eq!(
            runner.calls(),
            vec![(
                providers::WPCTL.to_owned(),
                vec![
                    "set-volume".to_owned(),
                    "--limit".to_owned(),
                    "1.0".to_owned(),
                    "@DEFAULT_AUDIO_SINK@".to_owned(),
                    "0.40".to_owned()
                ]
            )]
        );
    }
}
