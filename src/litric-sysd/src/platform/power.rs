//! Power facts: supplies, thermal zones, backlights.
//!
//! The shell already drew a battery indicator before this module existed, from
//! a reader inside the shell itself. That reader is folded in here so there is
//! one answer to "what is the battery doing" rather than two that can disagree,
//! and so the parsers are exercised by tests instead of only on a laptop.
//!
//! Two rules are lifted from that reader and kept, because both were learned
//! from a real misreport:
//!
//! * **A percentage without a status is not a percentage.** A battery at 62%
//!   that is discharging and one at 62% that is charging mean opposite things,
//!   and the number alone cannot tell them apart.
//! * **A machine without a battery produces no battery line at all**, never a
//!   zero. A desktop reporting `capacity=0` reads as a flat battery, which is
//!   the loudest possible false alarm.
//!
//! Nothing here is privileged and nothing here writes: `systemctl suspend` and
//! friends are the shell's business and are recorded as an action, not a fact
//! (`docs/design/wave1/03-power.md`).

use std::path::Path;

use super::{read_attr, read_dir};

/// What a power supply is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupplyKind {
    /// A battery.
    Battery,
    /// Mains/AC.
    Mains,
    /// An uninterruptible power supply.
    Ups,
    /// Something this module does not know, reported as `unknown` rather than
    /// guessed at from the name.
    Unknown,
}

impl SupplyKind {
    /// The stable word used in the reply, and the parse of the kernel's own.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        match text.trim() {
            "Battery" => Self::Battery,
            "Mains" => Self::Mains,
            "UPS" => Self::Ups,
            _ => Self::Unknown,
        }
    }

    /// The word written into the facts line.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Battery => "battery",
            Self::Mains => "mains",
            Self::Ups => "ups",
            Self::Unknown => "unknown",
        }
    }
}

/// One power supply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Supply {
    /// Directory name, e.g. `BAT0`.
    pub name: String,
    /// Kind.
    pub kind: SupplyKind,
    /// `charging`, `discharging`, `full`, `not-charging`, or `unknown`.
    ///
    /// The kernel's word, normalised, not translated: `Not charging` and
    /// `Discharging` are different states and a machine plugged in with a
    /// charge limit set reports the first one forever.
    pub status: Option<String>,
    /// Percentage, 0-100. `None` when absent or outside that range.
    pub capacity: Option<u8>,
    /// Charge now, in microwatt-hours, when the supply reports energy.
    pub energy_now_uwh: Option<u64>,
    /// Charge when full, in microwatt-hours.
    pub energy_full_uwh: Option<u64>,
    /// Instantaneous draw, in microwatts. Many machines do not publish this,
    /// and the honest reply to "how many watts" is then "the kernel does not
    /// say" rather than a zero.
    pub power_now_uw: Option<u64>,
    /// Charge cycles, when the battery counts them.
    pub cycle_count: Option<u32>,
}

impl Supply {
    /// Whether this supply can hold charge, i.e. whether a percentage is
    /// meaningful for it.
    #[must_use]
    pub const fn is_battery(&self) -> bool {
        matches!(self.kind, SupplyKind::Battery)
    }

    /// Percentage remaining, computed from energy when the kernel publishes
    /// energy rather than a capacity.
    ///
    /// A computed percentage is only produced when the full figure is nonzero;
    /// dividing by a zero `energy_full` is how a battery would read 0% at 40%.
    #[must_use]
    pub fn percent(&self) -> Option<u8> {
        if let Some(c) = self.capacity {
            return Some(c);
        }
        match (self.energy_now_uwh, self.energy_full_uwh) {
            (Some(now), Some(full)) if full > 0 => {
                // saturating_mul and checked_div rather than `* 100 / full`: the
                // workspace denies bare arithmetic in system code, and a
                // multiplication that wrapped would report an empty battery on
                // a full one. The divisor is `full`, which the guard above has
                // already shown to be nonzero.
                let scaled = now.saturating_mul(100);
                let pct = scaled.checked_div(full).unwrap_or(0).min(100);
                u8::try_from(pct).ok()
            }
            _ => None,
        }
    }
}

