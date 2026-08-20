//! Civil time boundaries.
//!
//! The ephemeris works entirely in Julian Day UT. Everything a user reads is in
//! the observer's local civil time. This module is the only place the two meet,
//! and it defers every offset and daylight saving question to the tz database
//! through `jiff` rather than doing offset arithmetic by hand.

use chandra_ephemeris::{jd_to_unix_seconds, unix_seconds_to_jd};
use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const MILLIS_PER_SECOND: f64 = 1000.0;

/// A calendar date, independent of any time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DateKey {
    pub year: i16,
    pub month: i8,
    pub day: i8,
}

impl DateKey {
    pub fn new(year: i16, month: i8, day: i8) -> Result<Self> {
        Date::new(year, month, day).map_err(|_| Error::InvalidDate { year, month, day })?;
        Ok(Self { year, month, day })
    }

    fn to_civil(self) -> Result<Date> {
        Date::new(self.year, self.month, self.day).map_err(|_| Error::InvalidDate {
            year: self.year,
            month: self.month,
            day: self.day,
        })
    }

    fn from_civil(date: Date) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}

/// An instant, carried to the front end as an epoch millisecond plus the number
/// of civil days it sits away from the day being displayed.
///
/// The front end formats the clock time itself with `Intl.DateTimeFormat` and
/// the zone name from the enclosing view, so presentation stays in the
/// presentation layer. `day_offset` is what lets a moonset after midnight render
/// as belonging to the previous day's row rather than appearing to be earlier
/// than the moonrise above it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moment {
    pub unix_ms: i64,
    pub day_offset: i32,
}

/// One civil day in a specific zone, with its boundaries expressed as Julian
/// Days so the ephemeris can be queried directly.
#[derive(Debug, Clone)]
pub struct CivilDay {
    pub date: DateKey,
    /// Julian Day (UT) of local midnight starting this day.
    pub start_jd: f64,
    /// Julian Day (UT) of local midnight starting the next day.
    pub end_jd: f64,
    zone: TimeZone,
}

impl CivilDay {
    /// Builds the day, resolving local midnight through the tz database.
    ///
    /// Where a daylight saving jump removes midnight entirely - which happens in
    /// Brazil, Lebanon and Chile among others - `jiff` returns the first instant
    /// that does exist, so the day is never zero length and never overlaps its
    /// neighbour.
    pub fn new(date: DateKey, zone: &TimeZone) -> Result<Self> {
        let civil = date.to_civil()?;
        let start = civil
            .to_zoned(zone.clone())
            .map_err(|e| Error::TimeZone(e.to_string()))?;
        let next = civil
            .tomorrow()
            .map_err(|_| Error::InvalidDate {
                year: date.year,
                month: date.month,
                day: date.day,
            })?
            .to_zoned(zone.clone())
            .map_err(|e| Error::TimeZone(e.to_string()))?;

        Ok(Self {
            date,
            start_jd: timestamp_to_jd(start.timestamp()),
            end_jd: timestamp_to_jd(next.timestamp()),
            zone: zone.clone(),
        })
    }

    /// Length of the day in days. Not always 1.0: a daylight saving transition
    /// makes it 23/24 or 25/24, which matters when a fraction of the day is used
    /// to place an event.
    pub fn length(&self) -> f64 {
        self.end_jd - self.start_jd
    }

    /// Local noon, used as the reference instant for a day's illuminated
    /// fraction. Chosen over midnight because it is the middle of the day being
    /// labelled rather than its edge, so the figure shown is representative of
    /// the day rather than of the moment it began.
    pub fn noon_jd(&self) -> f64 {
        self.start_jd + self.length() / 2.0
    }

    /// Converts a Julian Day into a [`Moment`] relative to this day.
    pub fn moment(&self, jd: f64) -> Result<Moment> {
        let unix_ms = (jd_to_unix_seconds(jd) * MILLIS_PER_SECOND).round() as i64;
        let timestamp =
            Timestamp::from_millisecond(unix_ms).map_err(|e| Error::TimeZone(e.to_string()))?;
        let local = timestamp.to_zoned(self.zone.clone()).date();
        let here = self.date.to_civil()?;

        let difference = here
            .until((jiff::Unit::Day, local))
            .map_err(|e| Error::TimeZone(e.to_string()))?;

        Ok(Moment {
            unix_ms,
            day_offset: difference.get_days(),
        })
    }

    /// The civil date a Julian Day falls on in this zone.
    pub fn date_of(&self, jd: f64) -> Result<DateKey> {
        let unix_ms = (jd_to_unix_seconds(jd) * MILLIS_PER_SECOND).round() as i64;
        let timestamp =
            Timestamp::from_millisecond(unix_ms).map_err(|e| Error::TimeZone(e.to_string()))?;
        Ok(DateKey::from_civil(
            timestamp.to_zoned(self.zone.clone()).date(),
        ))
    }
}

