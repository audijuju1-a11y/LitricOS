//! The lock screen: Liquid Glass.
//!
//! # What this is
//!
//! The session's lock **surface**, the half `docs/design/wave1/04-authentication.md`
//! §8 ticket 5 names as missing. The verifier has been real since 2026-10-12
//! (`litric-sysd` answers `AUTH VERIFY` through the distribution's own
//! `unix_chkpwd`); what did not exist was a screen that hides a session and
//! comes back. This module is that screen, and it changes nothing about how a
//! password is checked: the shell hands the secret to the caller that owns the
//! socket, the daemon runs the helper, and the answer decides whether this
//! surface draws another frame.
//!
//! # The design
//!
//! Ported from the prototype at `build/uiref/litricos-ui/screens/lock/liquid`:
//! three colour fields drifting behind a glass clock whose digits carry a glow
//! and a bloom, a breathing scale, liquid-glass widgets, and a customize sheet
//! whose controls are **presets rather than numbers** — the user picks "ฟุ้งฝัน"
//! or "ฝ้าเข้ม", not a glow of 70 and a blur of 50. Every preset's real value
//! lives in the tables below, so tuning the look is one edit in one place.
//!
//! Three things about the port are decisions rather than transcription:
//!
//! * **The widgets show measured state.** The prototype's cards are a song, an
//!   appointment and a battery; this machine has no music player and no
//!   calendar, and inventing a song title would be exactly the kind of
//!   fabricated data the project refuses. The same three slots carry the
//!   battery, the network and the uptime — numbers that come from
//!   `/proc` and `/sys` through [`crate::shell::SystemFacts`].
//! * **The colour fields are radial washes, not blurred circles.** The
//!   prototype blurs a solid circle by 90 px with `filter: blur(90px)`;
//!   [`Canvas`] has no filter, and a 90 px blur of a full-screen layer costs
//!   more than a frame. [`Canvas::radial_wash`] is the software rasteriser's
//!   equivalent, with a smoothstep falloff so no ring shows where it ends.
//! * **The glow and bloom are drawn as passes, not shadows.** A stroke
//!   typeface has no `text-shadow`, so the glow is the same digits drawn again
//!   at a heavier stroke and falling alpha in eight directions, and the bloom
//!   is an accent wash behind them. Both are scaled by the preset, so the
//!   control does what it says.

use std::rc::Rc;

use crate::canvas::{rgb, with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::shell::Hot;
use crate::ui::{self, State, Tokens, Tone};

// ---------------------------------------------------------------------------
// The secret
// ---------------------------------------------------------------------------

/// A password, in the one place it exists as plain text.
///
/// The type exists for its [`std::fmt::Debug`]: the shell hands the secret to
/// the caller that owns the daemon's socket, and that caller builds an
/// [`crate::shell::Effect`]. Effects derive `Debug` and are printed by
/// diagnostics, so a `String` here would put the user's password into a status
/// file the moment anything logged one. This prints the length and nothing
/// else, which is the same rule `docs/design/wave1/04-authentication.md` §5 U5
/// applies to the wire and the audit trail.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Take a secret from what the user typed.
    #[must_use]
    pub fn new(text: &str) -> Self {
        Self(text.to_owned())
    }

    /// Whether anything was typed at all. An empty secret is not submitted:
    /// `unix_chkpwd` runs with `nonull`, so an empty password cannot verify.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many bytes were typed. Never the bytes themselves.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The wire form `AUTH VERIFY` takes: lowercase hex of the UTF-8 bytes.
    ///
    /// The same encoding the daemon decodes with, so the secret travels as one
    /// unbroken token and never reaches an argument vector or a log line.
    #[must_use]
    pub fn hex(&self) -> String {
        let mut out = String::with_capacity(self.0.len().saturating_mul(2));
        for b in self.0.as_bytes() {
            out.push(char::from_digit(u32::from(b >> 4), 16).unwrap_or('0'));
            out.push(char::from_digit(u32::from(b & 0x0F), 16).unwrap_or('0'));
        }
        out
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret(<{} bytes, redacted>)", self.0.len())
    }
}

// ---------------------------------------------------------------------------
// Presets
// ---------------------------------------------------------------------------

/// The colour fields behind the lock: base colour plus three drifting ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wall {
    /// Teal, blue and violet over near-black.
    #[default]
    Aurora,
    /// Orange, pink and violet.
    Sunset,
    /// Blue, indigo and cyan.
    Ocean,
    /// Greys: the theme at its least colourful.
    Mono,
}

impl Wall {
    /// The order the customize sheet offers them in.
    pub const ALL: [Self; 4] = [Self::Aurora, Self::Sunset, Self::Ocean, Self::Mono];

    /// The palette: base, then the three field colours.
    ///
    /// Content, not tokens: these four palettes *are* the wallpapers, the same
    /// way `desktop.rs` owns the desktop's scene. Nothing else in the shell
    /// reads them and no component draws a colour from here.
    #[must_use]
    pub const fn palette(self) -> (u32, u32, u32, u32) {
        match self {
            Self::Aurora => (0x000A_1117, 0x0037_6A67, 0x0047_5D78, 0x0076_6277),
            Self::Sunset => (0x0017_0B12, 0x00FF_7A59, 0x00FF_3D77, 0x007A_3CFF),
            Self::Ocean => (0x0006_121F, 0x002B_B0FF, 0x002D_5BFF, 0x0019_E0C0),
            Self::Mono => (0x000B_0C0E, 0x008A_8F98, 0x004A_4F58, 0x00C9_CED6),
        }
    }

    /// The name the sheet shows.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Aurora => "Aurora",
            Self::Sunset => "Sunset",
            Self::Ocean => "Ocean",
            Self::Mono => "Mono",
        }
    }

    /// The colour its swatch button shows.
    #[must_use]
    pub const fn swatch(self) -> u32 {
        self.palette().1
    }

    /// From a sheet index, falling back to the first.
    #[must_use]
    fn from_index(i: u8) -> Self {
        Self::ALL
            .get(usize::from(i))
            .copied()
            .unwrap_or(Self::Aurora)
    }

    /// Its index in [`Wall::ALL`].
    #[must_use]
    fn index(self) -> u8 {
        let mut i = 0u8;
        while i < Self::ALL.len() as u8 {
            if let Some(w) = Self::ALL.get(usize::from(i)) {
                if *w == self {
                    return i;
                }
            }
            i = i.saturating_add(1);
        }
        0
    }
}

/// How the clock's digits are filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Digits {
    /// Translucent white: the glass look the screen is named for.
    #[default]
    Glass,
    /// Opaque white.
    Solid,
    /// The accent colour.
    Tint,
}

impl Digits {
    /// The order the sheet offers them in.
    pub const ALL: [Self; 3] = [Self::Glass, Self::Solid, Self::Tint];

    /// The name the sheet shows.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Glass => "Glass",
            Self::Solid => "Solid",
            Self::Tint => "Tint",
        }
    }

    /// From a sheet index, falling back to the first.
    #[must_use]
    fn from_index(i: u8) -> Self {
        Self::ALL
            .get(usize::from(i))
            .copied()
            .unwrap_or(Self::Glass)
    }

    /// Its index in [`Digits::ALL`].
    #[must_use]
    fn index(self) -> u8 {
        let mut i = 0u8;
        while i < Self::ALL.len() as u8 {
            if let Some(d) = Self::ALL.get(usize::from(i)) {
                if *d == self {
                    return i;
                }
            }
            i = i.saturating_add(1);
        }
        0
    }
}

/// One preset: the label, the numbers, and nothing else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset<const N: usize> {
    /// What the button says.
    pub label: &'static str,
    /// The values, in the order the sheet documents them.
    pub values: [f32; N],
}

/// The glow and bloom presets, as (glow, bloom) on 0–100.
pub const LIGHT: [Preset<2>; 4] = [
    Preset {
        label: "Off",
        values: [0.0, 0.0],
    },
    Preset {
        label: "Soft",
        values: [20.0, 25.0],
    },
    Preset {
        label: "Standard",
        values: [40.0, 45.0],
    },
    Preset {
        label: "Dreamy",
        values: [70.0, 80.0],
    },
];

/// The motion presets, as (animated, speed percent).
pub const MOTION: [Preset<2>; 4] = [
    Preset {
        label: "Still",
        values: [0.0, 100.0],
    },
    Preset {
        label: "Slow",
        values: [1.0, 50.0],
    },
    Preset {
        label: "Normal",
        values: [1.0, 100.0],
    },
    Preset {
        label: "Fast",
        values: [1.0, 160.0],
    },
];

/// The glass presets, as (blur px, tint percent, specular, edge).
pub const GLASS: [Preset<4>; 3] = [
    Preset {
        label: "Clear",
        values: [8.0, 6.0, 30.0, 70.0],
    },
    Preset {
        label: "Frosted",
        values: [28.0, 12.0, 22.0, 60.0],
    },
    Preset {
        label: "Heavy",
        values: [50.0, 24.0, 16.0, 45.0],
    },
];

/// The corner-radius presets, in logical pixels.
pub const CURVE: [Preset<1>; 3] = [
    Preset {
        label: "Soft",
        values: [16.0],
    },
    Preset {
        label: "Standard",
        values: [28.0],
    },
    Preset {
        label: "Round",
        values: [40.0],
    },
];

/// The clock size presets, in logical pixels.
pub const SIZE: [Preset<1>; 3] = [
    Preset {
        label: "Small",
        values: [170.0],
    },
    Preset {
        label: "Medium",
        values: [220.0],
    },
    Preset {
        label: "Large",
        values: [260.0],
    },
];

/// The clock stroke-weight presets, as the typeface's own fractions.
///
/// The prototype names CSS weights (200/300/500); this typeface is a stroke
/// font whose weight is a fraction of the size, so the presets map onto the
/// weights the rest of the interface already uses plus one lighter than
/// [`crate::font::WEIGHT_REGULAR`]. Inventing a fourth number would put a value
/// outside the type system, which is the thing the design lint exists to stop.
pub const WEIGHT_PRESETS: [Preset<1>; 3] = [
    Preset {
        label: "Thin",
        values: [0.100],
    },
    Preset {
        label: "Normal",
        values: [crate::font::WEIGHT_REGULAR],
    },
    Preset {
        label: "Thick",
        values: [crate::font::WEIGHT_BOLD],
    },
];

/// The accents the sheet offers, as `0xRRGGBB`. Content, like the wallpapers.
pub const ACCENTS: [u32; 5] = [
    0x008C_F2B6,
    0x00FF_C857,
    0x00FF_8FA3,
    0x007C_C4FF,
    0x00C9_A7FF,
];

/// The number of switchable widgets.
pub const WIDGETS: usize = 3;

// ---------------------------------------------------------------------------
// The state
// ---------------------------------------------------------------------------

/// Which of the lock screen's own controls a hotspot is.
///
/// Everything the sheet needs is one of these three: a tab, a preset button in
/// a group, or a switch. Groups are numbered here rather than in the shell, so
/// the mapping from a click to a value lives with the values.
pub const GROUP_WALL: u8 = 0;
/// Digits: glass, solid or tint.
pub const GROUP_DIGITS: u8 = 1;
/// Accent colour.
pub const GROUP_ACCENT: u8 = 2;
/// Clock size.
pub const GROUP_SIZE: u8 = 3;
/// Clock stroke weight.
pub const GROUP_WEIGHT: u8 = 4;
/// Glow and bloom.
pub const GROUP_LIGHT: u8 = 5;
/// Background motion.
pub const GROUP_MOTION: u8 = 6;
/// Glass material.
pub const GROUP_GLASS: u8 = 7;
/// Corner radius.
pub const GROUP_CURVE: u8 = 8;

