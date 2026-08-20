//! Assembly of a month view: one cell per day, plus events for graha months.
//!
//! Month views carry only what the grid draws. Everything else - entry and exit
//! times, rise and set, degrees - is computed when a day is actually opened, so
//! opening a month never pays for detail nobody looked at.

use chandra_ephemeris::{Engine, Graha, Observer, Source};
use serde::{Deserialize, Serialize};

use crate::day::{graha_day, moon_day, GrahaDay, MoonDay};
use crate::error::Result;
use crate::events::{combustion_orb, events_in_month, Event};
use crate::lunar::MonthSystem;
use crate::phase::{self, PhaseName};
use crate::time::{days_in_month, CivilDay, DateKey};
use crate::zodiac::{Nakshatra, Rashi};

/// A month of moon phases.
///
/// The month may be a Gregorian month or a lunar one; the difference is carried
/// entirely by `label`, `system` and which civil days appear in `days`. The grid
/// that draws it does not need to know which it is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonMonth {
    /// What the header shows: `August 2026`, `Shravana 2026`, `Adhika Shravana
    /// 2026`.
    pub label: String,
    pub system: MonthSystem,
    /// An instant inside this month, used to navigate to its neighbours. A lunar
    /// month has no year-and-number to step through, so the cursor is a time.
    pub anchor_unix_ms: i64,
    /// IANA zone name. The front end formats every timestamp against this rather
    /// than against the machine's own zone.
    pub time_zone: String,
    pub days: Vec<MoonCell>,
    pub source: Source,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MoonCell {
    /// Full civil date: a lunar month crosses Gregorian month boundaries, so a
    /// day-of-month alone would be ambiguous.
    pub date: DateKey,
    /// Illuminated fraction at local noon, 0.0 to 1.0.
    pub illumination: f64,
    pub is_waxing: bool,
    pub phase: PhaseName,
    /// True when a principal phase falls on this day. The grid marks these.
    pub principal: bool,
    /// Within the Sun's rays: the Moon's orb is 12 degrees, so this covers the
    /// days either side of new moon when it cannot be seen.
    pub combust: bool,
}

/// A month of one graha's transit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahaMonth {
    pub graha: Graha,
    pub label: String,
    pub system: MonthSystem,
    pub anchor_unix_ms: i64,
    pub time_zone: String,
    pub days: Vec<GrahaCell>,
    pub events: Vec<Event>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahaCell {
    pub date: DateKey,
    pub longitude: f64,
    pub rashi: Rashi,
    pub nakshatra: Nakshatra,
    pub retrograde: bool,
    pub speed: f64,
    /// Within the Sun's rays at this day's reference instant.
    pub combust: bool,
}

/// The civil days of a Gregorian month in the observer's zone.
pub fn month_days(year: i16, month: i8, zone: &jiff::tz::TimeZone) -> Result<Vec<CivilDay>> {
    let count = days_in_month(year, month)?;
    (1..=count as i8)
        .map(|day| CivilDay::new(DateKey::new(year, month, day)?, zone))
        .collect()
}

/// The civil days from `first` to `last` inclusive, for a lunar month.
pub fn days_between(
    first: DateKey,
    last: DateKey,
    zone: &jiff::tz::TimeZone,
) -> Result<Vec<CivilDay>> {
    let mut days = Vec::new();
    let mut date = first;

    // A lunar month is 29 or 30 civil days; the bound guards against a malformed
    // range turning into an unbounded loop.
    for _ in 0..40 {
        let day = CivilDay::new(date, zone)?;
        let reached_end = date == last;
        date = day.date_of(day.end_jd + 0.5)?;
        days.push(day);
        if reached_end {
            break;
        }
    }
    Ok(days)
}

