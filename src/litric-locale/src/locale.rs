//! Locale: what the user's region expects, separate from where the machine is.
//!
//! A locale decides how things are *written*. It does not decide what time it
//! is — that is the time zone — and the two are genuinely independent: somebody
//! in Bangkok may well prefer `en-GB` formatting, and a machine set to `th-TH`
//! may be sitting in a rack in Frankfurt. Anything that reads one and assumes
//! the other is wrong for real people.
//!
//! # What is here and what is not
//!
//! The region table below is a **heuristic**, and is documented as one. Real
//! regional data comes from CLDR, which is tens of megabytes and is not
//! something this crate can carry. What it does instead is cover the axes that
//! visibly differ — hour cycle, date order, first day of week, measurement
//! system, decimal separator — with a table small enough to read, and make
//! every one of them overridable by the user. A wrong default the user can
//! change in Settings is a preference; a hard-coded format is a bug.
//!
//! **No country is special-cased.** Thailand appears in the tables exactly
//! where Sweden and Peru do.

use crate::civil::{DateTime, Weekday, MONTH_SHORT};

/// Whether the clock runs to 12 or 24.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HourCycle {
    /// 00:00 to 23:59.
    H24,
    /// 12:00 AM to 11:59 PM.
    H12,
}

/// The order the parts of a date are written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateOrder {
    /// 2026-08-29.
    Ymd,
    /// 29/08/2026.
    Dmy,
    /// 08/29/2026.
    Mdy,
}

/// Which units are used for distance, mass and temperature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measurement {
    /// Metres, kilograms, Celsius.
    Metric,
    /// Feet, pounds, Fahrenheit.
    Imperial,
}

/// Where a locale came from.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocaleSource {
    /// An environment variable named it.
    Environment {
        /// Which one: `LC_ALL`, `LC_TIME` or `LANG`.
        variable: String,
    },
    /// `/etc/locale.conf` named it.
    SystemConfig,
    /// The user chose it in Settings.
    UserChoice,
    /// Nothing said. The C locale, labelled as a fallback.
    Fallback {
        /// Why.
        reason: String,
    },
}

impl LocaleSource {
    /// Whether the locale is known rather than assumed.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        !matches!(self, Self::Fallback { .. })
    }

    /// A short token for the status file.
    #[must_use]
    pub fn token(&self) -> String {
        match self {
            Self::Environment { variable } => variable.clone(),
            Self::SystemConfig => "/etc/locale.conf".to_owned(),
            Self::UserChoice => "settings".to_owned(),
            Self::Fallback { reason } => format!("fallback-c({reason})"),
        }
    }
}

/// Regions whose default clock is 12-hour.
///
/// A short list because the 12-hour clock is the minority worldwide. Everything
/// not listed defaults to 24-hour, which is also what the `C` locale does.
const H12_REGIONS: &[&str] = &[
    "US", "PH", "CA", "AU", "NZ", "IN", "PK", "BD", "EG", "SA", "JO", "MX", "CO", "PE", "CL", "MY",
    "GB", "IE", "ZA", "NG", "KE",
];

/// Regions that write the year first.
const YMD_REGIONS: &[&str] = &["CN", "JP", "KR", "TW", "HU", "LT", "MN", "BT"];

/// Regions that write the month first.
const MDY_REGIONS: &[&str] = &["US", "PH", "FM", "PW", "MH"];

/// Regions whose week starts on Sunday.
const SUNDAY_REGIONS: &[&str] = &[
    "US", "CA", "JP", "TW", "KR", "IL", "BR", "MX", "PH", "CO", "PE", "ZA", "TH", "HK", "ID",
];

/// Regions whose week starts on Saturday.
const SATURDAY_REGIONS: &[&str] = &["EG", "SA", "AE", "JO", "KW", "QA", "BH", "OM", "IQ", "LY"];

/// Regions that are not metric.
const IMPERIAL_REGIONS: &[&str] = &["US", "LR", "MM"];

/// Regions that use a comma as the decimal separator.
const COMMA_DECIMAL_REGIONS: &[&str] = &[
    "DE", "FR", "ES", "IT", "NL", "BE", "PT", "AT", "CH", "SE", "NO", "DK", "FI", "PL", "CZ", "SK",
    "HU", "RO", "BG", "GR", "TR", "RU", "UA", "BR", "AR", "CL", "CO", "PE", "VN", "ID",
];

