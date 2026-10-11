//! Calendar arithmetic: seconds since the epoch to a date and back.
//!
//! Proleptic Gregorian, which is what Unix time is defined against. The
//! conversion is Howard Hinnant's `civil_from_days`, a published algorithm
//! whose correctness is established for every day in a ±32767-year range; it is
//! implemented here rather than pulled in because the crate has no
//! dependencies, and it is implemented *exactly* rather than approximately
//! because a clock that is right except in March is worse than no clock.
//!
//! Everything in this module is in **UTC**. Local time is UTC plus an offset
//! that only [`crate::tz`] knows, and mixing the two is the bug that makes a
//! desktop show the right time in the wrong place.

/// A civil date and time, with no zone attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    /// Year, e.g. 2026.
    pub year: i32,
    /// Month, 1-12.
    pub month: u8,
    /// Day of month, 1-31.
    pub day: u8,
    /// Hour, 0-23.
    pub hour: u8,
    /// Minute, 0-59.
    pub minute: u8,
    /// Second, 0-59. Leap seconds are not represented: Unix time does not have
    /// them either, and pretending otherwise would put the clock a second out
    /// for everybody in order to be right for nobody.
    pub second: u8,
    /// Day of the week.
    pub weekday: Weekday,
    /// Day of the year, 1-366.
    pub yday: u16,
}

/// A day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl Weekday {
    /// Every day, starting Monday.
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// From an index where 0 is Monday.
    #[must_use]
    pub fn from_monday_index(i: u32) -> Self {
        *Self::ALL.get((i % 7) as usize).unwrap_or(&Self::Monday)
    }

    /// Index where 0 is Monday.
    #[must_use]
    pub const fn monday_index(self) -> u32 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }

    /// Three-letter English abbreviation. Localised names come from the
    /// resource layer, not from here.
    #[must_use]
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Monday => "Mon",
            Self::Tuesday => "Tue",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
            Self::Saturday => "Sat",
            Self::Sunday => "Sun",
        }
    }
}

/// Three-letter English month abbreviations, January first.
pub const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Days in a month, accounting for leap years.
#[must_use]
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Whether a year is a leap year in the proleptic Gregorian calendar.
#[must_use]
pub const fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days since 1970-01-01 for a civil date.
///
/// Hinnant's `days_from_civil`. Every divisor below is a nonzero literal, which
/// is why the integer-division lint is allowed here and nowhere else: rewriting
/// it with checked division would obscure a published algorithm and add no
/// safety at all.
#[must_use]
#[allow(clippy::integer_division, clippy::arithmetic_side_effects)]
pub const fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let m = month as i64;
    let d = day as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 }; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// The civil date for a day number since 1970-01-01.
#[must_use]
#[allow(clippy::integer_division, clippy::arithmetic_side_effects)]
const fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u8, d as u8)
}

/// Split a signed value into a floored quotient and a non-negative remainder.
///
/// Rust's `/` and `%` truncate toward zero, so `-1 / 86400` is 0 and
/// `-1 % 86400` is -1. For a timestamp one second before the epoch that gives
/// 1970-01-01 at minus one second, which formats as hour 255 after a cast. The
/// calendar needs floor division.
#[must_use]
#[allow(clippy::integer_division, clippy::arithmetic_side_effects)]
const fn div_floor(a: i64, b: i64) -> (i64, i64) {
    let mut q = a / b;
    let mut r = a % b;
    if r < 0 {
        q -= 1;
        r += b;
    }
    (q, r)
}

/// Convert seconds since the epoch to a civil date and time.
///
/// Integer division is allowed here for the same reason as in
/// `days_from_civil`: every divisor is a literal constant from the definition
/// of a day, and expressing 3600 as a checked division would obscure the
/// arithmetic without making it safer.
#[must_use]
#[allow(clippy::arithmetic_side_effects, clippy::integer_division)]
pub const fn from_unix(secs: i64) -> DateTime {
    let (days, rem) = div_floor(secs, 86_400);
    let (year, month, day) = civil_from_days(days);

    let hour = (rem / 3600) as u8;
    let minute = ((rem % 3600) / 60) as u8;
    let second = (rem % 60) as u8;

    // 1970-01-01 was a Thursday, so day 0 is index 3 counting from Monday.
    let (_, widx) = div_floor(days + 3, 7);
    let weekday = match widx {
        0 => Weekday::Monday,
        1 => Weekday::Tuesday,
        2 => Weekday::Wednesday,
        3 => Weekday::Thursday,
        4 => Weekday::Friday,
        5 => Weekday::Saturday,
        _ => Weekday::Sunday,
    };

    let jan1 = days_from_civil(year, 1, 1);
    let yday = (days - jan1 + 1) as u16;

    DateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        weekday,
        yday,
    }
}

