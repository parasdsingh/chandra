//! Assembly of a month view: one cell per day, plus events for graha months.
//!
//! Month views carry only what the grid draws. Everything else - entry and exit
//! times, rise and set, degrees - is computed when a day is actually opened, so
//! opening a month never pays for detail nobody looked at.

use chandra_ephemeris::{Engine, Graha, Observer, Source};
use serde::{Deserialize, Serialize};

use crate::day::{graha_day, moon_day, GrahaDay, MoonDay};
use crate::error::Result;
use crate::events::{combustion_at, combustion_from, events_in_month, Event};
use crate::lunar::MonthSystem;
use crate::phase::{self, PhaseName};
use crate::time::{days_in_month, CivilDay, DateKey};
use crate::tithi::{self, CellTithi, SkippedTithi, Tithi, Vriddhi};
use crate::zodiac::{Nakshatra, Rashi};

/// A month of moon phases.
///
/// The month may be a Gregorian month or a lunar one; the difference is carried
/// entirely by `label`, `system` and which civil days appear in `days`. The grid
/// that draws it does not need to know which it is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonMonth {
    /// What the header shows: `August 2026`, `Shravana 2083`, `Adhika Shravana
    /// 2080`.
    pub label: String,
    /// The month's own name, without the `Adhika` prefix and without the year,
    /// so the header can set the qualifier apart from the name it qualifies.
    pub name: String,
    /// Intercalary: the Sun crossed no rashi boundary inside the month.
    pub adhika: bool,
    /// The second name of a kshaya masa, where the Sun crossed two.
    pub kshaya_masa_name: Option<String>,
    /// Vikram Samvat year. `None` in solar mode, which counts Gregorian years.
    pub era_year: Option<i16>,
    pub system: MonthSystem,
    /// An instant inside this month, used to navigate to its neighbours. A lunar
    /// month has no year-and-number to step through, so the cursor is a time.
    pub anchor_unix_ms: i64,
    /// IANA zone name. The front end formats every timestamp against this rather
    /// than against the machine's own zone.
    pub time_zone: String,
    /// The 42 cells the grid draws, in reading order, each flagged as inside the
    /// month or not. Laid out here rather than in the front end: only this layer
    /// knows which civil days a lunar month contains, and a front end that
    /// guesses the neighbouring dates draws cells it has no data for.
    pub days: Vec<MoonCell>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonCell {
    /// Full civil date: a lunar month crosses Gregorian month boundaries, so a
    /// day-of-month alone would be ambiguous.
    pub date: DateKey,
    /// False for the leading and trailing cells, which belong to the
    /// neighbouring months and are drawn dimmed.
    pub in_month: bool,
    /// The lunar day. Present only in a lunar month: solar mode does not name
    /// tithis (`docs/DECISIONS.md` D-010, D-021).
    pub tithi: Option<CellTithi>,
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
    pub name: String,
    pub adhika: bool,
    pub kshaya_masa_name: Option<String>,
    pub era_year: Option<i16>,
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
    pub in_month: bool,
    /// The same lunar day the Moon's calendar shows. A day is named the same
    /// whichever subject is being plotted on it.
    pub tithi: Option<CellTithi>,
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

/// What a month is called, and how it is qualified.
///
/// Passed as one value because these six travel together from the resolver to
/// both builders, and a builder that took them apart could put a lunar name on a
/// solar month or a Gregorian year on a lunar one.
#[derive(Debug, Clone)]
pub struct MonthLabel {
    pub label: String,
    pub name: String,
    pub adhika: bool,
    pub kshaya_masa_name: Option<String>,
    pub era_year: Option<i16>,
    pub system: MonthSystem,
}

/// The lunar day for each of the 42 grid cells.
///
/// Computed once per month and shared by every subject, because a tithi is a
/// property of the day rather than of what is being plotted on it.
///
/// Kshaya and vriddhi are read from the tithi at each day's sunrise against its
/// neighbours', which is why the day before the grid and the day after it are
/// resolved too: without them the first and last cells could not tell a repeated
/// tithi from an ordinary one.
pub fn tithi_frames(
    engine: &Engine,
    grid: &[(CivilDay, bool)],
    observer: Observer,
    zone: &jiff::tz::TimeZone,
) -> Result<Vec<CellTithi>> {
    let before = grid[0].0.date_of(grid[0].0.start_jd - 0.5)?;
    let last = &grid[grid.len() - 1].0;
    let after = last.date_of(last.end_jd + 0.5)?;

    let mut extended = Vec::with_capacity(grid.len() + 2);
    extended.push(CivilDay::new(before, zone)?);
    extended.extend(grid.iter().map(|(day, _)| day.clone()));
    extended.push(CivilDay::new(after, zone)?);

    let anchors = extended
        .iter()
        .map(|day| sunrise_anchor(engine, day, observer))
        .collect::<Result<Vec<_>>>()?;

    let mut frames = Vec::with_capacity(grid.len());
    for position in 1..=grid.len() {
        let (day, _) = &grid[position - 1];
        let anchor = &anchors[position];
        let previous = &anchors[position - 1];
        let next = &anchors[position + 1];

        // A tithi holding two sunrises names two days; the pair is identified
        // from the outside, by which neighbour shares the number.
        let vriddhi = if anchor.tithi == next.tithi {
            Some(Vriddhi::First)
        } else if anchor.tithi == previous.tithi {
            Some(Vriddhi::Second)
        } else {
            None
        };

        // Whatever lies strictly between this sunrise's tithi and the next one's
        // began and ended in between, so no day is named after it.
        let mut kshaya = Vec::new();
        let mut skipped = anchor.tithi.next();
        for _ in 1..anchor.tithi.steps_to(next.tithi) {
            kshaya.push(SkippedTithi::new(skipped));
            skipped = skipped.next();
        }

        frames.push(CellTithi {
            index: anchor.tithi.index(),
            number: anchor.tithi.number(),
            paksha: anchor.tithi.paksha(),
            name: anchor.tithi.name().to_string(),
            reference: anchor.reference,
            sunrise: anchor.sunrise.map(|jd| day.moment(jd)).transpose()?,
            kshaya,
            vriddhi,
        });
    }
    Ok(frames)
}

/// A day's reference instant and the tithi in force at it.
struct Anchor {
    tithi: Tithi,
    reference: crate::tithi::Reference,
    sunrise: Option<f64>,
}

/// Sunrise, or local noon where the Sun does not rise.
///
/// Reads it through `day::sunrise_of`, the one place the rule lives, so a cell
/// and the day it opens can never name different tithis.
fn sunrise_anchor(engine: &Engine, day: &CivilDay, observer: Observer) -> Result<Anchor> {
    let sunrise = crate::day::sunrise_of(engine, day, observer)?;

    let instant = sunrise.unwrap_or_else(|| day.noon_jd());
    Ok(Anchor {
        tithi: tithi::at(engine, instant)?,
        reference: if sunrise.is_some() {
            crate::tithi::Reference::Sunrise
        } else {
            crate::tithi::Reference::LocalNoon
        },
        sunrise,
    })
}

pub fn moon_month(
    engine: &Engine,
    grid: &[(CivilDay, bool)],
    frames: Option<&[CellTithi]>,
    zone_name: &str,
    naming: MonthLabel,
) -> Result<MoonMonth> {
    let mut cells = Vec::with_capacity(grid.len());
    let mut sources = Vec::with_capacity(grid.len());

    // Elongation at each day boundary. Consecutive days share a boundary, so
    // carrying the previous value forward halves the ephemeris calls.
    let mut elongation_start = elongation(engine, grid[0].0.start_jd)?;

    for (position, (day, in_month)) in grid.iter().enumerate() {
        let elongation_end = elongation(engine, day.end_jd)?;
        let illumination = engine.illumination(day.noon_jd())?;
        let principal = phase::principal_phase_in(elongation_start, elongation_end);

        // Combustion is judged at local noon, the day's midpoint, for every
        // subject alike. The day view asks at the same instant, so the mark on
        // a cell and the reading inside it always agree.
        let combustion = combustion_at(engine, Graha::Chandra, day.noon_jd())?;

        cells.push(MoonCell {
            date: day.date,
            in_month: *in_month,
            tithi: frames.map(|frames| frames[position].clone()),
            illumination: illumination.fraction,
            is_waxing: phase::is_waxing(elongation_start),
            phase: principal
                .map(|(name, _)| name)
                .unwrap_or_else(|| phase::intermediate_phase(elongation_start)),
            principal: principal.is_some(),
            combust: combustion.combust,
        });
        sources.push(illumination.source);
        elongation_start = elongation_end;
    }

    Ok(MoonMonth {
        label: naming.label,
        name: naming.name,
        adhika: naming.adhika,
        kshaya_masa_name: naming.kshaya_masa_name,
        era_year: naming.era_year,
        system: naming.system,
        anchor_unix_ms: anchor(grid),
        time_zone: zone_name.to_string(),
        days: cells,
        source: Source::weakest(sources),
    })
}

/// An instant safely inside the month, used as its navigation cursor.
///
/// Local noon of the middle day *of the month*, not of the grid: the grid's own
/// middle can fall in a neighbouring month, and stepping from there would walk
/// the calendar off by one. Far from both boundaries, so stepping cannot land
/// back in the same month through a rounding accident.
fn anchor(grid: &[(CivilDay, bool)]) -> i64 {
    let inside: Vec<&CivilDay> = grid
        .iter()
        .filter(|(_, in_month)| *in_month)
        .map(|(day, _)| day)
        .collect();
    let middle = inside[inside.len() / 2];
    (chandra_ephemeris::jd_to_unix_seconds(middle.noon_jd()) * 1000.0).round() as i64
}

pub fn graha_month(
    engine: &Engine,
    graha: Graha,
    grid: &[(CivilDay, bool)],
    frames: Option<&[CellTithi]>,
    zone_name: &str,
    naming: MonthLabel,
) -> Result<GrahaMonth> {
    let mut cells = Vec::with_capacity(grid.len());
    let mut sources = Vec::with_capacity(grid.len());

    for (cell, (day, in_month)) in grid.iter().enumerate() {
        let position = engine.position(day.noon_jd(), graha)?;
        let sun = engine.position(day.noon_jd(), Graha::Surya)?;
        let combustion = combustion_from(
            graha,
            position.longitude,
            sun.longitude,
            position.is_retrograde(),
        );

        cells.push(GrahaCell {
            date: day.date,
            in_month: *in_month,
            tithi: frames.map(|frames| frames[cell].clone()),
            longitude: position.longitude,
            rashi: Rashi::from_longitude(position.longitude),
            nakshatra: Nakshatra::from_longitude(position.longitude),
            retrograde: position.is_retrograde(),
            speed: position.speed,
            combust: combustion.combust,
        });
        sources.push(position.source);
    }

    // Events are found across the month itself, not the grid: a station in the
    // trailing cells belongs to the next month's list, and listing it twice
    // would put two marks on one instant.
    let inside: Vec<CivilDay> = grid
        .iter()
        .filter(|(_, in_month)| *in_month)
        .map(|(day, _)| day.clone())
        .collect();
    let events = events_in_month(engine, graha, &inside)?;

    Ok(GrahaMonth {
        graha,
        label: naming.label,
        name: naming.name,
        adhika: naming.adhika,
        kshaya_masa_name: naming.kshaya_masa_name,
        era_year: naming.era_year,
        system: naming.system,
        anchor_unix_ms: anchor(grid),
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
    system: MonthSystem,
) -> Result<DayDetail> {
    if graha == Graha::Chandra {
        Ok(DayDetail::Moon(Box::new(moon_day(
            engine, day, observer, system,
        )?)))
    } else {
        Ok(DayDetail::Graha(Box::new(graha_day(
            engine, graha, day, observer, system,
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