/// Keyboard layouts by language, for the ones this build ships.
const LAYOUT_BY_LANGUAGE: &[(&str, &str)] = &[("en", "us")];

/// Keyboard layouts by region, which wins over language where both apply.
const LAYOUT_BY_REGION: &[(&str, &str)] = &[("GB", "gb"), ("IE", "gb"), ("US", "us")];

/// A user's regional preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    /// The BCP 47 tag, e.g. `en-US`, `th-TH`.
    pub tag: String,
    /// Language subtag, lowercase.
    pub language: String,
    /// Region subtag, uppercase, when there is one.
    pub region: Option<String>,
    /// 12- or 24-hour clock.
    pub hour_cycle: HourCycle,
    /// Date part order.
    pub date_order: DateOrder,
    /// Which day a week starts on.
    pub first_day: Weekday,
    /// Decimal separator.
    pub decimal: char,
    /// Thousands separator.
    pub group: char,
    /// Unit system.
    pub measurement: Measurement,
    /// Keyboard layout name, matching `litric-input`'s layout table.
    pub keyboard_layout: String,
    /// Where it came from.
    pub source: LocaleSource,
}

impl Default for Locale {
    /// The `C` locale: 24-hour, ISO dates, metric, Monday first.
    ///
    /// Not "American defaults": the C locale is the one thing every Unix agrees
    /// on, and choosing any country's conventions as the fallback would be
    /// exactly the hard-coding this module exists to avoid.
    fn default() -> Self {
        Self {
            tag: "C".to_owned(),
            language: "c".to_owned(),
            region: None,
            hour_cycle: HourCycle::H24,
            date_order: DateOrder::Ymd,
            first_day: Weekday::Monday,
            decimal: '.',
            group: ',',
            measurement: Measurement::Metric,
            keyboard_layout: "us".to_owned(),
            source: LocaleSource::Fallback {
                reason: "no locale configured".to_owned(),
            },
        }
    }
}

impl Locale {
    /// Build a locale from a BCP 47 or POSIX tag.
    ///
    /// Accepts `en_US.UTF-8`, `en-US`, `th_TH`, `pt`. The encoding and any
    /// `@modifier` are ignored: they say how the text is stored, not how it is
    /// formatted.
    #[must_use]
    pub fn from_tag(raw: &str, source: LocaleSource) -> Self {
        let cleaned = raw
            .split('.')
            .next()
            .unwrap_or(raw)
            .split('@')
            .next()
            .unwrap_or(raw)
            .replace('_', "-");
        let mut parts = cleaned.split('-');
        let language = parts.next().unwrap_or("").to_ascii_lowercase();
        let region = parts
            .next()
            .map(str::to_ascii_uppercase)
            .filter(|r| r.len() == 2 && r.chars().all(|c| c.is_ascii_alphabetic()));

        if language.is_empty() || language == "c" || language == "posix" {
            return Self {
                source,
                ..Self::default()
            };
        }

        let r = region.as_deref().unwrap_or("");
        let tag = region
            .as_ref()
            .map_or_else(|| language.clone(), |reg| format!("{language}-{reg}"));

        Self {
            hour_cycle: if H12_REGIONS.contains(&r) {
                HourCycle::H12
            } else {
                HourCycle::H24
            },
            date_order: if YMD_REGIONS.contains(&r) {
                DateOrder::Ymd
            } else if MDY_REGIONS.contains(&r) {
                DateOrder::Mdy
            } else {
                DateOrder::Dmy
            },
            first_day: if SUNDAY_REGIONS.contains(&r) {
                Weekday::Sunday
            } else if SATURDAY_REGIONS.contains(&r) {
                Weekday::Saturday
            } else {
                Weekday::Monday
            },
            decimal: if COMMA_DECIMAL_REGIONS.contains(&r) {
                ','
            } else {
                '.'
            },
            group: if COMMA_DECIMAL_REGIONS.contains(&r) {
                '.'
            } else {
                ','
            },
            measurement: if IMPERIAL_REGIONS.contains(&r) {
                Measurement::Imperial
            } else {
                Measurement::Metric
            },
            keyboard_layout: LAYOUT_BY_REGION
                .iter()
                .find(|(k, _)| *k == r)
                .or_else(|| {
                    LAYOUT_BY_LANGUAGE
                        .iter()
                        .find(|(k, _)| *k == language.as_str())
                })
                .map_or_else(|| "us".to_owned(), |(_, v)| (*v).to_owned()),
            tag,
            language,
            region,
            source,
        }
    }