pub fn moon_month(
    engine: &Engine,
    days: &[CivilDay],
    zone_name: &str,
    label: String,
    system: MonthSystem,
) -> Result<MoonMonth> {
    let mut cells = Vec::with_capacity(days.len());
    let mut sources = Vec::with_capacity(days.len());

    // Elongation at each day boundary. Consecutive days share a boundary, so
    // carrying the previous value forward halves the ephemeris calls.
    let mut elongation_start = elongation(engine, days[0].start_jd)?;

    let moon_orb = combustion_orb(Graha::Chandra, false).unwrap_or(0.0);

    for day in days {
        let elongation_end = elongation(engine, day.end_jd)?;
        let illumination = engine.illumination(day.noon_jd())?;
        let principal = phase::principal_phase_in(elongation_start, elongation_end);

        // Elongation at local noon, folded to the shorter way round, is the
        // distance from the Sun the combustion orb is measured against.
        let noon_elongation = elongation(engine, day.noon_jd())?;
        let separation = noon_elongation.min(360.0 - noon_elongation);

        cells.push(MoonCell {
            date: day.date,
            illumination: illumination.fraction,
            is_waxing: phase::is_waxing(elongation_start),
            phase: principal
                .map(|(name, _)| name)
                .unwrap_or_else(|| phase::intermediate_phase(elongation_start)),
            principal: principal.is_some(),
            combust: separation < moon_orb,
        });
        sources.push(illumination.source);
        elongation_start = elongation_end;
    }

    Ok(MoonMonth {
        label,
        system,
        anchor_unix_ms: anchor(days),
        time_zone: zone_name.to_string(),
        days: cells,
        source: Source::weakest(sources),
    })
}

/// An instant safely inside the month, used as its navigation cursor.
///
/// Local noon of the middle day: far from both boundaries, so stepping from it
/// cannot land back in the same month through a rounding accident.
fn anchor(days: &[CivilDay]) -> i64 {
    let middle = &days[days.len() / 2];
    (chandra_ephemeris::jd_to_unix_seconds(middle.noon_jd()) * 1000.0).round() as i64
}

pub fn graha_month(
    engine: &Engine,
    graha: Graha,
    days: &[CivilDay],
    zone_name: &str,
    label: String,
    system: MonthSystem,
) -> Result<GrahaMonth> {
    let mut cells = Vec::with_capacity(days.len());
    let mut sources = Vec::with_capacity(days.len());

    for day in days {
        let position = engine.position(day.noon_jd(), graha)?;
        let sun = engine.position(day.noon_jd(), Graha::Surya)?;
        let separation = crate::roots::signed_delta(position.longitude, sun.longitude).abs();
        let combust =
            combustion_orb(graha, position.is_retrograde()).is_some_and(|orb| separation < orb);

        cells.push(GrahaCell {
            date: day.date,
            longitude: position.longitude,
            rashi: Rashi::from_longitude(position.longitude),
            nakshatra: Nakshatra::from_longitude(position.longitude),
            retrograde: position.is_retrograde(),
            speed: position.speed,
            combust,
        });
        sources.push(position.source);
    }

    let events = events_in_month(engine, graha, days)?;

    Ok(GrahaMonth {
        graha,
        label,
        system,
        anchor_unix_ms: anchor(days),
        time_zone: zone_name.to_string(),
        days: cells,
        events,
        source: Source::weakest(sources),
    })
}

/// Detail for one day, dispatched by subject.
pub fn day_detail(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    observer: Observer,
) -> Result<DayDetail> {
    if graha == Graha::Chandra {
        Ok(DayDetail::Moon(Box::new(moon_day(engine, day, observer)?)))
    } else {
        Ok(DayDetail::Graha(Box::new(graha_day(
            engine, graha, day, observer,
        )?)))
    }
}

/// Either kind of day detail, tagged for the front end.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DayDetail {
    Moon(Box<MoonDay>),
    Graha(Box<GrahaDay>),
}

fn elongation(engine: &Engine, jd: f64) -> Result<f64> {
    let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
    Ok((bodies[0].longitude - bodies[1].longitude).rem_euclid(360.0))
}