/// The switch indices of the widget row.
pub const SWITCH_BREATHE: u8 = 0;
/// The measured-state widget.
pub const SWITCH_SYSTEM: u8 = 1;
/// The network widget.
pub const SWITCH_NETWORK: u8 = 2;
/// The battery widget.
pub const SWITCH_BATTERY: u8 = 3;

/// Everything the customize sheet can change.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tweaks {
    /// Wallpaper.
    pub wall: Wall,
    /// Digit fill.
    pub digits: Digits,
    /// Index into [`ACCENTS`].
    pub accent: u8,
    /// Index into [`SIZE`].
    pub size: u8,
    /// Index into [`WEIGHT_PRESETS`].
    pub weight: u8,
    /// Index into [`LIGHT`].
    pub light: u8,
    /// Index into [`MOTION`].
    pub motion: u8,
    /// Index into [`GLASS`].
    pub glass: u8,
    /// Index into [`CURVE`].
    pub curve: u8,
    /// Whether the clock breathes.
    pub breathe: bool,
    /// Whether each widget is shown: system, network, battery.
    pub widgets: [bool; WIDGETS],
}

impl Default for Tweaks {
    /// The shipped look: a quiet Aurora field, crisp digits, subtle materials,
    /// no decorative animation, and the three useful system facts.
    fn default() -> Self {
        Self {
            wall: Wall::default(),
            // Quiet and readable at first boot; expressive presets remain in Customize.
            digits: Digits::Solid,
            accent: 0,
            size: 1,
            weight: 1,
            light: 0,
            motion: 0,
            glass: 0,
            curve: 1,
            breathe: false,
            widgets: [true; WIDGETS],
        }
    }
}

impl Tweaks {
    /// The accent colour.
    #[must_use]
    pub fn accent_color(self) -> u32 {
        ACCENTS
            .get(usize::from(self.accent))
            .copied()
            .unwrap_or(ACCENTS[0])
    }

    /// Glow strength, 0–100.
    #[must_use]
    pub fn glow(self) -> f32 {
        value(&LIGHT, self.light).first().copied().unwrap_or(0.0)
    }

    /// Bloom strength, 0–100.
    #[must_use]
    pub fn bloom(self) -> f32 {
        value(&LIGHT, self.light).get(1).copied().unwrap_or(0.0)
    }

    /// Whether the background moves.
    #[must_use]
    pub fn animating(self) -> bool {
        value(&MOTION, self.motion).first().copied().unwrap_or(1.0) >= 0.5
    }

    /// Speed as a fraction of the designed pace: 0.5, 1.0 or 1.6.
    #[must_use]
    pub fn speed(self) -> f32 {
        value(&MOTION, self.motion).get(1).copied().unwrap_or(100.0) / 100.0
    }

    /// Glass blur in pixels.
    #[must_use]
    pub fn blur(self) -> u32 {
        let px = value(&GLASS, self.glass).first().copied().unwrap_or(28.0);
        // A blur radius of zero is still a material; a negative one would be a
        // subtraction. Clamped, then rounded once, at the point of use.
        u32::try_from(px.round().clamp(0.0, 64.0) as i32).unwrap_or(1)
    }

    /// Glass tint as a 0–1 alpha.
    #[must_use]
    pub fn tint(self) -> f32 {
        value(&GLASS, self.glass).get(1).copied().unwrap_or(12.0) / 100.0
    }

    /// The specular highlight as a 0–1 alpha.
    #[must_use]
    pub fn spec(self) -> f32 {
        value(&GLASS, self.glass).get(2).copied().unwrap_or(22.0) / 100.0 * 0.35
    }

    /// The border hairline as a 0–1 alpha.
    #[must_use]
    pub fn edge(self) -> f32 {
        value(&GLASS, self.glass).get(3).copied().unwrap_or(60.0) / 100.0 * 0.5
    }

    /// Corner radius, from the curve presets.
    #[must_use]
    pub fn radius(self) -> f32 {
        value(&CURVE, self.curve).first().copied().unwrap_or(28.0)
    }

    /// The clock's designed size in logical pixels.
    #[must_use]
    pub fn clock_px(self) -> f32 {
        value(&SIZE, self.size).first().copied().unwrap_or(220.0)
    }

    /// The clock's stroke weight as a fraction of its size.
    #[must_use]
    pub fn clock_weight(self) -> f32 {
        value(&WEIGHT_PRESETS, self.weight)
            .first()
            .copied()
            .unwrap_or(font::WEIGHT_REGULAR)
    }

    /// The clock size for a screen, clamped so a big preset on a short display
    /// cannot run the digits into the widgets.
    #[must_use]
    pub fn clock_size(self, t: &Tokens, height: f32) -> f32 {
        t.px(self.clock_px()).min(height * 0.36)
    }

    /// The preset index a group is showing.
    #[must_use]
    pub fn option(self, group: u8) -> u8 {
        match group {
            GROUP_WALL => self.wall.index(),
            GROUP_DIGITS => self.digits.index(),
            GROUP_ACCENT => self.accent,
            GROUP_SIZE => self.size,
            GROUP_WEIGHT => self.weight,
            GROUP_LIGHT => self.light,
            GROUP_MOTION => self.motion,
            GROUP_GLASS => self.glass,
            _ => self.curve,
        }
    }

    /// Choose the `option`-th preset of `group`.
    ///
    /// A group or index this build does not know leaves the look untouched
    /// rather than clamping to something the user did not ask for.
    pub fn choose(&mut self, group: u8, option: u8) {
        match group {
            GROUP_WALL if usize::from(option) < Wall::ALL.len() => {
                self.wall = Wall::from_index(option);
            }
            GROUP_DIGITS if usize::from(option) < Digits::ALL.len() => {
                self.digits = Digits::from_index(option);
            }
            GROUP_ACCENT if usize::from(option) < ACCENTS.len() => self.accent = option,
            GROUP_SIZE if usize::from(option) < SIZE.len() => self.size = option,
            GROUP_WEIGHT if usize::from(option) < WEIGHT_PRESETS.len() => self.weight = option,
            GROUP_LIGHT if usize::from(option) < LIGHT.len() => self.light = option,
            GROUP_MOTION if usize::from(option) < MOTION.len() => self.motion = option,
            GROUP_GLASS if usize::from(option) < GLASS.len() => self.glass = option,
            GROUP_CURVE if usize::from(option) < CURVE.len() => self.curve = option,
            // A group or an index this build does not know leaves the look
            // alone rather than clamping to a value nobody asked for.
            _ => {}
        }
    }

    /// Flip the `index`-th switch. An index this build does not know is
    /// nothing, not the first switch.
    pub fn toggle(&mut self, index: u8) {
        match index {
            SWITCH_BREATHE => self.breathe = !self.breathe,
            _ => {
                let Some(slot) = self.widgets.get_mut(usize::from(index.saturating_sub(1))) else {
                    return;
                };
                *slot = !*slot;
            }
        }
    }
}

/// One preset's values, or the first preset's when the index is out of range.
fn value<const N: usize>(presets: &[Preset<N>], index: u8) -> [f32; N] {
    presets
        .get(usize::from(index))
        .or_else(|| presets.first())
        .map(|p| p.values)
        .unwrap_or([0.0; N])
}

/// What the screen is asking of the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stage {
    /// The clock, the widgets and the hint. One key or click moves on.
    #[default]
    Idle,
    /// The password field is out.
    Entry,
}

/// What happened to the last attempt.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Verdict {
    /// Nothing attempted yet.
    #[default]
    Idle,
    /// The password is with the daemon.
    Checking,
    /// The daemon refused, and said why.
    Failed(String),
    /// This machine cannot check a password, so this surface cannot unlock and
    /// says so rather than collecting a secret it could never use.
    NoVerifier(String),
}

/// Which sheet tab is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// Wallpaper, digits, accent, size, weight.
    #[default]
    Look,
    /// Glow, bloom, motion, breathing.
    Effects,
    /// Glass material and corner radius.
    Glass,
    /// Which widgets are shown.
    Widgets,
}

impl Tab {
    /// The tabs, in order.
    pub const ALL: [Self; 4] = [Self::Look, Self::Effects, Self::Glass, Self::Widgets];

    /// The name on the tab.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Look => "Look",
            Self::Effects => "Effects",
            Self::Glass => "Glass",
            Self::Widgets => "Widgets",
        }
    }

    /// Its index, for the hotspot.
    #[must_use]
    pub const fn index(self) -> u8 {
        match self {
            Self::Look => 0,
            Self::Effects => 1,
            Self::Glass => 2,
            Self::Widgets => 3,
        }
    }

    /// From a hotspot index, falling back to the first tab.
    #[must_use]
    pub fn from_index(i: u8) -> Self {
        Self::ALL.get(usize::from(i)).copied().unwrap_or(Self::Look)
    }
}

/// The lock screen's state.
///
/// Held by [`crate::shell::Shell`] and drawn by [`draw`]. The password lives
/// here while it is being typed and leaves as a [`Secret`]; the plain text is
/// never mirrored anywhere else in the shell.
#[derive(Debug, Clone, Default)]
pub struct LockState {
    /// Whether the session is locked. While true, nothing else is drawn and
    /// nothing else receives input.
    pub locked: bool,
    /// Whether the password field is out.
    pub stage: Stage,
    /// What has been typed, in the clear, for as long as it takes to submit.
    password: String,
    /// What the last attempt did.
    pub verdict: Verdict,
    /// Whether the customize sheet is open.
    pub edit: bool,
    /// Which sheet tab is showing.
    pub tab: Tab,
    /// The look.
    pub tweaks: Tweaks,
    /// The user the password is checked against, from logind's session record.
    pub user: String,
    /// Milliseconds the colour fields have been drifting for. Advanced by
    /// [`LockState::advance`] only while the motion preset is running, so
    /// "Still" holds the fields where they are instead of snapping them home.
    pub flow_ms: u64,
    /// When the drift was last advanced, in the caller's timebase.
    last_ms: u64,
}

impl LockState {
    /// How long one trip round the first field's keyframes takes, in
    /// milliseconds, at the current speed. The second and third are slower, as
    /// the design specifies.
    #[must_use]
    fn period_ms(&self) -> u64 {
        let speed = self.tweaks.speed();
        let base = 28_000.0 / speed.max(0.01);
        u64::try_from(base.round() as i64).unwrap_or(28_000)
    }

    /// Move the drift on by however long passed. Returns whether an animation
    /// frame is due so a still lock sleeps between clock changes.
    ///
    /// Called from the shell's `tick`, with the same monotonic clock the rest
    /// of the shell uses. A first call (or one after a pause) closes the gap
    /// rather than replaying it: a lock screen that had been asleep for an hour
    /// must not sweep its fields through an hour of keyframes in one frame.
    pub fn advance(&mut self, now_ms: u64) -> bool {
        let elapsed = now_ms.saturating_sub(self.last_ms);
        self.last_ms = now_ms;
        if !self.locked || !self.tweaks.animating() || elapsed > 1_000 {
            return false;
        }
        let before = self.flow_ms;
        self.flow_ms = self.flow_ms.saturating_add(elapsed);
        // The drift moves at ~30 Hz, so only a new 33 ms bucket is worth a
        // frame. The division is the quantization and is deliberate;
        // `checked_rem`-free `checked_div` states that without tripping the
        // restriction lint, which exists to catch divisions nobody meant.
        const FLOW_FRAME_MS: u64 = 33;
        self.flow_ms.checked_div(FLOW_FRAME_MS) != before.checked_div(FLOW_FRAME_MS)
    }

