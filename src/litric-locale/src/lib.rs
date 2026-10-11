//! LitricOS localisation.
//!
//! # Layer position
//!
//! **Layer 3 (System API).** No rendering, no devices, no clock of its own: it
//! is handed a UTC instant and answers how this machine's user would write it.
//!
//! # Three things that must not be confused
//!
//! ```text
//!   UTC        what the kernel counts. No zone, no summer time.
//!   time zone  which offset applies at which instant. From /usr/share/zoneinfo.
//!   locale     how it is written. 24-hour or 12-hour, day before month.
//! ```
//!
//! They are independent. Somebody in Bangkok may prefer `en-GB` formatting, and
//! a machine set to `th-TH` may be in a rack in Frankfurt. Every desktop that
//! shows the wrong time twice a year has folded two of these into one.
//!
//! # Nothing is faked
//!
//! If the zone cannot be determined, the answer is UTC **and the source says
//! so**. If the locale cannot be determined, the answer is the `C` locale —
//! not American defaults — **and the source says so**. A confident wrong
//! answer is worse than an honest fallback, and the status file carries the
//! difference so a diagnostic can show it.

#![forbid(unsafe_code)]

pub mod civil;
pub mod locale;
pub mod tz;

pub use civil::{days_in_month, from_unix, is_leap, to_unix, DateTime, Weekday, MONTH_SHORT};
pub use locale::{DateOrder, HourCycle, Locale, LocaleSource, Measurement};
pub use tz::{detect, load_named, Offset, TimeZone, ZoneSource};

use std::path::Path;

/// Everything the desktop needs to show a clock: where the machine is, and how
/// its user writes things.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regional {
    /// Where the machine is.
    pub zone: TimeZone,
    /// How its user writes things.
    pub locale: Locale,
}

impl Regional {
    /// Detect both from the system, reading nothing outside `root`.
    #[must_use]
    pub fn detect<F: Fn(&str) -> Option<String>>(root: &Path, env: F) -> Self {
        let tz_env = env("TZ");
        let config = std::fs::read_to_string(root.join("etc/locale.conf")).ok();
        Self {
            zone: tz::detect(root, tz_env.as_deref()),
            locale: Locale::detect(&env, config.as_deref()),
        }
    }

    /// The C locale in UTC, for a machine that has told us nothing.
    #[must_use]
    pub fn fallback(reason: &str) -> Self {
        Self {
            zone: TimeZone::utc(reason),
            locale: Locale::default(),
        }
    }

    /// The local civil time for a UTC instant.
    #[must_use]
    pub fn local(&self, utc: i64) -> DateTime {
        self.zone.local(utc)
    }

    /// The clock as it should appear in the top bar.
    ///
    /// No zone suffix when the zone is known: a local clock does not need to
    /// name its own zone, and naming it is how "01:26 UTC" ends up on a machine
    /// in Bangkok. When the zone is *not* known, the suffix is the honest
    /// signal that this is UTC rather than local time.
    #[must_use]
    pub fn clock(&self, utc: i64) -> String {
        let text = self.locale.format_time(self.local(utc));
        if self.zone.source().is_known() {
            text
        } else {
            format!("{text} UTC")
        }
    }

    /// The date as it should appear beside the clock.
    #[must_use]
    pub fn date(&self, utc: i64) -> String {
        self.locale.format_date_medium(self.local(utc))
    }

    /// Seconds until the displayed clock would change.
    ///
    /// The compositor uses this to sleep until the minute turns instead of
    /// waking every second to find out nothing changed.
    #[must_use]
    pub fn seconds_until_clock_changes(&self, utc: i64) -> i64 {
        let d = self.local(utc);
        let into_minute = i64::from(d.second);
        let to_minute = 60i64.saturating_sub(into_minute).max(1);
        match self.zone.next_transition(utc) {
            Some(t) => to_minute.min(t.saturating_sub(utc).max(1)),
            None => to_minute,
        }
    }

    /// Lines for the status file and the diagnostic overlay.
    #[must_use]
    pub fn lines(&self, utc: i64) -> Vec<String> {
        let mut out = vec![
            format!("tz.name={}", self.zone.name()),
            format!("tz.source={}", self.zone.source().token()),
            format!("tz.offset={}", self.zone.offset_at(utc).iso()),
            format!("tz.abbrev={}", self.zone.offset_at(utc).abbrev),
            format!("tz.dst={}", self.zone.offset_at(utc).is_dst),
            format!("tz.transitions={}", self.zone.transition_count()),
            format!("clock.local={}", self.clock(utc)),
            format!(
                "clock.utc={}",
                Locale::default().format_time(from_unix(utc))
            ),
        ];
        out.extend(self.locale.lines());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_undetectable_machine_says_utc_on_its_clock() {
        // The signal that matters: a clock with no zone must not look local.
        let r = Regional::fallback("nothing configured");
        assert!(r.clock(1_787_966_760).ends_with(" UTC"));
        assert!(!r.zone.source().is_known());
    }

    #[test]
    fn a_known_zone_shows_a_local_clock_with_no_suffix() {
        let mut r = Regional::fallback("x");
        let data = {
            // Asia/Bangkok: +07, no transitions.
            let mut out = Vec::new();
            out.extend_from_slice(b"TZif");
            out.push(0);
            out.extend_from_slice(&[0u8; 15]);
            for v in [0u32, 0, 0, 0, 1, 4] {
                out.extend_from_slice(&v.to_be_bytes());
            }
            out.extend_from_slice(&25_200i32.to_be_bytes());
            out.push(0);
            out.push(0);
            out.extend_from_slice(b"+07\0");
            out
        };
        r.zone = TimeZone::parse("Asia/Bangkok", &data, ZoneSource::LocaltimeLink).expect("parses");
        r.locale = Locale::from_tag("th-TH", LocaleSource::UserChoice);
        assert_eq!(r.clock(1_787_966_760), "08:26");
        assert!(!r.clock(1_787_966_760).contains("UTC"));
    }

    #[test]
    fn the_clock_sleeps_until_the_minute_turns() {
        let r = Regional::fallback("x");
        // 20 seconds into a minute leaves 40 until it changes.
        assert_eq!(r.seconds_until_clock_changes(20), 40);
        assert_eq!(r.seconds_until_clock_changes(59), 1);
        assert!(r.seconds_until_clock_changes(0) > 0, "never sleep forever");
    }

    #[test]
    fn detection_of_an_empty_root_is_honest_about_both_halves() {
        let r = Regional::detect(std::path::Path::new("no-such-root-2c8a"), |_| None);
        assert!(!r.zone.source().is_known());
        assert!(!r.locale.source.is_known());
        let lines = r.lines(0);
        assert!(lines
            .iter()
            .any(|l| l.starts_with("tz.source=fallback-utc")));
        assert!(lines
            .iter()
            .any(|l| l.starts_with("locale.source=fallback-c")));
    }
}