    /// Find the locale from environment variables and system configuration.
    ///
    /// `env` is a lookup, so this is testable without touching the process
    /// environment. The order is the one every POSIX system uses: `LC_ALL`
    /// overrides everything, `LC_TIME` overrides `LANG` for anything the clock
    /// cares about.
    #[must_use]
    pub fn detect<F: Fn(&str) -> Option<String>>(env: F, config: Option<&str>) -> Self {
        for var in ["LC_ALL", "LC_TIME", "LANG"] {
            if let Some(v) = env(var)
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
            {
                return Self::from_tag(
                    &v,
                    LocaleSource::Environment {
                        variable: var.to_owned(),
                    },
                );
            }
        }
        if let Some(text) = config {
            for line in text.lines() {
                let line = line.trim();
                for key in ["LC_ALL=", "LC_TIME=", "LANG="] {
                    if let Some(v) = line.strip_prefix(key) {
                        let v = v.trim().trim_matches('"');
                        if !v.is_empty() {
                            return Self::from_tag(v, LocaleSource::SystemConfig);
                        }
                    }
                }
            }
        }
        Self::default()
    }

    /// The time of day, in this locale's convention.
    ///
    /// Never includes a zone: the caller has already converted to local time,
    /// and appending "UTC" to a local time is how a clock lies while looking
    /// precise.
    #[must_use]
    pub fn format_time(&self, dt: DateTime) -> String {
        match self.hour_cycle {
            HourCycle::H24 => format!("{:02}:{:02}", dt.hour, dt.minute),
            HourCycle::H12 => {
                let (h, suffix) = to_h12(dt.hour);
                format!("{h}:{:02} {suffix}", dt.minute)
            }
        }
    }

