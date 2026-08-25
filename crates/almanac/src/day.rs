//! Assembly of a single day's detail.

use chandra_ephemeris::{Engine, Graha, Observer, Source};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::events::{combustion_at, Combustion};
use crate::lunar::MonthSystem;
use crate::phase::{self, PhaseName};
use crate::spans::{divisions_in_day, Division, Span};
use crate::time::{self, CivilDay, DateKey, Moment};
use crate::tithi::{self, Reference, TithiSpan};
use crate::zodiac::{degrees_in_rashi, pada, Nakshatra, Rashi};

/// The panchanga limbs a lunar calendar needs to explain the day it drew.
///
/// Present only in a lunar month. The grid states a tithi number there, and a
/// number nobody can check is worse than no number: `tithis` names it in words
/// with its true boundaries, and `sunrise` is the instant it was taken at, so
/// the reading can be audited rather than trusted.
///
/// Yoga and karana are still absent (`docs/DECISIONS.md` D-010). Neither appears
/// in the grid, so neither has a number to explain; a karana is in any case half
/// a tithi and derivable from the row above it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayPanchanga {
    /// Every tithi touching the civil day, in order.
    pub tithis: Vec<TithiSpan>,
    pub sunrise: Option<Moment>,
    pub reference: Reference,
    /// 0 = Ravivara. Independent of the locale's first day of week.
    pub vara: u8,
    pub vara_name: String,
}

/// The v1 moon detail, and nothing beyond it.
///
/// Combustion is here because the month grid marks it and a mark the day it
/// opens cannot explain is worse than no mark at all.
///
/// Tithi, yoga, karana and muhurta are deliberately absent rather than computed
/// and hidden: see `docs/DECISIONS.md` D-010.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonDay {
    pub date: DateKey,
    pub phase: PhaseName,
    /// Illuminated fraction at local noon, 0.0 to 1.0.
    pub illumination: f64,
    pub is_waxing: bool,
    pub moonrise: Option<Moment>,
    pub moonset: Option<Moment>,
    /// How far from the Sun, and whether that puts the Moon inside its rays.
    /// Judged at local noon, the same instant the month grid marks.
    pub combustion: Combustion,
    /// Present only in a lunar month.
    pub panchanga: Option<DayPanchanga>,
    pub nakshatras: Vec<NakshatraSpan>,
    pub rashis: Vec<RashiSpan>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NakshatraSpan {
    pub nakshatra: Nakshatra,
    pub name: String,
    pub lord: Graha,
    /// Pada at the day's reference instant, 1 to 4.
    pub pada: u8,
    pub entry: Moment,
    pub exit: Moment,
    pub prevailing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RashiSpan {
    pub rashi: Rashi,
    pub name: String,
    pub western: String,
    pub entry: Moment,
    pub exit: Moment,
    pub prevailing: bool,
}

/// Detail for one graha other than the Moon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahaDay {
    pub date: DateKey,
    pub graha: Graha,
    /// Sidereal longitude at the reference instant, in degrees.
    pub longitude: f64,
    /// The same longitude as degrees, arcminutes and arcseconds within its rashi.
    pub degrees_in_rashi: (u32, u32, f64),
    /// Degrees per day. Negative while retrograde.
    pub speed: f64,
    pub retrograde: bool,
    pub rise: Option<Moment>,
    pub set: Option<Moment>,
    /// How far from the Sun, and whether that puts the graha inside its rays.
    /// Judged at local noon, the same instant the month grid marks.
    pub combustion: Combustion,
    /// The same lunar day the Moon's view shows. A day is named the same
    /// whichever subject is being read on it.
    pub panchanga: Option<DayPanchanga>,
    pub nakshatras: Vec<NakshatraSpan>,
    pub rashis: Vec<RashiSpan>,
    pub source: Source,
}