    /// The drift's phase, 0.0 to 1.0, for the field that laps in `period_ms`.
    #[must_use]
    pub fn phase(&self, period_ms: u64) -> f32 {
        let period = period_ms.max(1);
        let at = self.flow_ms.checked_rem(period).unwrap_or(0);
        at as f32 / period as f32
    }

    /// Where the three fields are, as (offset x, offset y, scale) per field.
    ///
    /// The keyframes are the prototype's CSS: field A laps in 28 s, B in 34 s,
    /// C in 40 s, each with the three positions its `@keyframes` names. Easing
    /// between them is a smoothstep per segment, which is what `ease-in-out`
    /// looks like at four keyframes.
    #[must_use]
    pub fn flow(&self) -> [(f32, f32, f32); 3] {
        let period = self.period_ms();
        [
            sample(&FLOW_A, self.phase(period)),
            sample(
                &FLOW_B,
                self.phase(period.saturating_mul(34).checked_div(28).unwrap_or(34)),
            ),
            sample(
                &FLOW_C,
                self.phase(period.saturating_mul(40).checked_div(28).unwrap_or(40)),
            ),
        ]
    }

    /// Bring the password field out.
    pub fn wake(&mut self) {
        if self.verdict == Verdict::Idle || matches!(self.verdict, Verdict::Failed(_)) {
            self.stage = Stage::Entry;
        }
    }

    /// Put the password field away and forget what was typed.
    pub fn back(&mut self) {
        self.stage = Stage::Idle;
        self.password.clear();
    }

    /// Type into the field, ignoring control characters.
    pub fn push(&mut self, text: &str) {
        if self.stage != Stage::Entry {
            return;
        }
        // A ceiling, because an unbounded buffer fed by a held key is a memory
        // leak with a keyboard attached. 128 is longer than any password this
        // build's helper will accept, so nothing real is truncated.
        const MAX: usize = 128;
        for c in text.chars().filter(|c| !c.is_control()) {
            if self.password.chars().count() >= MAX {
                break;
            }
            self.password.push(c);
        }
    }

    /// Delete the last character typed.
    pub fn backspace(&mut self) {
        self.password.pop();
    }

    /// How many characters have been typed, for the dot row.
    #[must_use]
    pub fn typed(&self) -> usize {
        self.password.chars().count()
    }

    /// Take the password out of the shell.
    ///
    /// The buffer is cleared as it is taken: the secret exists in exactly one
    /// place at a time, and the only caller is the one that can hand it to the
    /// daemon.
    pub fn take_secret(&mut self) -> Secret {
        let secret = Secret::new(&self.password);
        self.password.clear();
        secret
    }

    /// The last thing said about an attempt, for the status line.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match &self.verdict {
            Verdict::Idle | Verdict::Checking => None,
            Verdict::Failed(s) | Verdict::NoVerifier(s) => Some(s.as_str()),
        }
    }
}

/// The colour fields' keyframes: `(at, dx, dy, scale)`, from the prototype's
/// `@keyframes flowA/B/C`.
const FLOW_A: [(f32, f32, f32, f32); 4] = [
    (0.0, 0.0, 0.0, 1.0),
    (0.33, 140.0, 90.0, 1.12),
    (0.66, 50.0, 170.0, 0.94),
    (1.0, 0.0, 0.0, 1.0),
];
/// Field B's keyframes.
const FLOW_B: [(f32, f32, f32, f32); 4] = [
    (0.0, 0.0, 0.0, 1.0),
    (0.33, -130.0, 110.0, 0.95),
    (0.66, -40.0, -40.0, 1.1),
    (1.0, 0.0, 0.0, 1.0),
];
/// Field C's keyframes.
const FLOW_C: [(f32, f32, f32, f32); 4] = [
    (0.0, 0.0, 0.0, 1.0),
    (0.33, -110.0, -70.0, 1.08),
    (0.66, 120.0, -40.0, 0.96),
    (1.0, 0.0, 0.0, 1.0),
];

