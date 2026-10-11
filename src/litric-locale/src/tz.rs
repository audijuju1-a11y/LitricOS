//! IANA time zones, read from the system's own `zoneinfo` database.
//!
//! The clock in the top bar has to show the time where the machine is. Doing
//! that correctly means three separate things, and conflating any two of them
//! is how a desktop ends up an hour out twice a year:
//!
//! * **UTC** is what the kernel counts. It has no zone and no summer time.
//! * **A time zone** is a rule: which offset applies at which instant. Not a
//!   number — `Europe/London` is +0 in January and +1 in July.
//! * **A locale** is a preference: 24-hour or 12-hour, day before month. It
//!   has nothing to do with where the machine is.
//!
//! This module does the middle one, by parsing TZif files (RFC 8536) straight
//! out of `/usr/share/zoneinfo`. No table of offsets is compiled in: a built-in
//! table is a table that is wrong the next time a country changes its mind, and
//! several do every year.
//!
//! # If the zone cannot be determined
//!
//! It reports UTC **and says so**. A desktop that guesses a zone shows a
//! confident wrong time, which is worse than an honest one that says the
//! machine has not been told where it is.

use std::fs;
use std::path::Path;

use crate::civil;

/// One set of rules that applied over some interval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offset {
    /// Seconds east of UTC. Negative in the Americas.
    pub seconds: i32,
    /// Whether this is a daylight-saving offset.
    pub is_dst: bool,
    /// The abbreviation in force, e.g. `+07`, `BST`, `PDT`.
    pub abbrev: String,
}

impl Offset {
    /// UTC itself.
    #[must_use]
    pub fn utc() -> Self {
        Self {
            seconds: 0,
            is_dst: false,
            abbrev: "UTC".to_owned(),
        }
    }

    /// The offset as `+07:00` / `-05:30`.
    #[must_use]
    pub fn iso(&self) -> String {
        let sign = if self.seconds < 0 { '-' } else { '+' };
        let total = self.seconds.unsigned_abs();
        let h = total.checked_div(3600).unwrap_or(0);
        let m = total
            .checked_rem(3600)
            .unwrap_or(0)
            .checked_div(60)
            .unwrap_or(0);
        format!("{sign}{h:02}:{m:02}")
    }
}

/// Where a time zone came from. Recorded so the diagnostics can say whether the
/// clock is authoritative or a fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ZoneSource {
    /// The `TZ` environment variable named it.
    Environment,
    /// `/etc/localtime` is a symlink into the zoneinfo database.
    LocaltimeLink,
    /// `/etc/localtime` is a copy, named by `/etc/timezone`.
    LocaltimeCopy,
    /// Nothing said where the machine is.
    FallbackUtc {
        /// Why.
        reason: String,
    },
}

impl ZoneSource {
    /// Whether the zone is known rather than assumed.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        !matches!(self, Self::FallbackUtc { .. })
    }

    /// A short token for the status file.
    #[must_use]
    pub fn token(&self) -> String {
        match self {
            Self::Environment => "TZ".to_owned(),
            Self::LocaltimeLink => "/etc/localtime".to_owned(),
            Self::LocaltimeCopy => "/etc/timezone".to_owned(),
            Self::FallbackUtc { reason } => format!("fallback-utc({reason})"),
        }
    }
}

/// A parsed time zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZone {
    name: String,
    /// Transition instants, ascending, with the index of the offset that starts
    /// at each.
    transitions: Vec<(i64, usize)>,
    offsets: Vec<Offset>,
    /// Which offset applies before the first transition.
    initial: usize,
    source: ZoneSource,
}

impl TimeZone {
    /// UTC, with a reason for why it was chosen.
    #[must_use]
    pub fn utc(reason: &str) -> Self {
        Self {
            name: "UTC".to_owned(),
            transitions: Vec::new(),
            offsets: vec![Offset::utc()],
            initial: 0,
            source: ZoneSource::FallbackUtc {
                reason: reason.to_owned(),
            },
        }
    }

    /// The zone's IANA name, e.g. `Asia/Bangkok`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Where the zone came from.
    #[must_use]
    pub const fn source(&self) -> &ZoneSource {
        &self.source
    }