fn timestamp_to_jd(timestamp: Timestamp) -> f64 {
    unix_seconds_to_jd(timestamp.as_millisecond() as f64 / MILLIS_PER_SECOND)
}

/// Number of days in a month, from the calendar rather than a lookup table, so
/// leap years need no special case.
pub fn days_in_month(year: i16, month: i8) -> Result<u8> {
    let first = Date::new(year, month, 1).map_err(|_| Error::InvalidDate {
        year,
        month,
        day: 1,
    })?;
    Ok(first.days_in_month() as u8)
}

/// Weekday of the first of the month as a zero-based offset from Monday, used to
/// place the first cell in the grid.
pub fn first_weekday_offset(year: i16, month: i8) -> Result<u8> {
    let first = Date::new(year, month, 1).map_err(|_| Error::InvalidDate {
        year,
        month,
        day: 1,
    })?;
    Ok(first.weekday().to_monday_zero_offset() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone(name: &str) -> TimeZone {
        TimeZone::get(name).expect("tz database must be available")
    }

    #[test]
    fn a_normal_day_is_exactly_twenty_four_hours() {
        let day = CivilDay::new(DateKey::new(2026, 8, 20).unwrap(), &zone("Asia/Kolkata")).unwrap();
        assert!((day.length() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn daylight_saving_changes_the_length_of_the_day() {
        // 2026-03-08 is the US spring transition: a 23 hour day.
        let short =
            CivilDay::new(DateKey::new(2026, 3, 8).unwrap(), &zone("America/New_York")).unwrap();
        assert!(
            (short.length() - 23.0 / 24.0).abs() < 1e-9,
            "got {} hours",
            short.length() * 24.0
        );

        // 2026-11-01 is the autumn transition: a 25 hour day.
        let long = CivilDay::new(
            DateKey::new(2026, 11, 1).unwrap(),
            &zone("America/New_York"),
        )
        .unwrap();
        assert!(
            (long.length() - 25.0 / 24.0).abs() < 1e-9,
            "got {} hours",
            long.length() * 24.0
        );
    }

    #[test]
    fn a_zone_that_skips_midnight_still_yields_a_full_day() {
        // Lebanon moved its spring transition to midnight; 2026-03-29 has no
        // 00:00 local. The day must still be well formed and non-overlapping.
        let day = CivilDay::new(DateKey::new(2026, 3, 29).unwrap(), &zone("Asia/Beirut")).unwrap();
        assert!(day.end_jd > day.start_jd, "day must have positive length");
        assert!(day.length() <= 1.0);
        let previous =
            CivilDay::new(DateKey::new(2026, 3, 28).unwrap(), &zone("Asia/Beirut")).unwrap();
        assert!(
            previous.end_jd <= day.start_jd + 1e-9,
            "consecutive days must not overlap"
        );
    }

    #[test]
    fn day_offset_places_an_after_midnight_event_on_the_next_day() {
        let day = CivilDay::new(DateKey::new(2026, 8, 20).unwrap(), &zone("Asia/Kolkata")).unwrap();

        let same_day = day.moment(day.start_jd + 0.5).unwrap();
        assert_eq!(same_day.day_offset, 0);

        // Half an hour past local midnight.
        let after = day.moment(day.end_jd + 0.5 / 24.0).unwrap();
        assert_eq!(after.day_offset, 1);

        // An hour before the day began.
        let before = day.moment(day.start_jd - 1.0 / 24.0).unwrap();
        assert_eq!(before.day_offset, -1);
    }

    #[test]
    fn month_lengths_follow_the_calendar() {
        assert_eq!(days_in_month(2026, 2).unwrap(), 28);
        assert_eq!(days_in_month(2024, 2).unwrap(), 29);
        assert_eq!(days_in_month(2000, 2).unwrap(), 29);
        assert_eq!(days_in_month(1900, 2).unwrap(), 28);
        assert_eq!(days_in_month(2026, 8).unwrap(), 31);
        assert_eq!(days_in_month(2026, 4).unwrap(), 30);
    }

    #[test]
    fn first_weekday_offset_is_monday_based() {
        // 2026-08-01 is a Saturday: five days past Monday.
        assert_eq!(first_weekday_offset(2026, 8).unwrap(), 5);
        // 2026-06-01 is a Monday.
        assert_eq!(first_weekday_offset(2026, 6).unwrap(), 0);
    }

    #[test]
    fn invalid_dates_are_rejected_rather_than_clamped() {
        assert!(DateKey::new(2026, 2, 30).is_err());
        assert!(DateKey::new(2026, 13, 1).is_err());
        assert!(DateKey::new(2023, 2, 29).is_err());
        assert!(DateKey::new(2024, 2, 29).is_ok());
    }
}