/// One field's offset and scale at a phase, eased between keyframes.
fn sample(track: &[(f32, f32, f32, f32); 4], phase: f32) -> (f32, f32, f32) {
    let t = phase.clamp(0.0, 1.0);
    for pair in track.windows(2) {
        let (Some(a), Some(b)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        if t <= b.0 || (b.0 >= 1.0 && t <= 1.0) {
            let span = b.0 - a.0;
            let local = if span <= f32::EPSILON {
                0.0
            } else {
                ((t - a.0) / span).clamp(0.0, 1.0)
            };
            let e = local * local * (3.0 - 2.0 * local);
            return (
                a.1 + (b.1 - a.1) * e,
                a.2 + (b.2 - a.2) * e,
                a.3 + (b.3 - a.3) * e,
            );
        }
    }
    (0.0, 0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

/// Everything the surface needs that the shell owns.
pub struct View<'a> {
    /// The resolved design tokens.
    pub t: &'a Tokens,
    /// The lock screen's state.
    pub state: &'a LockState,
    /// The clock, already formatted by the locale.
    pub clock: String,
    /// The date, already formatted by the locale.
    pub date: String,
    /// Display size in pixels.
    pub w: f32,
    /// Display height in pixels.
    pub h: f32,
    /// Battery percentage, when the machine has a battery.
    pub battery: Option<u8>,
    /// Network state: `wifi`, `ethernet`, `offline`.
    pub network: Option<String>,
    /// Seconds since boot.
    pub uptime: Option<f64>,
    /// Whether the daemon is reachable at all. When it is not, an attempt
    /// cannot be checked and the surface says so instead of collecting one.
    pub verifier: Option<String>,
}

/// The saturation every lock panel's glass pushes its backdrop by.
///
/// One number, used by every panel, so the material cannot drift between the
/// widget row and the password pill.
const GLASS_SATURATION: f32 = 1.05;

/// What the frame's background is drawn from.
///
/// The surface behind everything — the opaque base, the three drifting fields
/// and the bloom wash — moves less than a fifth of a pixel in one thirty-third
/// of a second. So the picture is the *same picture* for several frames in a
/// row, and re-deriving it every frame from keyframes that have moved by a
/// fifth of a pixel was most of what BUG-0018 measured: 3.1M of a locked
/// frame's 10.4M pixels.
///
/// This is the test for "has it changed". The fields are quantised to whole
/// device pixels and the scale to a 1/512 step — both finer than anything the
/// drift's own rate can show in one frame, and at most a pixel of movement
/// between the frames that share a key.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Key {
    /// Each field's offset in device pixels and its scale, ×512.
    flow: [(i32, i32, i32); 3],
    /// The four wall colours: base, then the three fields.
    palette: (u32, u32, u32, u32),
    /// Logical-to-device scale, ×4096, which every field's size is built from.
    scale: i32,
    /// The frame's size in pixels.
    size: (u32, u32),
    /// The clock size the wash above it is placed from, ×16.
    clock: i32,
    /// The bloom preset, whose value moves the wash's strength.
    light: u8,
}

impl Key {
    /// The key of the frame `v` is in.
    fn of(t: &Tokens, v: &View<'_>, tw: Tweaks, clock: f32, size: (u32, u32)) -> Self {
        let px = t.px(1.0);
        let flow = v.state.flow();
        Self {
            flow: [
                steady(flow.first().copied().unwrap_or((0.0, 0.0, 1.0)), px),
                steady(flow.get(1).copied().unwrap_or((0.0, 0.0, 1.0)), px),
                steady(flow.get(2).copied().unwrap_or((0.0, 0.0, 1.0)), px),
            ],
            palette: tw.wall.palette(),
            scale: (px * 4096.0).round() as i32,
            size,
            clock: (clock * 16.0).round() as i32,
            light: tw.light,
        }
    }

    /// The drift in the units [`field`] draws with: device pixels and a plain
    /// scale.
    fn flow(self) -> [(f32, f32, f32); 3] {
        self.flow.map(|(dx, dy, scale)| {
            (
                dx as f32 * FLOW_STEP,
                dy as f32 * FLOW_STEP,
                scale as f32 / SCALE_STEP,
            )
        })
    }
}

/// How far a field may move before its frame is a different frame, in device
/// pixels, and how finely its scale is kept.
///
/// Four pixels, and 1/128 of the scale -- which is about four pixels on the
/// edge of a field, since the scale multiplies a radius of roughly 550.
///
/// Not one pixel, which is what the drift moves in about a frame: a key that
/// changes every frame is a cache that never hits, and the fields would then be
/// derived again every tick -- the thing being fixed. Four is safe because of
/// what a field *is*: a radial wash whose falloff spans 550 px, so moving it
/// four pixels changes a covered pixel by well under one of the 255 levels it
/// is stored in. The drift's own rate is 170 px in 9.3 s, so a key lasts about
/// four ticks while a field is moving fastest and indefinitely when the easing
/// has it nearly still.
const FLOW_STEP: f32 = 4.0;
/// Steps of the scale per unit, as [`FLOW_STEP`] describes.
const SCALE_STEP: f32 = 128.0;

/// One field's keyframe offset, in whole [`FLOW_STEP`]s of device pixels.
fn steady((dx, dy, scale): (f32, f32, f32), px: f32) -> (i32, i32, i32) {
    (
        (dx * px / FLOW_STEP).round() as i32,
        (dy * px / FLOW_STEP).round() as i32,
        (scale * SCALE_STEP).round() as i32,
    )
}

/// The lock surface's memory between one frame and the next.
///
/// Two pictures, both of them the same picture for several ticks in a row:
///
/// * The **background**: the opaque base, the drifting fields and the bloom
///   wash, drawn over by everything else. Derived again only when [`Key`]
///   changes, and copied into the frame when it does not.
/// * The **glass backdrop**: that same picture blurred and saturated where the
///   panels need it, which is what the material samples when one is attached.
///   It is made from the lock's *own* pixels — the fields, not the desktop's
///   wallpaper, which is what a panel on this surface is actually refracting —
///   and derived once for all the panels together rather than once per panel
///   per frame. That is the "blur the union of the glass regions once" from
///   BUG-0018, and it is exact: the region contains every pixel each panel's own
///   blur would have read, so the truncated means come out the same.
///
/// Nothing here changes a pixel the uncached path drew, so a frame that misses
/// the cache is the frame the surface has always drawn.
#[derive(Debug, Clone, Default)]
pub struct Cache {
    /// The background and the key it was drawn at.
    key: Option<Key>,
    background: Option<Rc<Canvas>>,
    /// The blurred copy and the key it was made from.
    glass_key: Option<Key>,
    glass: Option<Rc<Canvas>>,
    /// The rectangles of `glass` that are blurred, one per panel.
    ///
    /// One region per panel rather than the union of them all: the panels of
    /// this surface do not overlap, and a union would make every panel that
    /// asks for a region blur the whole union again -- blurring what an earlier
    /// panel had already blurred, and paying for the same pixels twice.
    covered: Vec<Rect>,
}

impl Cache {
    /// Draw the background into `canvas`, from this cache when it is current.
    ///
    /// `clock` is the fitted clock size, which the wash above the digits is
    /// placed from; the key carries it quantised, because a wash that moved by
    /// a sixteenth of a pixel is not worth a second composition.
    fn paint(&mut self, canvas: &mut Canvas, v: &View<'_>, key: Key, clock: f32) {
        if self.key != Some(key) {
            let t = v.t;
            let (base, c1, c2, c3) = key.palette;
            let mut bg = Canvas::new(canvas.width(), canvas.height(), rgb(base));
            let flow = key.flow();
            field(
                &mut bg,
                v,
                flow.first().copied().unwrap_or((0.0, 0.0, 1.0)),
                -160.0,
                -120.0,
                760.0,
                c1,
                0.48,
            );
            field(
                &mut bg,
                v,
                flow.get(1).copied().unwrap_or((0.0, 0.0, 1.0)),
                700.0,
                -80.0,
                680.0,
                c2,
                0.38,
            );
            field(
                &mut bg,
                v,
                flow.get(2).copied().unwrap_or((0.0, 0.0, 1.0)),
                260.0,
                380.0,
                800.0,
                c3,
                0.28,
            );
            let tw = v.state.tweaks;
            let wash_y = t.px(84.0) + clock * 0.5;
            bg.radial_wash(
                v.w / 2.0,
                wash_y,
                t.px(760.0),
                rgb(0xFFFFFF),
                tw.bloom() / 100.0 * 0.22,
            );
            self.background = Some(Rc::new(bg));
            self.key = Some(key);
            // The glass was made from the picture this has just replaced.
            self.glass = None;
            self.glass_key = None;
            self.covered.clear();
        }
        if let Some(bg) = self.background.clone() {
            canvas.blit_from(&bg, Rect::new(0.0, 0.0, v.w, v.h));
        }
    }

    /// The blurred backdrop that covers `r`, or `None` when this canvas cannot
    /// use one and the caller should blur the panel itself.
    ///
    /// A partially repainted frame is refused: [`Canvas::blur_region`] writes
    /// only inside the clip, so the pixels it skipped would be left unblurred
    /// for whichever frame sampled them next. A lock frame is composited whole
    /// (it is always damage, and the whole surface changes), so this costs the
    /// surface nothing a person can see.
    fn blurred_for(
        &mut self,
        canvas: &Canvas,
        key: Key,
        r: Rect,
        blur: u32,
        saturation: f32,
    ) -> Option<Rc<Canvas>> {
        if blur == 0 || canvas.clip().is_some() {
            return None;
        }
        // Every painted pixel's value depends on source pixels no further away
        // than the blur's reach -- two box passes of `blur` reach `2 * blur` in
        // each direction, plus the rounding at the edge of each pass.
        let want = r.inflate(blur as f32 * 2.0 + 2.0);
        if self.glass_key != Some(key) {
            let source = self.background.as_ref()?;
            // Empty rather than a copy: only the pixels the panels ask for are
            // ever read out of it, and they are filled in below, from the
            // background, over the region the blur needs to read.
            self.glass = Some(Rc::new(Canvas::new(source.width(), source.height(), 0)));
            self.glass_key = Some(key);
            self.covered.clear();
        }
        if !self.covered.iter().any(|c| contains(c, &want)) {
            let source = self.background.clone()?;
            let region = absorb(&mut self.covered, want);
            let glass = Rc::get_mut(self.glass.as_mut()?)?;
            glass.blit_from(&source, region);
            glass.blur_region(region, blur, 2);
            glass.saturate_region(region, saturation);
            self.covered.push(region);
        }
        self.glass.clone()
    }
}

/// Whether `outer` contains `inner`.
fn contains(outer: &Rect, inner: &Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.right() >= inner.right()
        && outer.bottom() >= inner.bottom()
}

/// Whether two rectangles share a pixel.
fn overlaps(a: &Rect, b: &Rect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

/// Take `want` out of the covered list and hand back the region to blur for
/// it: `want` grown until it has swallowed every rectangle it touches.
///
/// A blur computed over one region is truncated at that region's edge, so two
/// overlapping regions blurred one after the other do not come out the same as
/// one region covering both -- measured on the widget row, where the cards sit
/// 16 px apart with a 56 px kernel between them, that was five levels of green
/// along the edge they share. Growing to the union keeps every painted pixel's
/// *whole* kernel inside the region it is blurred over, which is what makes a
/// panel's pixels the same whichever order the panels were drawn in, and
/// therefore the same whether this frame built the backdrop or reused it.
///
/// Growing can reach a rectangle the first pass did not touch, so it repeats
/// until a pass finds nothing new.
fn absorb(covered: &mut Vec<Rect>, want: Rect) -> Rect {
    let mut region = want;
    let mut pending = std::mem::take(covered);
    loop {
        let mut rest = Vec::with_capacity(pending.len());
        let mut grew = false;
        for c in pending {
            if overlaps(&c, &region) {
                region = union(region, c);
                grew = true;
            } else {
                rest.push(c);
            }
        }
        pending = rest;
        if !grew {
            break;
        }
    }
    *covered = pending;
    region
}

/// The smallest rectangle containing both.
fn union(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let right = a.right().max(b.right());
    let bottom = a.bottom().max(b.bottom());
    Rect::new(x, y, (right - x).max(0.0), (bottom - y).max(0.0))
}

/// Paint the backdrop behind one glass panel, and nothing else.
///
/// The cached blurred picture when there is one, and the live per-panel blur
/// when there is not. Both put the same pixels under the panel; they differ in
/// who pays for them: one blur for the whole surface, kept between frames,
/// against one blur per panel per frame.
fn glass_panel(
    canvas: &mut Canvas,
    r: Rect,
    radius: f32,
    blur: u32,
    saturation: f32,
    cache: &mut Cache,
    key: Key,
) {
    if let Some(blurred) = cache.blurred_for(canvas, key, r, blur, saturation) {
        let previous = canvas.backdrop_owned();
        if canvas.set_backdrop(Some(blurred)) {
            canvas.glass_backdrop(r, radius, blur, saturation);
            let _ = canvas.set_backdrop(previous);
            return;
        }
        let _ = canvas.set_backdrop(previous);
    }
    canvas.glass_backdrop(r, radius, blur, saturation);
}

/// Draw the lock surface over everything, and report what was drawn.
///
/// The first thing a frame does is lay down the background — the opaque base the
/// surface is cleared to, then the fields and the bloom wash. The surface is
/// opaque, so nothing that was on the screen before the lock is still in the
/// buffer: the "clear, then draw" order
/// `docs/design/wave1/04-authentication.md` §5 U7 requires, and the reason a
/// locked frame cannot show a window through a translucent panel.
pub fn draw(canvas: &mut Canvas, v: &View<'_>, cache: &mut Cache, hits: &mut Vec<(Rect, Hot)>) {
    let t = v.t;
    let (tw, radius) = (v.state.tweaks, t.px(v.state.tweaks.radius()));
    let accent = tw.accent_color();

    // 1-3. The opaque base, the three drifting fields and the bloom wash: the
    // picture everything else is drawn over. Kept between frames, because the
    // fields move a fifth of a pixel in one tick and re-deriving a picture that
    // did not change was most of what a locked frame cost (BUG-0018).
    let size = fit_clock(t, v, tw.clock_size(t, v.h), tw);
    let key = Key::of(t, v, tw, size, (canvas.width(), canvas.height()));
    cache.paint(canvas, v, key, size);

    // 4. The date, then the clock.
    let date_style = Style::new(t.px(20.0), with_alpha(rgb(0xFFFFFF), 0.78));
    centred(
        canvas,
        &v.date,
        v.w / 2.0,
        t.px(50.0) + date_style.size,
        &date_style,
    );
    clock(canvas, v, size, accent, tw);

    // 5. The widgets, at the height the clock size leaves.
    let widgets_y = (t.px(84.0) + size + t.px(8.0)).min(widget_top_limit(t, v.h));
    widgets(canvas, v, tw, widgets_y, radius, cache, key);

    // 6. The customize sheet, or the hint and the submit pill.
    if v.state.edit {
        sheet(canvas, v, tw, radius, cache, key, hits);
    } else {
        // The whole screen wakes the lock screen. It is registered **first**,
        // because a later hotspot wins a hit and the customize button is drawn
        // on top of this catcher: registered the other way round, pressing the
        // button would wake the password field instead of opening the sheet.
        if v.state.stage != Stage::Entry {
            hits.push((Rect::new(0.0, 0.0, v.w, v.h), Hot::LockWake));
        }
        hint(canvas, v, tw, radius, cache, key, hits);
        if v.verifier.is_some() {
            // Keep the account and next action visible at rest. The card
            // becomes a live password field after the first click or keypress.
            entry(canvas, v, tw, radius, cache, key, hits);
        } else if v.state.stage == Stage::Entry {
            // The daemon went away, or was never there. Collecting a password
            // that nothing can check is the failure this surface avoids.
            unavailable(canvas, v, tw, radius, cache, key);
        }
    }
}

/// How much of the preset size the typeface's digits are drawn at.
///
/// The presets are the design's CSS font sizes, and LitricOS Sans draws its
/// digits about 1.14x taller than the size says, so the same number would put
/// the numerals 250 px tall where the design's are 175. Scaling the *drawn*
/// size, and leaving the preset (and the layout that hangs off it) alone, puts
/// the digits where the design has them.
const CLOCK_GLYPH: f32 = 0.70;

/// The clock size that keeps the digits inside the screen.
///
/// The preset sizes are chosen for `HH:MM`. A locale that prints more than that
/// (a seconds field, a 12-hour suffix) would run past both edges of the screen
/// at the design's 220 px, so the size is reduced until the measured width fits
/// within 90% of the display. It only ever shrinks.
fn fit_clock(t: &Tokens, v: &View<'_>, size: f32, tw: Tweaks) -> f32 {
    let probe = Style {
        size: size * CLOCK_GLYPH,
        weight: tw.clock_weight(),
        tracking: -size * CLOCK_GLYPH * 0.03,
        color: rgb(0xFFFFFF),
        face: font::FontFace::Sans,
    };
    let width = font::measure(&v.clock, &probe);
    let room = v.w * 0.9;
    let _ = t;
    if width > room && width > 0.0 {
        size * room / width
    } else {
        size
    }
}

/// Where the widget row may not go past: the hint's own band at the bottom.
fn widget_top_limit(t: &Tokens, height: f32) -> f32 {
    height - t.px(200.0)
}

/// One drifting colour field.
///
/// Positioned from the prototype's element rectangle plus the keyframe offset,
/// with the scale applied about the field's own centre. The radius is larger
/// than the element because the prototype's 90 px blur spreads it; the wash's
/// smoothstep falloff is what stands in for that blur.
///
/// `flow` is the field's keyframe offset in *device* pixels and its scale, the
/// form [`Key`] keeps it in: quantising an offset that is then multiplied by
/// the token scale again would round twice, and the pixel a frame drew would
/// not be the pixel its key said.
#[allow(clippy::too_many_arguments)]
fn field(
    canvas: &mut Canvas,
    v: &View<'_>,
    (dx, dy, scale): (f32, f32, f32),
    x: f32,
    y: f32,
    size: f32,
    colour: u32,
    peak: f32,
) {
    let t = v.t;
    let (x, y, size) = (t.px(x), t.px(y), t.px(size));
    let (cx, cy) = (x + size / 2.0, y + size / 2.0);
    // The keyframe offsets move the field; the scale is applied about its own
    // centre, which is what the prototype's transform does. The radius is 72%
    // of the element rather than 50%, standing in for the 90 px blur the design
    // spreads it with.
    canvas.radial_wash(
        cx + dx,
        cy + dy,
        size * 0.72 * scale,
        rgb(colour & 0x00FF_FFFF),
        peak,
    );
}

/// The glass clock: a bloom wash, a glow, then the digits themselves.
///
/// The bloom is an accent wash behind the numerals, the glow is the same text
/// drawn at a heavier stroke and falling alpha in eight directions, and the
/// digits go on last. Drawing the crisp text first and the halo over it would
/// put the light in front of the glass, which is the one thing a bloom is not.
fn clock(canvas: &mut Canvas, v: &View<'_>, size: f32, accent: u32, tw: Tweaks) {
    let t = v.t;
    // Breathing: the design's ±1.4% over eight seconds, applied to the whole
    // block by scaling the size. A scale of exactly 1 when the switch is off,
    // so "off" is off rather than a smaller wobble.
    let breathe = if tw.breathe {
        let period = 8_000.0 / tw.speed().max(0.01);
        let phase = (v.state.flow_ms as f32 % period.max(1.0)) / period.max(1.0);
        // sin() rather than a keyframe walk: the design's 0/50/100% keyframes
        // *are* a sinusoid, and this is the same curve without a table.
        t.px(1.0) * 0.014 * (phase * std::f32::consts::TAU).sin()
    } else {
        0.0
    };
    let size = size * (1.0 + breathe);
    // Layout is in preset size; the digits are drawn smaller (see CLOCK_GLYPH)
    // and sit on the design's baseline, 24 px up from the preset's.
    let baseline = t.px(84.0) + size * 0.89;
    let size = size * CLOCK_GLYPH;
    let cx = v.w / 2.0;

    let fill = match tw.digits {
        Digits::Glass => with_alpha(rgb(0xFFFFFF), 0.62),
        Digits::Solid => rgb(0xFFFFFF),
        Digits::Tint => rgb(accent & 0x00FF_FFFF),
    };
    let glow_col = if tw.digits == Digits::Tint {
        rgb(accent & 0x00FF_FFFF)
    } else {
        rgb(0xFFFFFF)
    };

    // Bloom: a wide accent wash under the numerals, pulled from the preset.
    canvas.radial_wash(
        cx,
        baseline - size * 0.5,
        size * 1.6,
        rgb(accent & 0x00FF_FFFF),
        tw.bloom() / 100.0 * 0.30,
    );

    // Glow: one rasterisation of the same ink, blurred, instead of sixteen
    // offset copies of it. The preset still decides how much light there is, so
    // "Off" is still a single crisp pass.
    let glow = tw.glow() / 100.0;
    if glow > 0.01 {
        let style = Style {
            size,
            weight: tw.clock_weight(),
            tracking: -size * 0.03,
            color: with_alpha(glow_col, 1.0),
            face: font::FontFace::Sans,
        };
        glow_halo(canvas, &v.clock, cx, baseline, &style, glow_col, glow);
    }

    // The digits, last and once: everything before this pass is light, and a
    // second copy on top would double the ink of a translucent fill.
    let style = Style {
        size,
        weight: tw.clock_weight(),
        tracking: -size * 0.03,
        color: fill,
        face: font::FontFace::Sans,
    };
    let w = font::measure(&v.clock, &style);
    font::draw(canvas, &v.clock, cx - w / 2.0, baseline, &style);
}

/// The light the glow pattern carried, as a multiple of its preset.
///
/// The pattern this replaces drew eight copies of the digits at 0.10 of the
/// preset and eight more at half that, so the halo it produced held
/// `0.10 * 8 + 0.05 * 8 = 1.2` of the preset's light. A box blur of the ink
/// spreads the same light rather than adding to it, so this is what a blurred
/// mask has to be composited at to put that much light back.
const GLOW_ENERGY: f32 = 0.10 * 8.0 + 0.05 * 8.0;

/// The clock's glow: the digits' own ink, rasterised once into a small canvas,
/// blurred, and composited at the strength the preset asks for.
///
/// What this replaces: eight offset copies of the whole digit string at two
/// distances each, i.e. sixteen re-rasterisations of five glyphs on every
/// frame. Measured on a locked frame (BUG-0018), that was 5.3M of the frame's
/// 10.4M pixels and most of its time. A blur of one rasterisation is the halo
/// those offset copies approximate -- it is what a stencil blurred with a
/// Gaussian is, and the copies are a stencil sampled at sixteen points -- for
/// one pass of ink instead of sixteen.
///
/// The mask is drawn white on opaque black and blurred with the canvas's own
/// box blur, so the pixel's red channel *is* the halo's coverage there. The
/// blur keeps the light rather than adding to it, which is what makes
/// [`GLOW_ENERGY`] the right scale.
///
/// The mask is a canvas of its own, sized to the digits plus the blur's reach,
/// so nothing here depends on the screen and the cost follows the type rather
/// than the display.
fn glow_halo(
    canvas: &mut Canvas,
    text: &str,
    cx: f32,
    baseline: f32,
    style: &Style,
    colour: u32,
    glow: f32,
) {
    // Two box passes of two pixels: a spread of about four, which covers the
    // 2.6 px the outer copies reached and then some, and is the same soft edge
    // the pattern was reaching for.
    const BLUR: u32 = 2;
    const PASSES: u32 = 2;
    let width = font::measure(text, style);
    // The blur reads `BLUR` pixels each way per pass; the extra few keep the
    // ink away from the mask's edge, where the blur has no neighbours to
    // average and would darken the rim of the halo.
    let reach = (BLUR * PASSES * 2) as f32 + 4.0;
    let x0 = (cx - width / 2.0 - reach).floor();
    let y0 = (baseline - style.size - reach).floor();
    let x1 = (cx + width / 2.0 + reach).ceil();
    let y1 = (baseline + reach).ceil();
    let Some(w) = u32::try_from((x1 - x0) as i64).ok().filter(|n| *n > 0) else {
        return;
    };
    let Some(h) = u32::try_from((y1 - y0) as i64).ok().filter(|n| *n > 0) else {
        return;
    };
    // A stroke wider than the screen is a mis-set preset, not a glyph; the cap
    // keeps one frame from trying to allocate a canvas the machine does not
    // have. The halo is inside the screen anyway, so nothing is lost.
    let cap = canvas.width().saturating_mul(canvas.height());
    if w.saturating_mul(h) > cap {
        return;
    }

    let mut mask = Canvas::new(w, h, rgb(0x000000));
    font::draw(&mut mask, text, cx - width / 2.0 - x0, baseline - y0, style);
    mask.blur_region(Rect::new(0.0, 0.0, w as f32, h as f32), BLUR, PASSES);

    let scale = glow * GLOW_ENERGY;
    let (ox, oy) = (x0 as i64, y0 as i64);
    for y in 0..h {
        for x in 0..w {
            let Some(p) = mask.get(i64::from(x), i64::from(y)) else {
                continue;
            };
            // A blurred grey is its own coverage; the channels are equal by
            // construction, so the red one is the whole answer.
            let grey = f32::from(((p >> 16) & 0xFF) as u8) / 255.0;
            let cov = grey * scale;
            if cov > 0.0 {
                canvas.blend_px(
                    ox.saturating_add(i64::from(x)),
                    oy.saturating_add(i64::from(y)),
                    with_alpha(colour, cov),
                    1.0,
                );
            }
        }
    }
}

/// The widget row: the measured state of this machine, in glass cards.
#[allow(clippy::too_many_arguments)]
fn widgets(
    canvas: &mut Canvas,
    v: &View<'_>,
    tw: Tweaks,
    y: f32,
    radius: f32,
    cache: &mut Cache,
    key: Key,
) {
    let t = v.t;
    let card_w = t.px(300.0);
    let card_h = t.px(88.0);
    let gap = t.px(16.0);

    // Which cards are on, in the order of the switch indices.
    let shown: Vec<usize> = (0..WIDGETS)
        .filter(|i| tw.widgets.get(*i).copied().unwrap_or(false))
        .collect();
    if shown.is_empty() {
        return;
    }
    let total = card_w * shown.len() as f32 + gap * (shown.len().saturating_sub(1) as f32);
    let mut x = v.w / 2.0 - total / 2.0;
    for index in shown {
        let r = Rect::new(x, y, card_w, card_h);
        liquid(canvas, t, r, radius, tw, cache, key);
        let (icon, label, value) = widget_content(v, index);
        let chip = Rect::new(r.x + t.px(18.0), r.y + t.px(20.0), t.px(48.0), t.px(48.0));
        let accent = rgb(tw.accent_color() & 0x00FF_FFFF);
        match index {
            // First card: the accent disc with a dark glyph, as the music
            // card in the design.
            0 => {
                canvas.fill_circle(chip.center().0, chip.center().1, chip.w / 2.0, accent);
                crate::icons::draw(
                    canvas,
                    icon,
                    chip.inflate(-t.px(13.0)),
                    with_alpha(rgb(0x000000), 0.85),
                    1.0,
                );
            }
            // Second card: a quiet glass tile, as the calendar card.
            1 => {
                canvas.fill_round_rect(chip, t.px(12.0), with_alpha(rgb(0xFFFFFF), 0.18));
                crate::icons::draw(
                    canvas,
                    icon,
                    chip.inflate(-t.px(12.0)),
                    with_alpha(rgb(0xFFFFFF), 0.95),
                    1.0,
                );
            }
            // Third card: a progress ring when the machine has a battery to
            // measure; with none, the plain glyph rather than an empty ring.
            _ => match v.battery {
                Some(pct) => ring(canvas, t, chip, f32::from(pct.min(100)) / 100.0, accent),
                None => crate::icons::draw(
                    canvas,
                    icon,
                    chip.inflate(-t.px(8.0)),
                    with_alpha(rgb(0xFFFFFF), 0.92),
                    1.0,
                ),
            },
        }
        let label_style = Style::new(t.px(13.0), with_alpha(rgb(0xFFFFFF), 0.7));
        font::draw(
            canvas,
            label,
            r.x + t.px(82.0),
            r.y + t.px(36.0),
            &label_style,
        );
        let value_style = Style::new(t.px(22.0), rgb(0xFFFFFF));
        font::draw(
            canvas,
            &value,
            r.x + t.px(82.0),
            r.y + t.px(64.0),
            &value_style,
        );
        x += card_w + gap;
    }
}

/// A progress ring: a faint full track, and over it an arc from twelve o'clock
/// clockwise for `fraction` of the way round.
fn ring(canvas: &mut Canvas, t: &Tokens, r: Rect, fraction: f32, colour: u32) {
    let (cx, cy) = r.center();
    let width = t.px(5.0);
    let radius = r.w / 2.0 - width / 2.0;
    let track = crate::canvas::arc_points(cx, cy, radius, radius, 0.0, 360.0, 48);
    for pair in track.windows(2) {
        if let [a, b] = pair {
            canvas.stroke_line(a.0, a.1, b.0, b.1, width, with_alpha(rgb(0xFFFFFF), 0.22));
        }
    }
    let sweep = 360.0 * fraction.clamp(0.0, 1.0);
    if sweep > 1.0 {
        let steps = (sweep / 7.5).ceil().max(2.0) as u32;
        let arc = crate::canvas::arc_points(cx, cy, radius, radius, -90.0, -90.0 + sweep, steps);
        for pair in arc.windows(2) {
            if let [a, b] = pair {
                canvas.stroke_line(a.0, a.1, b.0, b.1, width, colour);
            }
        }
    }
}

/// One widget's icon, label and value — measured, or said to be unavailable.
fn widget_content(v: &View<'_>, index: usize) -> (crate::icons::Icon, &'static str, String) {
    match index {
        0 => (
            crate::icons::Icon::Cpu,
            "Uptime",
            v.uptime.map_or_else(
                || "\u{2014}".to_owned(),
                |s| {
                    let total = s.max(0.0) as u64;
                    format!(
                        "{}h {:02}m",
                        total.checked_div(3_600).unwrap_or(0),
                        total.checked_div(60).unwrap_or(0) % 60
                    )
                },
            ),
        ),
        1 => (
            crate::icons::Icon::Network,
            "Network",
            v.network
                .clone()
                .unwrap_or_else(|| "Unavailable".to_owned()),
        ),
        _ => (
            crate::icons::Icon::Battery,
            "Battery",
            v.battery
                .map_or_else(|| "\u{2014}".to_owned(), |p| format!("{p}%")),
        ),
    }
}

/// A liquid-glass panel: the backdrop blurred and saturated in place, then a
/// tint, a specular top edge and a hairline, all from the glass presets.
fn liquid(
    canvas: &mut Canvas,
    t: &Tokens,
    r: Rect,
    radius: f32,
    tw: Tweaks,
    cache: &mut Cache,
    key: Key,
) {
    glass_panel(canvas, r, radius, tw.blur(), GLASS_SATURATION, cache, key);
    canvas.fill_round_rect(r, radius, with_alpha(rgb(0xFFFFFF), tw.tint()));
    // The specular edge: three hairlines of falling alpha rather than a
    // gradient, because a gradient fill is a rectangle and this shape has
    // corners. It is the same trick `desktop::glass` uses.
    let inset = radius * 0.6;
    for (i, alpha) in [1.0_f32, 0.5, 0.25].into_iter().enumerate() {
        let y = r.y + t.px(0.5) + t.px(i as f32 * 1.5);
        canvas.fill_round_rect(
            Rect::new(r.x + inset, y, (r.w - inset * 2.0).max(0.0), t.px(1.0)),
            t.px(0.5),
            with_alpha(rgb(0xFFFFFF), tw.spec() * alpha),
        );
    }
    canvas.stroke_round_rect(r, radius, 1.0, with_alpha(rgb(0xFFFFFF), tw.edge()));
}

/// Text centred on `cx`, drawn at a baseline. Returns the width it took.
fn centred(canvas: &mut Canvas, text: &str, cx: f32, baseline: f32, style: &Style) -> f32 {
    let w = font::measure(text, style);
    font::draw(canvas, text, cx - w / 2.0, baseline, style);
    w
}

/// The idle hint and the customize button.
/// Whether a word of the hint is a key, which gets its own pill. Decided by
/// name: "Space" and "Enter" are both five letters, so a length test cannot.
fn is_key(word: &str) -> bool {
    matches!(word, "Space" | "Enter")
}

#[allow(clippy::too_many_arguments)]
fn hint(
    canvas: &mut Canvas,
    v: &View<'_>,
    tw: Tweaks,
    radius: f32,
    cache: &mut Cache,
    key: Key,
    hits: &mut Vec<(Rect, Hot)>,
) {
    let t = v.t;
    let hint_style = Style::new(t.px(16.0), with_alpha(rgb(0xFFFFFF), 0.8));
    let keys_style = Style::new(t.px(14.0), with_alpha(rgb(0xFFFFFF), 0.92));
    // The design's rows: the keys line centred 120 px above the bottom edge,
    // the Customize pill 92 px above it. When the password field is out it takes
    // the keys line's place, so the line is not drawn at all then.
    let y = v.h - t.px(114.0);
    let show_keys = v.state.stage != Stage::Entry && v.verifier.is_none();

    // "Press  Space  or  Enter" — the keys in their own pills, so the hint
    // reads as keys rather than as prose.
    let words = ["Press", "Space", "or", "Enter"];
    let widths: Vec<f32> = words
        .iter()
        .map(|w| {
            let style = if is_key(w) { &keys_style } else { &hint_style };
            font::measure(w, style) + t.px(if is_key(w) { 20.0 } else { 0.0 })
        })
        .collect();
    let gap = t.px(10.0);
    let total: f32 = widths.iter().sum::<f32>() + gap * 3.0;
    let mut x = v.w / 2.0 - total / 2.0;
    for (i, word) in words.iter().enumerate() {
        if !show_keys {
            break;
        }
        let w = widths.get(i).copied().unwrap_or(0.0);
        if is_key(word) {
            let pill = Rect::new(x, y - t.px(18.0), w, t.px(28.0));
            canvas.fill_round_rect(pill, t.px(8.0), with_alpha(rgb(0xFFFFFF), 0.10));
            canvas.stroke_round_rect(pill, t.px(8.0), 1.0, with_alpha(rgb(0xFFFFFF), 0.35));
            centred(
                canvas,
                word,
                pill.center().0,
                pill.y + t.px(19.0),
                &keys_style,
            );
        } else {
            centred(canvas, word, x + w / 2.0, y, &hint_style);
        }
        x += w + gap;
    }

    // The customize button, below the hint.
    let label = "Customize";
    let style = Style::new(t.px(16.0), rgb(0xFFFFFF));
    let w = font::measure(label, &style) + t.px(76.0);
    let r = Rect::new(v.w / 2.0 - w / 2.0, v.h - t.px(92.0), w, t.px(48.0));
    liquid(canvas, t, r, t.px(radius).min(r.h / 2.0), tw, cache, key);
    crate::icons::draw(
        canvas,
        crate::icons::Icon::Settings,
        Rect::new(r.x + t.px(20.0), r.y + t.px(14.0), t.px(20.0), t.px(20.0)),
        rgb(0xFFFFFF),
        1.0,
    );
    font::draw(canvas, label, r.x + t.px(48.0), r.y + t.px(30.0), &style);
    hits.push((r, Hot::LockCustomize));
}

/// The pill shown when this machine cannot check a password at all.
fn unavailable(
    canvas: &mut Canvas,
    v: &View<'_>,
    tw: Tweaks,
    radius: f32,
    cache: &mut Cache,
    key: Key,
) {
    let t = v.t;
    let w = t.px(560.0).min(v.w - t.px(40.0));
    let h = t.px(68.0);
    let r = Rect::new(v.w / 2.0 - w / 2.0, v.h - t.px(180.0), w, h);
    liquid(canvas, t, r, t.px(radius).min(h / 2.0), tw, cache, key);
    crate::icons::draw(
        canvas,
        crate::icons::Icon::Warning,
        Rect::new(r.x + t.px(20.0), r.y + t.px(22.0), t.px(24.0), t.px(24.0)),
        with_alpha(rgb(0xFFFFFF), 0.9),
        1.0,
    );
    let style = Style::new(t.px(15.0), with_alpha(rgb(0xFFFFFF), 0.9));
    let shown = ui::elide(
        v.state
            .message()
            .unwrap_or("No password can be checked here"),
        &style,
        r.w - t.px(72.0),
    );
    font::draw(canvas, &shown, r.x + t.px(56.0), r.y + t.px(42.0), &style);
}

/// The password pill: an avatar, the dot row, and the submit button.
#[allow(clippy::too_many_arguments)]
fn entry(
    canvas: &mut Canvas,
    v: &View<'_>,
    tw: Tweaks,
    radius: f32,
    cache: &mut Cache,
    key: Key,
    hits: &mut Vec<(Rect, Hot)>,
) {
    let t = v.t;
    let accent = tw.accent_color();
    let w = t.px(460.0).min(v.w - t.px(32.0));
    let h = t.px(80.0);
    let r = Rect::new(v.w / 2.0 - w / 2.0, v.h * 0.70 - h / 2.0, w, h);
    liquid(canvas, t, r, t.px(radius).min(h / 2.0), tw, cache, key);

    let avatar = Rect::new(r.x + t.px(12.0), r.y + t.px(16.0), t.px(48.0), t.px(48.0));
    canvas.fill_circle(
        avatar.center().0,
        avatar.center().1,
        avatar.h / 2.0,
        rgb(accent & 0x00FF_FFFF),
    );
    crate::icons::draw(
        canvas,
        crate::icons::Icon::User,
        avatar.inflate(-t.px(11.0)),
        with_alpha(rgb(0x000000), 0.8),
        1.0,
    );

    // The account is named above the password line so the unlock action is
    // discoverable without another keypress and the password stays masked.
    let field_x = avatar.right() + t.px(16.0);
    let account_style = Style::new(t.px(13.0), with_alpha(rgb(0xFFFFFF), 0.78));
    let account = format!("{} {}", crate::i18n::text("Unlock"), v.state.user);
    font::draw(canvas, &account, field_x, r.y + t.px(31.0), &account_style);

    // One dot per character typed, and an explicit placeholder when empty.
    if v.state.typed() == 0 {
        let style = Style::new(t.px(16.0), with_alpha(rgb(0xFFFFFF), 0.64));
        let prompt = if v.state.stage == Stage::Idle {
            "Click or press Enter to unlock"
        } else {
            "Enter password"
        };
        font::draw(canvas, prompt, field_x, r.y + t.px(59.0), &style);
    } else {
        let dot = t.px(5.0);
        let step = t.px(14.0);
        for i in 0..v.state.typed().min(28) {
            canvas.fill_circle(
                field_x + t.px(6.0) + step * i as f32,
                r.y + t.px(55.0),
                dot,
                rgb(0xFFFFFF),
            );
        }
    }

    // The lock opens directly into this focused field. A steady insertion
    // marker makes that state visible with both the keyboard and pointer.
    let caret_x = if v.state.typed() == 0 {
        field_x - t.px(2.0)
    } else {
        field_x + t.px(6.0) + t.px(14.0) * v.state.typed().min(28) as f32
    };
    if v.state.stage == Stage::Entry {
        canvas.stroke_line(
            caret_x,
            r.y + t.px(44.0),
            caret_x,
            r.y + t.px(63.0),
            t.px(1.0),
            rgb(accent & 0x00FF_FFFF),
        );
    }

    let submit = Rect::new(
        r.right() - t.px(57.0),
        r.center().1 - t.px(24.0),
        t.px(46.0),
        t.px(46.0),
    );
    // The submit is the one action on this surface, so it is the accent: the
    // same colour as the avatar and the digits' tint, and never a second hue.
    let state = if v.state.verdict == Verdict::Checking {
        State::Loading
    } else if matches!(v.state.verdict, Verdict::Failed(_)) {
        State::Error
    } else {
        State::Normal
    };
    canvas.fill_circle(
        submit.center().0,
        submit.center().1,
        submit.h / 2.0,
        with_alpha(
            rgb(accent & 0x00FF_FFFF),
            if state == State::Error { 0.5 } else { 1.0 },
        ),
    );
    crate::icons::draw(
        canvas,
        if state == State::Error {
            crate::icons::Icon::Warning
        } else {
            crate::icons::Icon::ChevronRight
        },
        submit.inflate(-t.px(13.0)),
        with_alpha(rgb(0x000000), 0.85),
        1.0,
    );
    if v.state.stage == Stage::Entry {
        hits.push((r, Hot::LockEntry));
        hits.push((submit, Hot::LockSubmit));
    }

    // What the last attempt did, above the pill. A refusal is named in the
    // daemon's own words, and this machine having no verifier is named as
    // itself rather than as a wrong password.
    if let Some(message) = v.state.message() {
        let style = Style::new(t.px(14.0), with_alpha(rgb(0xFFFFFF), 0.85));
        centred(canvas, message, v.w / 2.0, r.y - t.px(16.0), &style);
    }
}

// ---------------------------------------------------------------------------
// The customize sheet
// ---------------------------------------------------------------------------

/// One card in the sheet: a labelled row of preset buttons, or a switch.
struct Card {
    /// What the card is called.
    label: &'static str,
    /// What it holds.
    kind: CardKind,
}

/// The two kinds of card the sheet has.
enum CardKind {
    /// Preset buttons; `swatch` is a colour for the ones that show one.
    Options {
        /// Which group the buttons belong to.
        group: u8,
        /// The buttons.
        items: &'static [Choice],
    },
    /// A switch.
    Switch {
        /// Which switch it flips.
        index: u8,
    },
}

/// One preset button.
struct Choice {
    /// Its label, when it has words.
    label: &'static str,
    /// Its colour, when it shows one.
    swatch: Option<u32>,
}

/// A word-only preset button.
const fn word(label: &'static str) -> Choice {
    Choice {
        label,
        swatch: None,
    }
}

/// A colour-swatch preset button.
const fn swatch(colour: u32) -> Choice {
    Choice {
        label: "",
        swatch: Some(colour),
    }
}

/// The wallpaper buttons.
const WALL_CHOICES: [Choice; 4] = [
    swatch(Wall::Aurora.swatch()),
    swatch(Wall::Sunset.swatch()),
    swatch(Wall::Ocean.swatch()),
    swatch(Wall::Mono.swatch()),
];
/// The digit-fill buttons.
const DIGIT_CHOICES: [Choice; 3] = [word("Glass"), word("Solid"), word("Tint")];
/// The accent buttons.
const ACCENT_CHOICES: [Choice; 5] = [
    swatch(ACCENTS[0]),
    swatch(ACCENTS[1]),
    swatch(ACCENTS[2]),
    swatch(ACCENTS[3]),
    swatch(ACCENTS[4]),
];
/// The clock-size buttons.
const SIZE_CHOICES: [Choice; 3] = [word("S"), word("M"), word("L")];
/// The clock-weight buttons.
const WEIGHT_CHOICES: [Choice; 3] = [word("Thin"), word("Normal"), word("Thick")];
/// The glow-and-bloom buttons.
const LIGHT_CHOICES: [Choice; 4] = [word("Off"), word("Soft"), word("Standard"), word("Dreamy")];
/// The motion buttons.
const MOTION_CHOICES: [Choice; 4] = [word("Still"), word("Slow"), word("Normal"), word("Fast")];
/// The glass-material buttons.
const GLASS_CHOICES: [Choice; 3] = [word("Clear"), word("Frosted"), word("Heavy")];
/// The corner buttons.
const CURVE_CHOICES: [Choice; 3] = [word("Soft"), word("Standard"), word("Round")];

/// The cards for a tab.
fn cards(tab: Tab) -> Vec<Card> {
    /// One preset card.
    const fn options(label: &'static str, group: u8, items: &'static [Choice]) -> Card {
        Card {
            label,
            kind: CardKind::Options { group, items },
        }
    }
    /// One switch card.
    const fn switch(label: &'static str, index: u8) -> Card {
        Card {
            label,
            kind: CardKind::Switch { index },
        }
    }
    match tab {
        Tab::Look => vec![
            options("Wallpaper", GROUP_WALL, &WALL_CHOICES),
            options("Digits", GROUP_DIGITS, &DIGIT_CHOICES),
            options("Accent", GROUP_ACCENT, &ACCENT_CHOICES),
            options("Clock size", GROUP_SIZE, &SIZE_CHOICES),
            options("Weight", GROUP_WEIGHT, &WEIGHT_CHOICES),
        ],
        Tab::Effects => vec![
            options("Glow + bloom", GROUP_LIGHT, &LIGHT_CHOICES),
            options("Background motion", GROUP_MOTION, &MOTION_CHOICES),
            switch("Breathing clock", SWITCH_BREATHE),
        ],
        Tab::Glass => vec![
            options("Glass material", GROUP_GLASS, &GLASS_CHOICES),
            options("Corners", GROUP_CURVE, &CURVE_CHOICES),
        ],
        Tab::Widgets => vec![
            switch("Uptime", SWITCH_SYSTEM),
            switch("Network", SWITCH_NETWORK),
            switch("Battery", SWITCH_BATTERY),
        ],
    }
}

/// The sheet: a glass panel of tabs and preset cards.
#[allow(clippy::too_many_arguments)]
fn sheet(
    canvas: &mut Canvas,
    v: &View<'_>,
    tw: Tweaks,
    radius: f32,
    cache: &mut Cache,
    key: Key,
    hits: &mut Vec<(Rect, Hot)>,
) {
    let t = v.t;
    let tab = v.state.tab;
    let list = cards(tab);
    let cols = 3usize;
    let rows = list
        .len()
        .max(1)
        .saturating_add(cols.saturating_sub(1))
        .checked_div(cols)
        .unwrap_or(1);
    let card_h = t.px(96.0);
    let gap = t.px(12.0);
    let width = t.px(980.0).min(v.w - t.px(80.0));
    let header = t.px(70.0);
    let height = header + (card_h + gap) * rows as f32 + t.px(16.0);
    let r = Rect::new(
        (v.w - width) / 2.0,
        v.h - height - t.px(24.0),
        width,
        height,
    );
    canvas.soft_shadow(r, t.px(36.0), t.px(40.0), t.px(14.0), 0.5);
    liquid(
        canvas,
        t,
        r,
        t.px(36.0).min(radius + t.px(8.0)),
        tw,
        cache,
        key,
    );

    // Tabs, then Reset and Done.
    let mut x = r.x + t.px(22.0);
    for (i, item) in Tab::ALL.iter().enumerate() {
        let style = Style::new(t.px(14.0), t.text());
        let w = font::measure(item.label(), &style) + t.px(36.0);
        let tab_r = Rect::new(x, r.y + t.px(19.0), w, t.px(32.0));
        let selected = *item == tab;
        canvas.fill_round_rect(
            tab_r,
            t.px(16.0),
            with_alpha(rgb(0xFFFFFF), if selected { 0.20 } else { 0.06 }),
        );
        centred(
            canvas,
            item.label(),
            tab_r.center().0,
            tab_r.y + t.px(21.0),
            &style,
        );
        hits.push((tab_r, Hot::LockTab(u8::try_from(i).unwrap_or(0))));
        x += w + t.px(6.0);
    }
    let done = Rect::new(
        r.right() - t.px(22.0) - t.px(92.0),
        r.y + t.px(16.0),
        t.px(92.0),
        t.px(38.0),
    );
    ui::button(canvas, t, done, "Done", State::Normal, Tone::Accent);
    hits.push((done, Hot::LockDone));
    let reset = Rect::new(
        done.x - t.px(10.0) - t.px(84.0),
        done.y,
        t.px(84.0),
        t.px(38.0),
    );
    ui::button(canvas, t, reset, "Reset", State::Normal, Tone::Neutral);
    hits.push((reset, Hot::LockReset));

    // Cards.
    for (i, card) in list.iter().enumerate() {
        let col = i.checked_rem(cols).unwrap_or(0);
        let row = i.checked_div(cols).unwrap_or(0);
        let card_r = Rect::new(
            r.x + t.px(22.0) + (card_w(t, width) + gap) * col as f32,
            r.y + header + (card_h + gap) * row as f32,
            card_w(t, width),
            card_h,
        );
        canvas.fill_round_rect(card_r, t.px(20.0), with_alpha(rgb(0xFFFFFF), 0.07));
        canvas.stroke_round_rect(card_r, t.px(20.0), 1.0, with_alpha(rgb(0xFFFFFF), 0.10));
        let label = Style::new(t.px(14.0), with_alpha(t.text(), 0.78));
        font::draw(
            canvas,
            card.label,
            card_r.x + t.px(14.0),
            card_r.y + t.px(22.0),
            &label,
        );
        match &card.kind {
            CardKind::Options { group, items } => {
                chips(canvas, v, tw, card_r, *group, items, hits);
            }
            CardKind::Switch { index } => {
                let r_switch = Rect::new(
                    card_r.right() - t.px(14.0) - t.px(52.0),
                    card_r.y + t.px(13.0),
                    t.px(52.0),
                    t.px(30.0),
                );
                let on = switch_on(tw, *index);
                ui::toggle(canvas, t, r_switch, on, State::Normal);
                hits.push((r_switch.inflate(t.px(6.0)), Hot::LockSwitch(*index)));
            }
        }
    }
}

/// The width of one card in the sheet.
fn card_w(t: &Tokens, sheet_w: f32) -> f32 {
    let inner = sheet_w - t.px(44.0);
    (inner - t.px(12.0) * 2.0) / 3.0
}

/// The preset buttons inside a card, wrapping onto a second line if need be.
fn chips(
    canvas: &mut Canvas,
    v: &View<'_>,
    tw: Tweaks,
    card: Rect,
    group: u8,
    items: &'static [Choice],
    hits: &mut Vec<(Rect, Hot)>,
) {
    let t = v.t;
    let accent = tw.accent_color();
    let selected = tw.option(group);
    let mut x = card.x + t.px(14.0);
    let mut y = card.y + t.px(34.0);
    let style = Style::new(t.px(13.0), t.text());
    for (i, choice) in items.iter().enumerate() {
        let index = u8::try_from(i).unwrap_or(u8::MAX);
        let is_on = index == selected;
        let (w, h) = match choice.swatch {
            Some(_) => (t.px(34.0), t.px(34.0)),
            None => (font::measure(choice.label, &style) + t.px(24.0), t.px(34.0)),
        };
        if x + w > card.right() - t.px(12.0) {
            x = card.x + t.px(14.0);
            y += h + t.px(8.0);
        }
        let chip = Rect::new(x, y, w, h);
        // A swatch is a colour by definition; a word chip is a fill.
        canvas.fill_round_rect(
            chip,
            t.px(17.0),
            match choice.swatch {
                Some(c) => rgb(c & 0x00FF_FFFF),
                None if is_on => with_alpha(rgb(accent & 0x00FF_FFFF), 1.0),
                None => with_alpha(rgb(0xFFFFFF), 0.10),
            },
        );
        if is_on {
            // Selection is a ring, not only a fill: a chosen swatch must be
            // visible as chosen against the wallpaper behind it.
            canvas.stroke_round_rect(
                chip.inflate(t.px(3.0)),
                t.px(20.0),
                t.px(2.0),
                rgb(accent & 0x00FF_FFFF),
            );
        }
        if !choice.label.is_empty() {
            centred(
                canvas,
                choice.label,
                chip.center().0,
                chip.y + t.px(22.0),
                &Style::new(
                    t.px(13.0),
                    if is_on {
                        with_alpha(rgb(0x000000), 0.85)
                    } else {
                        t.text()
                    },
                ),
            );
        }
        hits.push((chip, Hot::LockOption(group, index)));
        x += w + t.px(8.0);
    }
}

/// Whether a switch is on.
fn switch_on(tw: Tweaks, index: u8) -> bool {
    if index == SWITCH_BREATHE {
        return tw.breathe;
    }
    tw.widgets
        .get(usize::from(index.saturating_sub(1)))
        .copied()
        .unwrap_or(false)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::panic)]
mod tests {
    use super::*;
    use litric_design::{Theme, ThemeKind};

    #[test]
    fn a_secret_never_prints_itself() {
        let s = Secret::new("hunter2");
        let shown = format!("{s:?}");
        assert!(!shown.contains("hunter2"), "{shown}");
        assert!(shown.contains("7 bytes"), "{shown}");
    }

    #[test]
    fn a_secret_is_hex_on_the_wire_and_lower_case() {
        assert_eq!(Secret::new("ab").hex(), "6162");
        assert_eq!(Secret::new("").hex(), "");
        // Non-ASCII goes as its UTF-8 bytes, which is what the daemon decodes.
        assert_eq!(Secret::new("\u{e9}").hex(), "c3a9");
    }

    #[test]
    fn taking_the_secret_empties_the_buffer() {
        let mut st = LockState::default();
        st.wake();
        st.push("abc");
        assert_eq!(st.typed(), 3);
        let taken = st.take_secret();
        assert_eq!(taken.len(), 3);
        assert_eq!(st.typed(), 0, "the clear buffer is the point of taking it");
    }

    #[test]
    fn nothing_is_typed_into_a_field_that_is_not_out() {
        let mut st = LockState::default();
        st.push("x");
        assert_eq!(st.typed(), 0);
        st.wake();
        // The tab and the newline are control characters: they are not text,
        // and a field that accepted them would be a field that could never
        // match a password.
        st.push("x\ty\n");
        assert_eq!(st.typed(), 2);
        st.backspace();
        assert_eq!(st.typed(), 1);
    }

    #[test]
    fn the_default_look_is_the_designed_one() {
        let tw = Tweaks::default();
        assert_eq!(tw.wall, Wall::Aurora);
        assert_eq!(tw.digits, Digits::Solid);
        assert_eq!(tw.glow(), 0.0, "crisp digits are the default");
        assert_eq!(tw.bloom(), 0.0);
        assert_eq!(tw.blur(), 8);
        assert_eq!(tw.radius(), 28.0);
        assert!(!tw.animating());
        assert!((tw.speed() - 1.0).abs() < f32::EPSILON);
        assert!(!tw.breathe && tw.widgets.iter().all(|w| *w));
    }

    #[test]
    fn still_lock_has_no_animation_frames_but_motion_is_bounded() {
        let mut state = LockState {
            locked: true,
            ..LockState::default()
        };
        assert!(!state.advance(10), "still mode does not animate");
        state.tweaks.motion = 2;
        assert!(!state.advance(20), "sub-frame motion waits");
        assert!(state.advance(50), "motion schedules a frame at 30 Hz");
    }

    #[test]
    fn every_preset_group_round_trips_through_choose() {
        let mut tw = Tweaks::default();
        for group in 0..=GROUP_CURVE {
            let count = match group {
                GROUP_WALL => Wall::ALL.len(),
                GROUP_DIGITS => Digits::ALL.len(),
                GROUP_ACCENT => ACCENTS.len(),
                GROUP_SIZE => SIZE.len(),
                GROUP_WEIGHT => WEIGHT_PRESETS.len(),
                GROUP_LIGHT => LIGHT.len(),
                GROUP_MOTION => MOTION.len(),
                GROUP_GLASS => GLASS.len(),
                _ => CURVE.len(),
            };
            for i in 0..count {
                let i = u8::try_from(i).unwrap_or(0);
                tw.choose(group, i);
                assert_eq!(tw.option(group), i, "group {group} option {i}");
            }
            // An index past the end is nothing, not a clamp: the look must not
            // change because a hotspot named a button that is not there.
            let before = tw;
            tw.choose(group, 200);
            assert_eq!(tw, before, "group {group} ignored an unknown option");
        }
    }

    #[test]
    fn the_widget_switches_are_two_way() {
        let mut tw = Tweaks::default();
        tw.toggle(SWITCH_BREATHE);
        assert!(tw.breathe);
        tw.toggle(SWITCH_BATTERY);
        assert!(!tw.widgets[2], "the battery widget is the third slot");
        tw.toggle(99);
        assert!(!tw.widgets[2], "an unknown switch index changes nothing");
    }

    #[test]
    fn the_drift_stops_when_motion_is_still() {
        let mut st = LockState {
            locked: true,
            ..LockState::default()
        };
        st.tweaks.motion = 2;
        st.advance(1_000);
        st.advance(1_016);
        assert!(st.flow_ms > 0, "the fields drift while motion is on");
        let frozen = st.flow_ms;
        st.tweaks.motion = 0; // Still
        st.advance(1_032);
        assert_eq!(st.flow_ms, frozen, "Still holds the fields where they are");
    }

    #[test]
    fn the_drift_does_not_replay_a_pause_in_one_frame() {
        let mut st = LockState {
            locked: true,
            ..LockState::default()
        };
        st.advance(1_000);
        st.advance(1_016);
        let before = st.flow_ms;
        st.advance(1_016 + 60_000);
        assert_eq!(st.flow_ms, before, "a long gap is not an hour of keyframes");
    }

    #[test]
    fn the_keyframes_start_and_end_at_rest() {
        let (dx, dy, scale) = sample(&FLOW_A, 0.0);
        assert!(dx.abs() < f32::EPSILON && dy.abs() < f32::EPSILON);
        assert!((scale - 1.0).abs() < f32::EPSILON);
        let (_, _, mid) = sample(&FLOW_A, 0.33);
        assert!(
            (mid - 1.12).abs() < 0.001,
            "the 33% keyframe is a scale of 1.12"
        );
    }

    // -----------------------------------------------------------------------
    // The frame cache (BUG-0018)
    // -----------------------------------------------------------------------

    /// A view of a locked surface at 1280x800 with the shipped look.
    fn frame_view<'a>(t: &'a Tokens, state: &'a LockState) -> View<'a> {
        View {
            t,
            state,
            clock: "10:24".to_owned(),
            date: "Mon 15 October".to_owned(),
            w: 1280.0,
            h: 800.0,
            battery: Some(72),
            network: Some("wifi".to_owned()),
            uptime: Some(4_000.0),
            verifier: Some("unix-chkpwd".to_owned()),
        }
    }

    #[test]
    fn a_frame_from_a_warm_cache_is_the_frame_from_a_cold_one() {
        let t = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
        for stage in [Stage::Idle, Stage::Entry] {
            let state = LockState {
                locked: true,
                stage,
                ..LockState::default()
            };
            let v = frame_view(&t, &state);
            let mut cache = Cache::default();
            let mut hits = Vec::new();

            let mut cold = Canvas::new(1280, 800, 0);
            draw(&mut cold, &v, &mut cache, &mut hits);
            let derived = cache.covered.len();
            assert!(derived > 0, "the first frame derived a backdrop to reuse");

            let mut warm = Canvas::new(1280, 800, 0);
            draw(&mut warm, &v, &mut cache, &mut hits);
            assert_eq!(
                cache.covered.len(),
                derived,
                "a frame that changed nothing derived the backdrop again"
            );
            let mut differing = 0_usize;
            let mut first = usize::MAX;
            for (i, (a, b)) in cold.pixels().iter().zip(warm.pixels()).enumerate() {
                if a != b {
                    if differing < 6 {
                        println!(
                            "DIFF x={} y={} cold={:#010x} warm={:#010x} d={}",
                            i % 1280,
                            i.div_euclid(1280),
                            a,
                            b,
                            (*a as i64 - *b as i64).abs()
                        );
                    }
                    differing = differing.saturating_add(1);
                    if first == usize::MAX {
                        first = i;
                    }
                }
            }
            assert_eq!(
                differing,
                0,
                "the cached frame is the frame the surface draws ({stage:?}); \
                 {differing} pixel(s) differ, first at index {first} \
                 (x={}, y={})",
                first % 1280,
                first.div_euclid(1280)
            );
        }
    }

    #[test]
    fn a_panel_is_the_same_whether_its_blur_is_reused_or_redone() {
        // The cache's own bookkeeping is the only difference between the two
        // frames: the pixels must not know which of the two drew them. This is
        // what catches a region blurred on top of an already-blurred one.
        let t = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
        let state = LockState {
            locked: true,
            edit: true,
            ..LockState::default()
        };
        let v = frame_view(&t, &state);
        let mut cache = Cache::default();
        let mut hits = Vec::new();

        let mut reused = Canvas::new(1280, 800, 0);
        draw(&mut reused, &v, &mut cache, &mut hits);

        let mut redone = Canvas::new(1280, 800, 0);
        cache.covered.clear();
        draw(&mut redone, &v, &mut cache, &mut hits);
        assert_eq!(
            reused.pixels(),
            redone.pixels(),
            "a re-derived blur changed the panel"
        );
    }

    #[test]
    fn a_drift_under_half_a_step_keeps_the_frame_it_had() {
        // The cache is only worth having if a frame usually has the one before
        // it: the fields move about 18 px a second, and the key must not turn
        // over on every one of them.
        let a = steady((100.0, 50.0, 1.04), 1.0);
        let b = steady((101.9, 51.9, 1.041), 1.0);
        let c = steady((102.5, 50.0, 1.04), 1.0);
        assert_eq!(a, b, "under half a step is still the same frame");
        assert_ne!(a, c, "past half a step is a new one");
    }

    #[test]
    fn the_drift_reaches_the_field_in_whole_pixels() {
        // The key keeps the offsets in steps; the field must be drawn at the
        // pixel those steps mean, or the frame is not the frame the key names.
        let key = Key {
            flow: [steady((140.0, -90.0, 1.12), 1.0); 3],
            palette: (0, 0, 0, 0),
            scale: 4096,
            size: (1280, 800),
            clock: 3_520,
            light: 2,
        };
        let flow = key.flow();
        assert!(!flow.is_empty(), "a key always has its three fields");
        let (dx, dy, scale) = match flow.first().copied() {
            Some(field) => field,
            // Unreachable given the assertion above, which is where an empty
            // key is reported.
            None => return,
        };
        assert!((dx - 140.0).abs() < f32::EPSILON, "x landed on {dx}");
        // -90 is exactly on a half step of four, and halves round away from
        // zero: the field lands on -92, which is a step, not on -90.
        assert!((dy + 92.0).abs() < f32::EPSILON, "y landed on {dy}");
        // The scale is kept in steps of 1/128, which is a quarter of a pixel on
        // the edge of a field that size: the value comes back on the step, not
        // on the keyframe's own number.
        assert!(
            (scale - 1.12).abs() <= 1.0 / SCALE_STEP,
            "scale landed on {scale}"
        );
    }
}