    /// The offset in force at a UTC instant.
    ///
    /// Binary search over the transition table, which is what makes a zone with
    /// two hundred years of rules cost eight comparisons rather than two
    /// hundred.
    #[must_use]
    pub fn offset_at(&self, utc: i64) -> &Offset {
        let idx = match self.transitions.binary_search_by(|(t, _)| t.cmp(&utc)) {
            Ok(i) => self.transitions.get(i).map_or(self.initial, |(_, o)| *o),
            Err(0) => self.initial,
            Err(i) => self
                .transitions
                .get(i.saturating_sub(1))
                .map_or(self.initial, |(_, o)| *o),
        };
        self.offsets
            .get(idx)
            .or_else(|| self.offsets.first())
            .unwrap_or(
                const {
                    &Offset {
                        seconds: 0,
                        is_dst: false,
                        abbrev: String::new(),
                    }
                },
            )
    }

    /// Local civil time for a UTC instant.
    #[must_use]
    pub fn local(&self, utc: i64) -> civil::DateTime {
        let off = self.offset_at(utc);
        civil::from_unix(utc.saturating_add(i64::from(off.seconds)))
    }

    /// The next transition strictly after `utc`, if the table has one.
    ///
    /// The clock uses this to know when it must redraw for a reason other than
    /// the minute changing — which is the whole of DST support: nothing special
    /// happens at the boundary except that the answer changes.
    #[must_use]
    pub fn next_transition(&self, utc: i64) -> Option<i64> {
        self.transitions
            .iter()
            .find(|(t, _)| *t > utc)
            .map(|(t, _)| *t)
    }

    /// How many transitions the zone knows about. Zero means a fixed offset.
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    /// Parse a TZif file.
    ///
    /// Returns `None` for anything that is not a TZif file, rather than a zone
    /// with no rules: a silently empty zone is a clock that is confidently
    /// wrong by an hour for half the year.
    #[must_use]
    pub fn parse(name: &str, data: &[u8], source: ZoneSource) -> Option<Self> {
        let head = Header::parse(data, 0)?;

        // Version 2 and later repeat everything in 64-bit form after the 32-bit
        // block. The 64-bit block is the authoritative one and is the only one
        // that covers dates past 2038, so it is preferred when present.
        let (head, base) = if head.version >= b'2' {
            let after_v1 = head.data_len(4)?;
            let second_start = 44usize.checked_add(after_v1)?;
            let h2 = Header::parse(data, second_start)?;
            let base = second_start.checked_add(44)?;
            (h2, base)
        } else {
            (head, 44usize)
        };

        let time_size = if head.version >= b'2' { 8 } else { 4 };
        let mut at = base;

        let mut times = Vec::with_capacity(head.timecnt);
        for _ in 0..head.timecnt {
            let v = if time_size == 8 {
                read_i64(data, at)?
            } else {
                i64::from(read_i32(data, at)?)
            };
            times.push(v);
            at = at.checked_add(time_size)?;
        }

        let mut indices = Vec::with_capacity(head.timecnt);
        for _ in 0..head.timecnt {
            indices.push(usize::from(*data.get(at)?));
            at = at.checked_add(1)?;
        }

        let mut raw_types = Vec::with_capacity(head.typecnt);
        for _ in 0..head.typecnt {
            let utoff = read_i32(data, at)?;
            let is_dst = *data.get(at.checked_add(4)?)? != 0;
            let idx = usize::from(*data.get(at.checked_add(5)?)?);
            raw_types.push((utoff, is_dst, idx));
            at = at.checked_add(6)?;
        }

        let chars = data.get(at..at.checked_add(head.charcnt)?)?;

        let offsets: Vec<Offset> = raw_types
            .iter()
            .map(|(utoff, is_dst, idx)| Offset {
                seconds: *utoff,
                is_dst: *is_dst,
                abbrev: abbrev_at(chars, *idx),
            })
            .collect();
        if offsets.is_empty() {
            return None;
        }

        let transitions: Vec<(i64, usize)> = times
            .iter()
            .zip(indices.iter())
            .map(|(t, i)| (*t, *i))
            .collect();

        // Before the first transition, POSIX says to use the first
        // non-daylight type, falling back to the first type of all.
        let initial = offsets.iter().position(|o| !o.is_dst).unwrap_or(0);

        Some(Self {
            name: name.to_owned(),
            transitions,
            offsets,
            initial,
            source,
        })
    }
}

fn abbrev_at(chars: &[u8], idx: usize) -> String {
    let tail = chars.get(idx..).unwrap_or(&[]);
    let end = tail.iter().position(|c| *c == 0).unwrap_or(tail.len());
    String::from_utf8_lossy(tail.get(..end).unwrap_or(&[])).into_owned()
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    let s = data.get(at..at.checked_add(4)?)?;
    Some(u32::from_be_bytes([
        *s.first()?,
        *s.get(1)?,
        *s.get(2)?,
        *s.get(3)?,
    ]))
}