/// Instant a day's divisions are attributed to.
///
/// Panchanga names a day after the nakshatra prevailing at sunrise, so sunrise
/// is used where the Sun rises. Inside a polar day or night there is no sunrise,
/// and local noon is used instead: it keeps the reference inside the day and
/// keeps the calendar usable at latitudes where the traditional rule has nothing
/// to point at.
///
/// The one place the question is asked. A month cell, a day view and a lunar
/// month's first day all need it, and three copies of the rule is three chances
/// for a cell and the day it opens to be read at different instants.
pub(crate) fn reference_instant(
    engine: &Engine,
    day: &CivilDay,
    observer: Observer,
) -> Result<f64> {
    Ok(sunrise_of(engine, day, observer)?.unwrap_or_else(|| day.noon_jd()))
}

/// Sunrise inside this civil day, if the Sun rises at all.
pub(crate) fn sunrise_of(
    engine: &Engine,
    day: &CivilDay,
    observer: Observer,
) -> Result<Option<f64>> {
    Ok(engine
        .rise_set(day.start_jd, day.end_jd, Graha::Surya, observer)?
        .rise)
}

/// The day's panchanga, in a lunar month.
///
/// The spans are counted against the sunrises of this day and its neighbours,
/// because a tithi routinely begins the day before and ends the day after, and
/// whether it holds none, one or two of them is what makes it a kshaya, an
/// ordinary tithi, or a vriddhi.
fn panchanga(
    engine: &Engine,
    day: &CivilDay,
    observer: Observer,
    system: MonthSystem,
) -> Result<Option<DayPanchanga>> {
    if !system.is_lunar() {
        return Ok(None);
    }

    let sunrise = sunrise_of(engine, day, observer)?;
    let reference = sunrise.unwrap_or_else(|| day.noon_jd());

    let neighbours = [day.previous()?, day.clone(), day.next()?];
    let mut sunrises = Vec::with_capacity(3);
    for neighbour in &neighbours {
        if let Some(jd) = sunrise_of(engine, neighbour, observer)? {
            sunrises.push(jd);
        }
    }

    let vara = time::vara(day.date)?;
    Ok(Some(DayPanchanga {
        tithis: tithi::spans_in_day(engine, day, reference, &sunrises)?,
        sunrise: sunrise.map(|jd| day.moment(jd)).transpose()?,
        reference: if sunrise.is_some() {
            Reference::Sunrise
        } else {
            Reference::LocalNoon
        },
        vara,
        vara_name: time::VARA_NAMES[vara as usize].to_string(),
    }))
}

pub fn moon_day(
    engine: &Engine,
    day: &CivilDay,
    observer: Observer,
    system: MonthSystem,
) -> Result<MoonDay> {
    let reference = reference_instant(engine, day, observer)?;

    let noon = engine.illumination(day.noon_jd())?;
    let elongation_start = elongation(engine, day.start_jd)?;
    let elongation_end = elongation(engine, day.end_jd)?;

    // A principal phase belongs to the day containing its exact instant, so the
    // day it is named on is the day it actually happens.
    let phase_name = phase::principal_phase_in(elongation_start, elongation_end)
        .unwrap_or_else(|| phase::intermediate_phase(elongation_start));

    let rise_set = engine.rise_set(day.start_jd, day.end_jd, Graha::Chandra, observer)?;
    let (nakshatras, nakshatra_source) = nakshatra_spans(engine, Graha::Chandra, day, reference)?;
    let (rashis, rashi_source) = rashi_spans(engine, Graha::Chandra, day, reference)?;
    let panchanga = panchanga(engine, day, observer, system)?;

    Ok(MoonDay {
        date: day.date,
        phase: phase_name,
        illumination: noon.fraction,
        is_waxing: phase::is_waxing(elongation_start),
        moonrise: rise_set.rise.map(|jd| day.moment(jd)).transpose()?,
        moonset: rise_set.set.map(|jd| day.moment(jd)).transpose()?,
        combustion: combustion_at(engine, Graha::Chandra, day.noon_jd())?,
        source: day_source(
            [noon.source, rise_set.source, nakshatra_source, rashi_source],
            panchanga.as_ref(),
        ),
        panchanga,
        nakshatras,
        rashis,
    })
}