/// One thermal zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    /// Directory name, e.g. `thermal_zone0`.
    pub name: String,
    /// `x86_pkg_temp`, `acpitz`, … as the kernel writes it.
    pub kind: Option<String>,
    /// Temperature in **millidegrees**, raw. Rounded into degrees at the last
    /// moment, if at all: the rounding is a display decision, not a fact.
    pub temp_mc: Option<i64>,
}

/// One cooling device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cooling {
    /// Directory name.
    pub name: String,
    /// `Processor`, `intel_powerclamp`, `Fan`, …
    pub kind: Option<String>,
    /// Current state, 0-31 as the kernel numbers them.
    pub state: Option<u32>,
}

/// One backlight device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backlight {
    /// Directory name, e.g. `intel_backlight`.
    pub name: String,
    /// Current brightness in the device's own units.
    pub brightness: Option<u32>,
    /// Maximum, which is the only way a brightness number means anything.
    pub max_brightness: Option<u32>,
}

/// Everything this module reports about power.
#[derive(Debug, Clone, Default)]
pub struct PowerFacts {
    /// Supplies, sorted by name.
    pub supplies: Vec<Supply>,
    /// Thermal zones.
    pub zones: Vec<Zone>,
    /// Cooling devices.
    pub cooling: Vec<Cooling>,
    /// Backlights.
    pub backlights: Vec<Backlight>,
}

impl PowerFacts {
    /// The facts as `key=value` lines.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        let batteries: Vec<&Supply> = self.supplies.iter().filter(|s| s.is_battery()).collect();
        // A machine with no battery says so once, and then says nothing about
        // batteries. The status area keys off the count.
        out.push(format!("power.battery.count={}", batteries.len()));
        for b in &batteries {
            out.push(format!(
                "power.supply name={} type={} status={} capacity={} energy_now_uwh={} energy_full_uwh={} power_now_uw={} cycles={}",
                b.name,
                b.kind.token(),
                b.status.as_deref().unwrap_or("unknown"),
                b.percent()
                    .map_or_else(|| "-".to_owned(), |c| c.to_string()),
                opt(b.energy_now_uwh),
                opt(b.energy_full_uwh),
                opt(b.power_now_uw),
                opt(b.cycle_count)
            ));
        }
        for s in self
            .supplies
            .iter()
            .filter(|s| matches!(s.kind, SupplyKind::Mains))
        {
            out.push(format!(
                "power.ac name={} online={}",
                s.name,
                s.status
                    .as_deref()
                    .map_or_else(|| "unknown".to_owned(), |st| st.to_owned())
            ));
        }
        for z in &self.zones {
            out.push(format!(
                "thermal.zone name={} type={} temp_mc={}",
                z.name,
                z.kind.as_deref().unwrap_or("-"),
                opt(z.temp_mc)
            ));
        }
        for c in &self.cooling {
            out.push(format!(
                "thermal.cooling name={} type={} state={}",
                c.name,
                c.kind.as_deref().unwrap_or("-"),
                opt(c.state)
            ));
        }
        for b in &self.backlights {
            out.push(format!(
                "power.backlight name={} brightness={} max_brightness={}",
                b.name,
                opt(b.brightness),
                opt(b.max_brightness)
            ));
        }
        out
    }
}

/// `Some(v)` as its digits, `None` as `-`.
///
/// A hyphen and not a zero: a fact the machine did not publish must not be
/// indistinguishable from a fact that is genuine zero. This is the single most
/// repeated rule in this module.
fn opt<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map_or_else(|| "-".to_owned(), |v| v.to_string())
}