fn read_i32(data: &[u8], at: usize) -> Option<i32> {
    read_u32(data, at).map(|v| v as i32)
}

fn read_i64(data: &[u8], at: usize) -> Option<i64> {
    let s = data.get(at..at.checked_add(8)?)?;
    let mut b = [0u8; 8];
    for (i, dst) in b.iter_mut().enumerate() {
        *dst = *s.get(i)?;
    }
    Some(i64::from_be_bytes(b))
}

/// A TZif header.
#[derive(Debug, Clone, Copy)]
struct Header {
    version: u8,
    isutcnt: usize,
    isstdcnt: usize,
    leapcnt: usize,
    timecnt: usize,
    typecnt: usize,
    charcnt: usize,
}

impl Header {
    fn parse(data: &[u8], at: usize) -> Option<Self> {
        if data.get(at..at.checked_add(4)?)? != b"TZif" {
            return None;
        }
        let version = *data.get(at.checked_add(4)?)?;
        let n = |k: usize| -> Option<usize> {
            read_u32(data, at.checked_add(20)?.checked_add(k.checked_mul(4)?)?).map(|v| v as usize)
        };
        Some(Self {
            version,
            isutcnt: n(0)?,
            isstdcnt: n(1)?,
            leapcnt: n(2)?,
            timecnt: n(3)?,
            typecnt: n(4)?,
            charcnt: n(5)?,
        })
    }

    /// Bytes in the data block that follows a header with this time size.
    fn data_len(&self, time_size: usize) -> Option<usize> {
        let leap_size = time_size.checked_add(4)?;
        self.timecnt
            .checked_mul(time_size)?
            .checked_add(self.timecnt)?
            .checked_add(self.typecnt.checked_mul(6)?)?
            .checked_add(self.charcnt)?
            .checked_add(self.leapcnt.checked_mul(leap_size)?)?
            .checked_add(self.isstdcnt)?
            .checked_add(self.isutcnt)
    }
}

/// Find the machine's time zone.
///
/// In the order the system itself resolves it:
/// 1. `TZ`, which overrides everything.
/// 2. `/etc/localtime` as a symlink into the zoneinfo database, which is how
///    every current distribution records it.
/// 3. `/etc/localtime` as a copy, with the name in `/etc/timezone`.
/// 4. UTC, labelled as a fallback with the reason.
#[must_use]
pub fn detect(root: &Path, tz_env: Option<&str>) -> TimeZone {
    if let Some(name) = tz_env.map(str::trim).filter(|s| !s.is_empty()) {
        // A leading colon is the POSIX way of saying "this is a file name".
        let name = name.strip_prefix(':').unwrap_or(name);
        if let Some(zone) = load_named(root, name, ZoneSource::Environment) {
            return zone;
        }
        return TimeZone::utc(&format!("TZ={name} is not in the zoneinfo database"));
    }

    let localtime = root.join("etc/localtime");
    if let Ok(target) = fs::read_link(&localtime) {
        let s = target.to_string_lossy().replace('\\', "/");
        if let Some(name) = s.split("zoneinfo/").nth(1) {
            if let Some(zone) = load_named(root, name, ZoneSource::LocaltimeLink) {
                return zone;
            }
        }
    }

    if let Ok(data) = fs::read(&localtime) {
        let name = fs::read_to_string(root.join("etc/timezone"))
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "localtime".to_owned());
        if let Some(zone) = TimeZone::parse(&name, &data, ZoneSource::LocaltimeCopy) {
            return zone;
        }
        return TimeZone::utc("/etc/localtime is not a TZif file");
    }

    TimeZone::utc("no TZ and no /etc/localtime")
}