/// Convert a civil date and time back to seconds since the epoch.
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub const fn to_unix(dt: DateTime) -> i64 {
    days_from_civil(dt.year, dt.month, dt.day) * 86_400
        + (dt.hour as i64) * 3600
        + (dt.minute as i64) * 60
        + dt.second as i64
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_itself() {
        let d = from_unix(0);
        assert_eq!((d.year, d.month, d.day), (1970, 1, 1));
        assert_eq!((d.hour, d.minute, d.second), (0, 0, 0));
        assert_eq!(d.weekday, Weekday::Thursday, "1970-01-01 was a Thursday");
        assert_eq!(d.yday, 1);
    }

    #[test]
    fn a_known_instant_round_trips() {
        // 2026-08-29T01:26:00Z, the time on the first desktop screenshot.
        let secs = 1_787_966_760;
        let d = from_unix(secs);
        assert_eq!((d.year, d.month, d.day), (2026, 8, 29));
        assert_eq!((d.hour, d.minute), (1, 26));
        assert_eq!(to_unix(d), secs);
    }

    #[test]
    fn dates_before_the_epoch_do_not_produce_nonsense() {
        // Truncating division puts this at 1970-01-01 minus one second, which
        // becomes hour 255 after the cast to u8. Floor division is the fix.
        let d = from_unix(-1);
        assert_eq!((d.year, d.month, d.day), (1969, 12, 31));
        assert_eq!((d.hour, d.minute, d.second), (23, 59, 59));
        assert_eq!(to_unix(d), -1);
    }

    #[test]
    fn a_leap_day_exists_and_the_day_after_it_is_march() {
        let feb29 = to_unix(DateTime {
            year: 2024,
            month: 2,
            day: 29,
            hour: 12,
            minute: 0,
            second: 0,
            weekday: Weekday::Monday,
            yday: 0,
        });
        let d = from_unix(feb29);
        assert_eq!((d.year, d.month, d.day), (2024, 2, 29));
        let next = from_unix(feb29 + 86_400);
        assert_eq!((next.month, next.day), (3, 1));
    }

    #[test]
    fn century_leap_rules_are_the_gregorian_ones() {
        assert!(is_leap(2000), "divisible by 400");
        assert!(!is_leap(1900), "divisible by 100 but not 400");
        assert!(is_leap(2024));
        assert!(!is_leap(2023));
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(
            days_in_month(2026, 13),
            0,
            "an impossible month has no days"
        );
    }

    #[test]
    fn every_day_of_a_leap_year_round_trips() {
        // 366 conversions each way. If the month tables and the algorithm ever
        // disagree, this finds the day they disagree on.
        let start = days_from_civil(2024, 1, 1);
        for i in 0..366i64 {
            let secs = (start + i) * 86_400 + 43_200;
            let d = from_unix(secs);
            assert_eq!(to_unix(d), secs, "day {i} of 2024 did not round trip");
            assert_eq!(d.yday, (i + 1) as u16);
        }
    }

    #[test]
    fn weekdays_advance_by_one_per_day_and_wrap() {
        let mut expect = Weekday::Thursday;
        for i in 0..14i64 {
            let d = from_unix(i * 86_400);
            assert_eq!(d.weekday, expect, "day {i}");
            expect = Weekday::from_monday_index(expect.monday_index() + 1);
        }
    }

    #[test]
    fn a_far_future_date_is_still_right() {
        // 2100 is not a leap year, which trips implementations that only test
        // the divisible-by-four rule.
        let d = from_unix(to_unix(DateTime {
            year: 2100,
            month: 3,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            weekday: Weekday::Monday,
            yday: 0,
        }));
        assert_eq!((d.year, d.month, d.day), (2100, 3, 1));
        assert_eq!(d.yday, 60, "2100 has 59 days before March");
    }

    #[test]
    fn month_abbreviations_line_up_with_month_numbers() {
        assert_eq!(MONTH_SHORT.first().copied(), Some("Jan"));
        assert_eq!(MONTH_SHORT.get(11).copied(), Some("Dec"));
        assert_eq!(MONTH_SHORT.len(), 12);
    }
}