/// Read everything from a filesystem root.
#[must_use]
pub fn read(root: &Path) -> PowerFacts {
    let mut supplies = Vec::new();
    for name in read_dir(root, "sys/class/power_supply") {
        let base = format!("sys/class/power_supply/{name}");
        let kind = read_attr(root, &format!("{base}/type"))
            .as_deref()
            .map_or(SupplyKind::Unknown, SupplyKind::parse);
        let status = read_attr(root, &format!("{base}/status"))
            .as_deref()
            .map(parse_status)
            .map(str::to_owned);
        supplies.push(Supply {
            capacity: read_attr(root, &format!("{base}/capacity")).and_then(|s| parse_capacity(&s)),
            energy_now_uwh: read_attr(root, &format!("{base}/energy_now"))
                .and_then(|s| s.parse().ok()),
            energy_full_uwh: read_attr(root, &format!("{base}/energy_full"))
                .and_then(|s| s.parse().ok()),
            power_now_uw: read_attr(root, &format!("{base}/power_now"))
                .and_then(|s| s.parse().ok()),
            cycle_count: read_attr(root, &format!("{base}/cycle_count"))
                .and_then(|s| s.parse().ok()),
            kind,
            status,
            name,
        });
    }

    let mut zones = Vec::new();
    for name in read_dir(root, "sys/class/thermal") {
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let base = format!("sys/class/thermal/{name}");
        zones.push(Zone {
            kind: read_attr(root, &format!("{base}/type")),
            temp_mc: read_attr(root, &format!("{base}/temp")).and_then(|s| s.parse().ok()),
            name,
        });
    }

    let mut cooling = Vec::new();
    for name in read_dir(root, "sys/class/thermal") {
        if !name.starts_with("cooling_device") {
            continue;
        }
        let base = format!("sys/class/thermal/{name}");
        cooling.push(Cooling {
            kind: read_attr(root, &format!("{base}/type")),
            state: read_attr(root, &format!("{base}/cur_state")).and_then(|s| s.parse().ok()),
            name,
        });
    }

    let mut backlights = Vec::new();
    for name in read_dir(root, "sys/class/backlight") {
        let base = format!("sys/class/backlight/{name}");
        backlights.push(Backlight {
            brightness: read_attr(root, &format!("{base}/brightness")).and_then(|s| s.parse().ok()),
            max_brightness: read_attr(root, &format!("{base}/max_brightness"))
                .and_then(|s| s.parse().ok()),
            name,
        });
    }

    PowerFacts {
        supplies,
        zones,
        cooling,
        backlights,
    }
}

/// Normalise the kernel's `status` words.
///
/// `Not charging` is kept as its own word rather than folded into `full` or
/// `discharging`: it is what a laptop with a charge threshold set reports while
/// plugged in, and calling that "full" is a claim the machine did not make.
#[must_use]
pub fn parse_status(text: &str) -> &'static str {
    match text.trim() {
        "Charging" => "charging",
        "Discharging" => "discharging",
        "Full" => "full",
        "Not charging" => "not-charging",
        "Unknown" => "unknown",
        _ => "unknown",
    }
}