/// Load a named zone from the database under `root`.
#[must_use]
pub fn load_named(root: &Path, name: &str, source: ZoneSource) -> Option<TimeZone> {
    // A zone name comes from the environment, so it must never be able to
    // escape the database. `Asia/Bangkok` is a name; `../../etc/shadow` is an
    // attempt.
    if name.contains("..") || name.starts_with('/') || name.is_empty() {
        return None;
    }
    let path = root.join("usr/share/zoneinfo").join(name);
    let data = fs::read(path).ok()?;
    TimeZone::parse(name, &data, source)
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
mod tests {
    use super::*;

    /// Build a minimal version-1 TZif file with the given transitions.
    fn tzif_v1(types: &[(i32, bool, &str)], transitions: &[(i32, u8)]) -> Vec<u8> {
        let mut chars = Vec::new();
        let mut idx = Vec::new();
        for (_, _, abbr) in types {
            idx.push(chars.len() as u8);
            chars.extend_from_slice(abbr.as_bytes());
            chars.push(0);
        }
        let mut out = Vec::new();
        out.extend_from_slice(b"TZif");
        out.push(0); // version 1
        out.extend_from_slice(&[0u8; 15]);
        for v in [
            0u32,
            0,
            0,
            transitions.len() as u32,
            types.len() as u32,
            chars.len() as u32,
        ] {
            out.extend_from_slice(&v.to_be_bytes());
        }
        for (t, _) in transitions {
            out.extend_from_slice(&t.to_be_bytes());
        }
        for (_, i) in transitions {
            out.push(*i);
        }
        for (n, (off, dst, _)) in types.iter().enumerate() {
            out.extend_from_slice(&off.to_be_bytes());
            out.push(u8::from(*dst));
            out.push(idx[n]);
        }
        out.extend_from_slice(&chars);
        out
    }

    #[test]
    fn a_fixed_offset_zone_parses_and_applies_everywhere() {
        // Asia/Bangkok has been +07 with no daylight saving since 1920.
        let data = tzif_v1(&[(25_200, false, "+07")], &[]);
        let z = TimeZone::parse("Asia/Bangkok", &data, ZoneSource::LocaltimeLink).expect("parses");
        assert_eq!(z.name(), "Asia/Bangkok");
        assert_eq!(z.offset_at(0).seconds, 25_200);
        assert_eq!(z.offset_at(1_787_966_760).abbrev, "+07");
        assert_eq!(z.offset_at(i64::MAX).seconds, 25_200);
        assert_eq!(z.transition_count(), 0);
        assert!(z.source().is_known());
    }

    #[test]
    fn a_zone_with_daylight_saving_changes_at_the_transition() {
        // Standard +0, summer +1, switching at t=1000 and back at t=2000.
        let data = tzif_v1(
            &[(0, false, "GMT"), (3600, true, "BST")],
            &[(1000, 1), (2000, 0)],
        );
        let z = TimeZone::parse("Europe/London", &data, ZoneSource::LocaltimeLink).expect("parses");
        assert_eq!(z.offset_at(999).seconds, 0, "before the change");
        assert_eq!(z.offset_at(1000).seconds, 3600, "at the change");
        assert_eq!(z.offset_at(1500).abbrev, "BST");
        assert!(z.offset_at(1500).is_dst);
        assert_eq!(z.offset_at(2000).seconds, 0, "back to standard");
        assert!(!z.offset_at(2000).is_dst);
    }

    #[test]
    fn the_next_transition_is_findable_so_the_clock_knows_when_to_redraw() {
        let data = tzif_v1(
            &[(0, false, "GMT"), (3600, true, "BST")],
            &[(1000, 1), (2000, 0)],
        );
        let z = TimeZone::parse("Europe/London", &data, ZoneSource::LocaltimeLink).expect("parses");
        assert_eq!(z.next_transition(0), Some(1000));
        assert_eq!(z.next_transition(1000), Some(2000));
        assert_eq!(z.next_transition(5000), None);
    }

    #[test]
    fn local_time_is_utc_plus_the_offset_in_force() {
        let data = tzif_v1(&[(25_200, false, "+07")], &[]);
        let z = TimeZone::parse("Asia/Bangkok", &data, ZoneSource::LocaltimeLink).expect("parses");
        // 2026-08-29T01:26:00Z is 08:26 in Bangkok.
        let d = z.local(1_787_966_760);
        assert_eq!((d.hour, d.minute), (8, 26));
        assert_eq!((d.year, d.month, d.day), (2026, 8, 29));
    }

    #[test]
    fn a_negative_offset_moves_the_date_backwards() {
        // New York is -05:00 in winter, so 02:00 UTC is 21:00 the day before.
        let data = tzif_v1(&[(-18_000, false, "EST")], &[]);
        let z =
            TimeZone::parse("America/New_York", &data, ZoneSource::LocaltimeLink).expect("parses");
        let utc = civil::to_unix(civil::DateTime {
            year: 2026,
            month: 1,
            day: 2,
            hour: 2,
            minute: 0,
            second: 0,
            weekday: civil::Weekday::Friday,
            yday: 0,
        });
        let d = z.local(utc);
        assert_eq!((d.year, d.month, d.day), (2026, 1, 1));
        assert_eq!(d.hour, 21);
        assert_eq!(z.offset_at(utc).iso(), "-05:00");
    }

    #[test]
    fn offsets_that_are_not_whole_hours_format_correctly() {
        // Kathmandu is +05:45. An implementation that only prints hours shows
        // the wrong offset for a fifth of the world's population.
        let o = Offset {
            seconds: 20_700,
            is_dst: false,
            abbrev: "+0545".to_owned(),
        };
        assert_eq!(o.iso(), "+05:45");
        let neg = Offset {
            seconds: -12_600,
            is_dst: false,
            abbrev: "-0330".to_owned(),
        };
        assert_eq!(neg.iso(), "-03:30");
    }

    #[test]
    fn a_file_that_is_not_tzif_is_refused_rather_than_producing_an_empty_zone() {
        assert!(TimeZone::parse("x", b"not a tzif file", ZoneSource::LocaltimeLink).is_none());
        assert!(TimeZone::parse("x", b"", ZoneSource::LocaltimeLink).is_none());
        assert!(TimeZone::parse("x", b"TZif", ZoneSource::LocaltimeLink).is_none());
    }

    #[test]
    fn a_truncated_tzif_is_refused_rather_than_read_past_its_end() {
        let full = tzif_v1(&[(3600, false, "CET")], &[(100, 0)]);
        for cut in 1..full.len() {
            let _ = TimeZone::parse("x", &full[..cut], ZoneSource::LocaltimeLink);
            // Reaching here without a panic is the assertion. Every read in the
            // parser is bounds-checked; a file that stops in the middle of a
            // transition table must produce None, not an index out of range.
        }
        assert!(TimeZone::parse("x", &full, ZoneSource::LocaltimeLink).is_some());
    }

    #[test]
    fn utc_is_labelled_as_a_fallback_with_the_reason() {
        let z = TimeZone::utc("no TZ and no /etc/localtime");
        assert_eq!(z.name(), "UTC");
        assert_eq!(z.offset_at(12_345).seconds, 0);
        assert!(
            !z.source().is_known(),
            "a fallback must not look authoritative"
        );
        assert!(z.source().token().contains("no TZ"));
    }

    #[test]
    fn a_zone_name_cannot_escape_the_database() {
        // The name comes from the environment. Without this check, TZ can read
        // any file the shell can.
        let root = std::env::temp_dir().join("litric-locale-escape");
        assert!(load_named(&root, "../../etc/shadow", ZoneSource::Environment).is_none());
        assert!(load_named(&root, "/etc/shadow", ZoneSource::Environment).is_none());
        assert!(load_named(&root, "", ZoneSource::Environment).is_none());
    }

    #[test]
    fn detection_prefers_tz_then_localtime_then_says_it_gave_up() {
        let root = std::env::temp_dir().join("litric-locale-detect");
        let _ = fs::remove_dir_all(&root);
        let zoneinfo = root.join("usr/share/zoneinfo/Asia");
        fs::create_dir_all(&zoneinfo).expect("dirs");
        fs::create_dir_all(root.join("etc")).expect("etc");
        fs::write(
            zoneinfo.join("Bangkok"),
            tzif_v1(&[(25_200, false, "+07")], &[]),
        )
        .expect("zone file");

        let by_env = detect(&root, Some("Asia/Bangkok"));
        assert_eq!(by_env.name(), "Asia/Bangkok");
        assert_eq!(by_env.source(), &ZoneSource::Environment);
        assert_eq!(by_env.offset_at(0).seconds, 25_200);

        // A copy rather than a symlink: Windows filesystems cannot hold one,
        // and container images usually ship a copy too.
        fs::write(
            root.join("etc/localtime"),
            tzif_v1(&[(25_200, false, "+07")], &[]),
        )
        .expect("localtime");
        fs::write(root.join("etc/timezone"), "Asia/Bangkok\n").expect("timezone");
        let by_file = detect(&root, None);
        assert_eq!(by_file.name(), "Asia/Bangkok");
        assert_eq!(by_file.source(), &ZoneSource::LocaltimeCopy);

        let unknown = detect(&root, Some("Mars/Olympus"));
        assert_eq!(unknown.name(), "UTC");
        assert!(!unknown.source().is_known());

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_machine_that_was_never_told_where_it_is_reports_utc_honestly() {
        let z = detect(Path::new("no-such-root-7f31"), None);
        assert_eq!(z.name(), "UTC");
        assert!(!z.source().is_known());
        assert!(z.source().token().starts_with("fallback-utc"));
    }
}
