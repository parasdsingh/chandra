//! Julian Day conversions.
//!
//! Implemented in pure Rust rather than through `swe_julday` so that date
//! arithmetic does not need the engine lock. Correctness against Swiss
//! Ephemeris is asserted by test rather than assumed.

/// Julian Day of the Unix epoch, 1970-01-01T00:00:00Z.
pub const JD_UNIX_EPOCH: f64 = 2_440_587.5;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// Julian Day (UT) for a proleptic Gregorian calendar date and time of day.
///
/// `day_fraction` is the fraction of a day elapsed since 00:00 UT, so 12:00 UT
/// is `0.5`. Meeus, *Astronomical Algorithms*, chapter 7, Gregorian branch only:
/// the app never navigates before 1582, and silently switching calendars at a
/// hidden boundary is worse than not supporting it.
pub fn julian_day(year: i32, month: u32, day: u32, day_fraction: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month as i32 + 12)
    } else {
        (year, month as i32)
    };

    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y as f64 + 4716.0)).floor() + (30.6001 * (m as f64 + 1.0)).floor() + day as f64 + b
        - 1524.5
        + day_fraction
}

/// Calendar date and day fraction for a Julian Day. Inverse of [`julian_day`].
pub fn from_julian_day(jd: f64) -> (i32, u32, u32, f64) {
    let z_and_f = jd + 0.5;
    let z = z_and_f.floor();
    let f = z_and_f - z;

    let alpha = ((z - 1_867_216.25) / 36_524.25).floor();
    let a = z + 1.0 + alpha - (alpha / 4.0).floor();
    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();

    let day = (b - d - (30.6001 * e).floor()) as u32;
    let month = if e < 14.0 { e - 1.0 } else { e - 13.0 } as u32;
    let year = if month > 2 { c - 4716.0 } else { c - 4715.0 } as i32;

    (year, month, day, f)
}

/// Seconds since the Unix epoch for a Julian Day in UT.
///
/// UT1 and UTC differ by under 0.9 s by definition of leap seconds. That
/// difference is below the display resolution of every time this app shows and
/// is deliberately ignored.
pub fn jd_to_unix_seconds(jd: f64) -> f64 {
    (jd - JD_UNIX_EPOCH) * SECONDS_PER_DAY
}

/// Inverse of [`jd_to_unix_seconds`].
pub fn unix_seconds_to_jd(seconds: f64) -> f64 {
    JD_UNIX_EPOCH + seconds / SECONDS_PER_DAY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_epochs() {
        // Meeus worked examples.
        assert!((julian_day(2000, 1, 1, 0.5) - 2_451_545.0).abs() < 1e-9);
        assert!((julian_day(1987, 1, 27, 0.0) - 2_446_822.5).abs() < 1e-9);
        assert!((julian_day(1988, 6, 19, 0.5) - 2_447_332.0).abs() < 1e-9);
        assert!((julian_day(1970, 1, 1, 0.0) - JD_UNIX_EPOCH).abs() < 1e-9);
    }

    #[test]
    fn round_trips_across_a_wide_range() {
        for year in [1800, 1900, 1999, 2000, 2026, 2100, 2399] {
            for month in 1..=12 {
                for day in [1, 15, 28] {
                    let jd = julian_day(year, month, day, 0.25);
                    let (y, m, d, f) = from_julian_day(jd);
                    assert_eq!((y, m, d), (year, month, day), "jd {jd}");
                    assert!((f - 0.25).abs() < 1e-9, "fraction drift at {jd}");
                }
            }
        }
    }

    #[test]
    fn unix_seconds_round_trip() {
        let jd = julian_day(2026, 8, 20, 0.5);
        assert!((unix_seconds_to_jd(jd_to_unix_seconds(jd)) - jd).abs() < 1e-9);
        // 2000-01-01T12:00:00Z is 946_728_000 Unix seconds.
        assert!((jd_to_unix_seconds(2_451_545.0) - 946_728_000.0).abs() < 1e-6);
    }
}