/// Parse a `capacity` attribute, refusing anything outside 0-100.
///
/// A capacity of 255 is what some drivers report for "no idea"; passing it
/// through would draw a battery at 255%, and clamping it to 100 would draw a
/// full battery. Neither is what the machine said, so neither is reported.
#[must_use]
pub fn parse_capacity(text: &str) -> Option<u8> {
    let v: u32 = text.trim().parse().ok()?;
    u8::try_from(v).ok().filter(|v| *v <= 100)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn fixture(root: &Path, files: &[(&str, &str)]) {
        for (rel, content) in files {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
            std::fs::write(p, content).expect("write");
        }
    }

    #[test]
    fn a_desktop_reports_no_battery_rather_than_a_flat_one() {
        let facts = PowerFacts {
            supplies: vec![Supply {
                name: "AC".to_owned(),
                kind: SupplyKind::Mains,
                status: Some("1".to_owned()),
                capacity: None,
                energy_now_uwh: None,
                energy_full_uwh: None,
                power_now_uw: None,
                cycle_count: None,
            }],
            ..PowerFacts::default()
        };
        let lines = facts.to_lines();
        assert!(lines.iter().any(|l| l == "power.battery.count=0"));
        assert!(
            !lines.iter().any(|l| l.starts_with("power.supply ")),
            "a machine with no battery must produce no battery line: {lines:?}"
        );
        assert!(lines.iter().any(|l| l == "power.ac name=AC online=1"));
    }

    #[test]
    fn percentage_can_come_from_energy_when_there_is_no_capacity_file() {
        let s = Supply {
            name: "BAT0".to_owned(),
            kind: SupplyKind::Battery,
            status: Some("discharging".to_owned()),
            capacity: None,
            energy_now_uwh: Some(25_000_000),
            energy_full_uwh: Some(50_000_000),
            power_now_uw: Some(8_000_000),
            cycle_count: Some(210),
        };
        assert_eq!(s.percent(), Some(50));

        // A zero full figure is not a percentage of anything: dividing by it
        // would report 0% at half charge.
        let broken = Supply {
            energy_full_uwh: Some(0),
            ..s.clone()
        };
        assert_eq!(broken.percent(), None);
    }

    #[test]
    fn a_capacity_the_kernel_does_not_know_is_not_a_number() {
        assert_eq!(parse_capacity("62\n"), Some(62));
        assert_eq!(parse_capacity("0\n"), Some(0));
        assert_eq!(parse_capacity("100\n"), Some(100));
        // What several drivers write for "no idea".
        assert_eq!(parse_capacity("255\n"), None);
        assert_eq!(parse_capacity("-1\n"), None);
        assert_eq!(parse_capacity("bat\n"), None);
    }

    #[test]
    fn not_charging_is_its_own_state() {
        assert_eq!(parse_status("Charging"), "charging");
        assert_eq!(parse_status("Discharging"), "discharging");
        assert_eq!(parse_status("Full"), "full");
        assert_eq!(parse_status("Not charging"), "not-charging");
        assert_eq!(parse_status("nonsense"), "unknown");
    }

    #[test]
    fn missing_attributes_are_hyphens_not_zeros() {
        let s = Supply {
            name: "BAT0".to_owned(),
            kind: SupplyKind::Battery,
            status: None,
            capacity: Some(31),
            energy_now_uwh: None,
            energy_full_uwh: None,
            power_now_uw: None,
            cycle_count: None,
        };
        let line = PowerFacts {
            supplies: vec![s],
            ..PowerFacts::default()
        }
        .to_lines()
        .into_iter()
        .find(|l| l.starts_with("power.supply "))
        .expect("a battery line");
        assert!(line.contains("power_now_uw=-"), "{line}");
        assert!(line.contains("cycles=-"), "{line}");
        assert!(line.contains("capacity=31"), "{line}");
    }

    #[test]
    fn a_laptop_fixture_is_read_end_to_end() {
        let root = std::env::temp_dir().join("litric-power-fixture");
        let _ = std::fs::remove_dir_all(&root);
        fixture(
            &root,
            &[
                ("sys/class/power_supply/AC/type", "Mains\n"),
                ("sys/class/power_supply/AC/online", "1\n"),
                ("sys/class/power_supply/BAT0/type", "Battery\n"),
                ("sys/class/power_supply/BAT0/status", "Discharging\n"),
                ("sys/class/power_supply/BAT0/capacity", "62\n"),
                ("sys/class/power_supply/BAT0/power_now", "9500000\n"),
                ("sys/class/power_supply/BAT0/cycle_count", "143\n"),
                ("sys/class/thermal/thermal_zone0/type", "x86_pkg_temp\n"),
                ("sys/class/thermal/thermal_zone0/temp", "47000\n"),
                ("sys/class/thermal/cooling_device0/type", "Processor\n"),
                ("sys/class/thermal/cooling_device0/cur_state", "0\n"),
                ("sys/class/backlight/intel_backlight/brightness", "480\n"),
                (
                    "sys/class/backlight/intel_backlight/max_brightness",
                    "1200\n",
                ),
            ],
        );
        let facts = read(&root);
        assert_eq!(facts.supplies.len(), 2);
        let bat = facts
            .supplies
            .iter()
            .find(|s| s.name == "BAT0")
            .expect("BAT0");
        assert_eq!(bat.percent(), Some(62));
        assert_eq!(bat.power_now_uw, Some(9_500_000));
        assert_eq!(bat.cycle_count, Some(143));
        assert_eq!(facts.zones.len(), 1);
        assert_eq!(facts.zones[0].temp_mc, Some(47_000));
        assert_eq!(facts.cooling.len(), 1);
        assert_eq!(facts.cooling[0].kind.as_deref(), Some("Processor"));
        assert_eq!(facts.backlights.len(), 1);
        assert_eq!(facts.backlights[0].max_brightness, Some(1200));

        let text = facts.to_lines().join("\n");
        assert!(text.contains("power.battery.count=1"), "{text}");
        assert!(text.contains("thermal.zone name=thermal_zone0"), "{text}");
        assert!(
            text.contains("power.backlight name=intel_backlight"),
            "{text}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