    /// The time of day including seconds.
    #[must_use]
    pub fn format_time_seconds(&self, dt: DateTime) -> String {
        match self.hour_cycle {
            HourCycle::H24 => format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second),
            HourCycle::H12 => {
                let (h, suffix) = to_h12(dt.hour);
                format!("{h}:{:02}:{:02} {suffix}", dt.minute, dt.second)
            }
        }
    }

    /// The date in numeric form.
    #[must_use]
    pub fn format_date(&self, dt: DateTime) -> String {
        match self.date_order {
            DateOrder::Ymd => format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day),
            DateOrder::Dmy => format!("{:02}/{:02}/{:04}", dt.day, dt.month, dt.year),
            DateOrder::Mdy => format!("{:02}/{:02}/{:04}", dt.month, dt.day, dt.year),
        }
    }

    /// A short date with the month named, for a calendar header.
    ///
    /// English month names, from [`MONTH_SHORT`]. Translating them belongs to
    /// the resource layer, which does not exist yet; naming that here is more
    /// honest than shipping a format that silently only works in English.
    #[must_use]
    pub fn format_date_medium(&self, dt: DateTime) -> String {
        let month = MONTH_SHORT
            .get(usize::from(dt.month).saturating_sub(1))
            .copied()
            .unwrap_or("?");
        match self.date_order {
            DateOrder::Ymd => format!("{} {month} {}", dt.year, dt.day),
            DateOrder::Mdy => format!("{month} {}, {}", dt.day, dt.year),
            DateOrder::Dmy => format!("{} {month} {}", dt.day, dt.year),
        }
    }

    /// The days of a week header, starting on this locale's first day.
    #[must_use]
    pub fn week_header(&self) -> Vec<&'static str> {
        let start = self.first_day.monday_index();
        (0..7)
            .map(|i| Weekday::from_monday_index(start.saturating_add(i)).short_name())
            .collect()
    }

    /// A number with this locale's separators.
    #[must_use]
    pub fn format_number(&self, value: f64, decimals: usize) -> String {
        let negative = value < 0.0;
        let text = format!("{:.*}", decimals, value.abs());
        let (int_part, frac_part) = text.split_once('.').unwrap_or((text.as_str(), ""));

        let mut grouped = String::new();
        let digits: Vec<char> = int_part.chars().collect();
        for (i, c) in digits.iter().enumerate() {
            let remaining = digits.len().saturating_sub(i);
            if i > 0 && remaining.checked_rem(3) == Some(0) {
                grouped.push(self.group);
            }
            grouped.push(*c);
        }

        let mut out = String::new();
        if negative {
            out.push('-');
        }
        out.push_str(&grouped);
        if !frac_part.is_empty() {
            out.push(self.decimal);
            out.push_str(frac_part);
        }
        out
    }

    /// A percentage, e.g. `8%` or `8,4 %` depending on the locale.
    #[must_use]
    pub fn format_percent(&self, fraction: f64, decimals: usize) -> String {
        format!("{}%", self.format_number(fraction * 100.0, decimals))
    }

    /// A temperature in this locale's units, with its symbol.
    #[must_use]
    pub fn format_temperature_c(&self, celsius: f64) -> String {
        match self.measurement {
            Measurement::Metric => format!("{}\u{00B0}C", self.format_number(celsius, 0)),
            Measurement::Imperial => {
                format!(
                    "{}\u{00B0}F",
                    self.format_number(celsius * 9.0 / 5.0 + 32.0, 0)
                )
            }
        }
    }

    /// Lines for the status file and the diagnostic overlay.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("locale.tag={}", self.tag),
            format!("locale.source={}", self.source.token()),
            format!(
                "locale.hour_cycle={}",
                match self.hour_cycle {
                    HourCycle::H12 => "12",
                    HourCycle::H24 => "24",
                }
            ),
            format!("locale.first_day={}", self.first_day.short_name()),
            format!(
                "locale.measurement={}",
                match self.measurement {
                    Measurement::Metric => "metric",
                    Measurement::Imperial => "imperial",
                }
            ),
            format!("locale.keyboard={}", self.keyboard_layout),
        ]
    }
}

