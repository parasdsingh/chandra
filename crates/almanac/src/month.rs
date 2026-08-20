//! Assembly of a month view: one cell per day, plus events for graha months.
//!
//! Month views carry only what the grid draws. Everything else - entry and exit
//! times, rise and set, degrees - is computed when a day is actually opened, so
//! opening a month never pays for detail nobody looked at.

use chandra_ephemeris::{Engine, Graha, Observer, Source};
use serde::{Deserialize, Serialize};

use crate::day::{graha_day, moon_day, GrahaDay, MoonDay};
use crate::error::Result;
use crate::events::{events_in_month, Event};
use crate::phase::{self, PhaseName};
use crate::time::{days_in_month, first_weekday_offset, CivilDay, DateKey};
use crate::zodiac::{Nakshatra, Rashi};

/// A month of moon phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonMonth {
    pub year: i16,
    pub month: i8,
    /// IANA zone name. The front end formats every timestamp against this rather
    /// than against the machine's own zone.
    pub time_zone: String,
    /// Zero-based offset of the first day from Monday, for grid placement.
    pub leading_blanks: u8,
    pub days: Vec<MoonCell>,
    pub source: Source,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MoonCell {
    pub day: i8,
    /// Illuminated fraction at local noon, 0.0 to 1.0.
    pub illumination: f64,
    pub is_waxing: bool,
    pub phase: PhaseName,
    /// True when a principal phase falls on this day. The grid marks these.
    pub principal: bool,
}

/// A month of one graha's transit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahaMonth {
    pub graha: Graha,
    pub year: i16,
    pub month: i8,
    pub time_zone: String,
    pub leading_blanks: u8,
    pub days: Vec<GrahaCell>,
    pub events: Vec<Event>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahaCell {
    pub day: i8,
    pub longitude: f64,
    pub rashi: Rashi,
    pub nakshatra: Nakshatra,
    pub retrograde: bool,
    pub speed: f64,
}

/// The civil days of a month in the observer's zone.
pub fn month_days(year: i16, month: i8, zone: &jiff::tz::TimeZone) -> Result<Vec<CivilDay>> {
    let count = days_in_month(year, month)?;
    (1..=count as i8)
        .map(|day| CivilDay::new(DateKey::new(year, month, day)?, zone))
        .collect()
}

pub fn moon_month(engine: &Engine, days: &[CivilDay], zone_name: &str) -> Result<MoonMonth> {
    let mut cells = Vec::with_capacity(days.len());
    let mut sources = Vec::with_capacity(days.len());

    // Elongation at each day boundary. Consecutive days share a boundary, so
    // carrying the previous value forward halves the ephemeris calls.
    let mut elongation_start = elongation(engine, days[0].start_jd)?;

    for day in days {
        let elongation_end = elongation(engine, day.end_jd)?;
        let illumination = engine.illumination(day.noon_jd())?;
        let principal = phase::principal_phase_in(elongation_start, elongation_end);

        cells.push(MoonCell {
            day: day.date.day,
            illumination: illumination.fraction,
            is_waxing: phase::is_waxing(elongation_start),
            phase: principal
                .map(|(name, _)| name)
                .unwrap_or_else(|| phase::intermediate_phase(elongation_start)),
            principal: principal.is_some(),
        });
        sources.push(illumination.source);
        elongation_start = elongation_end;
    }

    let first = days[0].date;
    Ok(MoonMonth {
        year: first.year,
        month: first.month,
        time_zone: zone_name.to_string(),
        leading_blanks: first_weekday_offset(first.year, first.month)?,
        days: cells,
        source: Source::weakest(sources),
    })
}

pub fn graha_month(
    engine: &Engine,
    graha: Graha,
    days: &[CivilDay],
    zone_name: &str,
) -> Result<GrahaMonth> {
    let mut cells = Vec::with_capacity(days.len());
    let mut sources = Vec::with_capacity(days.len());

    for day in days {
        let position = engine.position(day.noon_jd(), graha)?;
        cells.push(GrahaCell {
            day: day.date.day,
            longitude: position.longitude,
            rashi: Rashi::from_longitude(position.longitude),
            nakshatra: Nakshatra::from_longitude(position.longitude),
            retrograde: position.is_retrograde(),
            speed: position.speed,
        });
        sources.push(position.source);
    }

    let events = events_in_month(engine, graha, days)?;
    let first = days[0].date;

    Ok(GrahaMonth {
        graha,
        year: first.year,
        month: first.month,
        time_zone: zone_name.to_string(),
        leading_blanks: first_weekday_offset(first.year, first.month)?,
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