/// The weakest ephemeris behind everything a day is assembled from.
///
/// Every part, not a count of them: chaining a constant `Swieph` once per
/// nakshatra read as accounting for span provenance while ignoring it entirely,
/// so a day whose spans were resolved from Moshier positions reported itself as
/// exact.
fn day_source(parts: [Source; 4], panchanga: Option<&DayPanchanga>) -> Source {
    Source::weakest(
        parts.into_iter().chain(
            panchanga
                .into_iter()
                .flat_map(|p| p.tithis.iter().map(|span| span.source)),
        ),
    )
}

pub fn graha_day(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    observer: Observer,
    system: MonthSystem,
) -> Result<GrahaDay> {
    let reference = reference_instant(engine, day, observer)?;
    let position = engine.position(reference, graha)?;
    let rise_set = engine.rise_set(day.start_jd, day.end_jd, graha, observer)?;
    let (nakshatras, nakshatra_source) = nakshatra_spans(engine, graha, day, reference)?;
    let (rashis, rashi_source) = rashi_spans(engine, graha, day, reference)?;
    let panchanga = panchanga(engine, day, observer, system)?;

    Ok(GrahaDay {
        date: day.date,
        graha,
        longitude: position.longitude,
        degrees_in_rashi: degrees_in_rashi(position.longitude),
        speed: position.speed,
        retrograde: position.is_retrograde(),
        rise: rise_set.rise.map(|jd| day.moment(jd)).transpose()?,
        set: rise_set.set.map(|jd| day.moment(jd)).transpose()?,
        combustion: combustion_at(engine, graha, day.noon_jd())?,
        source: day_source(
            [
                position.source,
                rise_set.source,
                nakshatra_source,
                rashi_source,
            ],
            panchanga.as_ref(),
        ),
        panchanga,
        nakshatras,
        rashis,
    })
}

fn elongation(engine: &Engine, jd: f64) -> Result<f64> {
    let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
    Ok((bodies[0].longitude - bodies[1].longitude).rem_euclid(360.0))
}

/// The nakshatras the graha occupies, and the weakest ephemeris behind them.
///
/// The provenance is returned rather than put on each span: nothing shows it per
/// span, and a payload field nothing reads is weight. It must not be dropped
/// either - a day assembled from Moshier positions is not an exact day - so it
/// is folded into the day's own source.
fn nakshatra_spans(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    reference: f64,
) -> Result<(Vec<NakshatraSpan>, Source)> {
    let spans = divisions_in_day(engine, graha, day, Division::Nakshatra, reference)?;
    let reference_longitude = engine.position(reference, graha)?.longitude;
    let source = Source::weakest(spans.iter().map(|span| span.source));

    let named = spans
        .into_iter()
        .map(|span: Span| {
            let nakshatra = Nakshatra::ALL[span.index];
            Ok(NakshatraSpan {
                nakshatra,
                name: nakshatra.name().to_string(),
                lord: nakshatra.lord(),
                // Pada is only meaningful for the division actually occupied at
                // the reference instant; for the others the entry longitude is
                // the boundary itself, so pada 1 is correct by construction.
                pada: if span.prevailing {
                    pada(reference_longitude)
                } else {
                    1
                },
                entry: span.entry,
                exit: span.exit,
                prevailing: span.prevailing,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((named, source))
}

/// The rashis the graha occupies, and the weakest ephemeris behind them.
fn rashi_spans(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    reference: f64,
) -> Result<(Vec<RashiSpan>, Source)> {
    let spans = divisions_in_day(engine, graha, day, Division::Rashi, reference)?;
    let source = Source::weakest(spans.iter().map(|span| span.source));

    let named = spans
        .into_iter()
        .map(|span| {
            let rashi = Rashi::ALL[span.index];
            RashiSpan {
                rashi,
                name: rashi.name().to_string(),
                western: rashi.western().to_string(),
                entry: span.entry,
                exit: span.exit,
                prevailing: span.prevailing,
            }
        })
        .collect();
    Ok((named, source))
}