/// 24-hour to 12-hour, with the suffix. Midnight is 12 AM and noon is 12 PM,
/// which is the one part of the 12-hour clock everybody gets wrong at least
/// once.
#[must_use]
fn to_h12(hour: u8) -> (u8, &'static str) {
    let suffix = if hour < 12 { "AM" } else { "PM" };
    let h = match hour.checked_rem(12) {
        Some(0) | None => 12,
        Some(h) => h,
    };
    (h, suffix)
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::civil;

    fn at(hour: u8, minute: u8) -> DateTime {
        DateTime {
            year: 2026,
            month: 8,
            day: 29,
            hour,
            minute,
            second: 5,
            weekday: Weekday::Saturday,
            yday: 241,
        }
    }

    #[test]
    fn posix_and_bcp47_tags_both_parse() {
        for raw in ["th_TH.UTF-8", "th-TH", "th_TH", "th-TH@calendar=buddhist"] {
            let l = Locale::from_tag(raw, LocaleSource::UserChoice);
            assert_eq!(l.language, "th", "{raw}");
            assert_eq!(l.region.as_deref(), Some("TH"), "{raw}");
            assert_eq!(l.tag, "th-TH");
        }
    }

    #[test]
    fn a_language_without_a_region_is_accepted() {
        let l = Locale::from_tag("pt", LocaleSource::UserChoice);
        assert_eq!(l.language, "pt");
        assert_eq!(l.region, None);
        assert_eq!(l.tag, "pt");
    }

    #[test]
    fn the_c_locale_is_the_fallback_and_is_not_american() {
        let l = Locale::default();
        assert_eq!(l.hour_cycle, HourCycle::H24);
        assert_eq!(l.date_order, DateOrder::Ymd);
        assert_eq!(l.measurement, Measurement::Metric);
        assert_eq!(l.first_day, Weekday::Monday);
        assert!(!l.source.is_known());

        let posix = Locale::from_tag("C", LocaleSource::SystemConfig);
        assert_eq!(posix.hour_cycle, HourCycle::H24);
        assert_eq!(posix.tag, "C");
    }

    #[test]
    fn a_24_hour_locale_shows_a_24_hour_clock() {
        let th = Locale::from_tag("th_TH.UTF-8", LocaleSource::UserChoice);
        assert_eq!(th.hour_cycle, HourCycle::H24);
        assert_eq!(th.format_time(at(11, 0)), "11:00");
        assert_eq!(th.format_time(at(18, 35)), "18:35");
        assert_eq!(th.format_time(at(23, 45)), "23:45");
        assert_eq!(th.format_time(at(0, 5)), "00:05");
    }

    #[test]
    fn a_12_hour_locale_shows_a_12_hour_clock() {
        let us = Locale::from_tag("en_US.UTF-8", LocaleSource::UserChoice);
        assert_eq!(us.hour_cycle, HourCycle::H12);
        assert_eq!(us.format_time(at(11, 0)), "11:00 AM");
        assert_eq!(us.format_time(at(18, 35)), "6:35 PM");
        assert_eq!(us.format_time(at(23, 45)), "11:45 PM");
    }

    #[test]
    fn midnight_and_noon_are_twelve_not_zero() {
        let us = Locale::from_tag("en-US", LocaleSource::UserChoice);
        assert_eq!(us.format_time(at(0, 0)), "12:00 AM", "midnight");
        assert_eq!(us.format_time(at(12, 0)), "12:00 PM", "noon");
        assert_eq!(us.format_time(at(12, 30)), "12:30 PM");
        assert_eq!(us.format_time(at(13, 0)), "1:00 PM");
    }

    #[test]
    fn seconds_are_available_without_changing_the_convention() {
        let us = Locale::from_tag("en-US", LocaleSource::UserChoice);
        assert_eq!(us.format_time_seconds(at(18, 35)), "6:35:05 PM");
        let de = Locale::from_tag("de-DE", LocaleSource::UserChoice);
        assert_eq!(de.format_time_seconds(at(18, 35)), "18:35:05");
    }

    #[test]
    fn date_order_follows_the_region_and_no_country_is_hard_coded() {
        let d = at(1, 0);
        assert_eq!(
            Locale::from_tag("en-US", LocaleSource::UserChoice).format_date(d),
            "08/29/2026"
        );
        assert_eq!(
            Locale::from_tag("en-GB", LocaleSource::UserChoice).format_date(d),
            "29/08/2026"
        );
        assert_eq!(
            Locale::from_tag("ja-JP", LocaleSource::UserChoice).format_date(d),
            "2026-08-29"
        );
        assert_eq!(
            Locale::from_tag("th-TH", LocaleSource::UserChoice).format_date(d),
            "29/08/2026"
        );
    }

    #[test]
    fn the_week_starts_where_the_region_starts_it() {
        let first = |tag: &str| Locale::from_tag(tag, LocaleSource::UserChoice).first_day;
        assert_eq!(first("en-US"), Weekday::Sunday);
        assert_eq!(first("th-TH"), Weekday::Sunday);
        assert_eq!(first("de-DE"), Weekday::Monday);
        assert_eq!(first("ar-EG"), Weekday::Saturday);

        let us = Locale::from_tag("en-US", LocaleSource::UserChoice);
        assert_eq!(us.week_header().first().copied(), Some("Sun"));
        let de = Locale::from_tag("de-DE", LocaleSource::UserChoice);
        assert_eq!(de.week_header().first().copied(), Some("Mon"));
        assert_eq!(de.week_header().len(), 7);
    }

    #[test]
    fn numbers_use_the_regions_separators() {
        let us = Locale::from_tag("en-US", LocaleSource::UserChoice);
        assert_eq!(us.format_number(1_234_567.25, 2), "1,234,567.25");
        let de = Locale::from_tag("de-DE", LocaleSource::UserChoice);
        assert_eq!(de.format_number(1_234_567.25, 2), "1.234.567,25");
        assert_eq!(us.format_number(-1234.5, 1), "-1,234.5");
        assert_eq!(us.format_number(0.0, 0), "0");
        assert_eq!(us.format_number(999.0, 0), "999");
        assert_eq!(us.format_number(1000.0, 0), "1,000");
    }

    #[test]
    fn percentages_follow_the_number_format() {
        let de = Locale::from_tag("de-DE", LocaleSource::UserChoice);
        assert_eq!(de.format_percent(0.084, 1), "8,4%");
        let us = Locale::from_tag("en-US", LocaleSource::UserChoice);
        assert_eq!(us.format_percent(0.08, 0), "8%");
    }

    #[test]
    fn temperature_converts_for_the_regions_that_are_not_metric() {
        let us = Locale::from_tag("en-US", LocaleSource::UserChoice);
        assert_eq!(us.measurement, Measurement::Imperial);
        assert_eq!(us.format_temperature_c(100.0), "212\u{00B0}F");
        let th = Locale::from_tag("th-TH", LocaleSource::UserChoice);
        assert_eq!(th.format_temperature_c(32.0), "32\u{00B0}C");
    }

    #[test]
    fn detection_follows_the_posix_precedence() {
        let env = |k: &str| match k {
            "LC_ALL" => Some("de_DE.UTF-8".to_owned()),
            "LANG" => Some("en_US.UTF-8".to_owned()),
            _ => None,
        };
        let l = Locale::detect(env, None);
        assert_eq!(l.tag, "de-DE", "LC_ALL must win");
        assert_eq!(
            l.source,
            LocaleSource::Environment {
                variable: "LC_ALL".to_owned()
            }
        );

        let lang_only = |k: &str| (k == "LANG").then(|| "ja_JP.UTF-8".to_owned());
        assert_eq!(Locale::detect(lang_only, None).tag, "ja-JP");
    }

    #[test]
    fn detection_falls_back_to_the_config_file_then_says_it_gave_up() {
        let none = |_: &str| None;
        let l = Locale::detect(none, Some("# comment\nLANG=\"th_TH.UTF-8\"\n"));
        assert_eq!(l.tag, "th-TH");
        assert_eq!(l.source, LocaleSource::SystemConfig);

        let empty = Locale::detect(none, None);
        assert!(!empty.source.is_known());
        assert!(empty.source.token().starts_with("fallback-c"));
    }

    #[test]
    fn an_empty_environment_variable_is_ignored_rather_than_parsed() {
        let env = |k: &str| match k {
            "LC_ALL" => Some("   ".to_owned()),
            "LANG" => Some("fr_FR.UTF-8".to_owned()),
            _ => None,
        };
        assert_eq!(Locale::detect(env, None).tag, "fr-FR");
    }

    #[test]
    fn the_keyboard_layout_comes_from_the_locale() {
        assert_eq!(
            Locale::from_tag("en-GB", LocaleSource::UserChoice).keyboard_layout,
            "gb"
        );
        assert_eq!(
            Locale::from_tag("en-US", LocaleSource::UserChoice).keyboard_layout,
            "us"
        );
        // A locale whose layout this build does not ship falls back to one it
        // does, rather than leaving the keyboard dead.
        assert_eq!(
            Locale::from_tag("th-TH", LocaleSource::UserChoice).keyboard_layout,
            "us"
        );
    }

    #[test]
    fn a_local_time_formats_the_same_instant_differently_in_two_places() {
        // The whole point of separating zone from locale: one instant, two
        // machines, two correct answers.
        let utc = 1_787_966_760; // 2026-08-29T01:26:00Z
        let bangkok = civil::from_unix(utc + 25_200);
        let new_york = civil::from_unix(utc - 5 * 3600);

        let th = Locale::from_tag("th-TH", LocaleSource::UserChoice);
        let us = Locale::from_tag("en-US", LocaleSource::UserChoice);
        assert_eq!(th.format_time(bangkok), "08:26");
        assert_eq!(us.format_time(new_york), "8:26 PM");
        assert_eq!(us.format_date(new_york), "08/28/2026", "still yesterday");
    }

    #[test]
    fn status_lines_say_where_every_setting_came_from() {
        let l = Locale::from_tag("en-GB", LocaleSource::SystemConfig);
        let lines = l.lines();
        assert!(lines.contains(&"locale.tag=en-GB".to_owned()), "{lines:?}");
        assert!(
            lines.contains(&"locale.source=/etc/locale.conf".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"locale.hour_cycle=12".to_owned()),
            "{lines:?}"
        );
    }
}
