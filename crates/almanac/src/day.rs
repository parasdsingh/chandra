//! Assembly of a single day's detail.

use chandra_ephemeris::{Engine, Graha, Observer, Source};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::phase::{self, PhaseName};
use crate::spans::{divisions_in_day, Division, Span};
use crate::time::{CivilDay, DateKey, Moment};
use crate::zodiac::{degrees_in_rashi, pada, Nakshatra, Rashi};

/// The six fields of the v1 moon detail, and nothing beyond them.
///
/// Tithi, yoga, karana and muhurta are deliberately absent rather than computed
/// and hidden: see `docs/DECISIONS.md` D-010.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonDay {
    pub date: DateKey,
    pub phase: PhaseName,
    /// Present only when a principal phase occurs during this day, in which case
    /// it is the exact instant of that phase.
    pub principal_at: Option<Moment>,
    /// Illuminated fraction at local noon, 0.0 to 1.0.
    pub illumination: f64,
    pub is_waxing: bool,
    pub moonrise: Option<Moment>,
    pub moonset: Option<Moment>,
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
fn reference_instant(engine: &Engine, day: &CivilDay, observer: Observer) -> Result<f64> {
    let sunrise = engine
        .rise_set(day.start_jd, Graha::Surya, observer)?
        .rise
        .filter(|&jd| jd >= day.start_jd && jd < day.end_jd);
    Ok(sunrise.unwrap_or_else(|| day.noon_jd()))
}

pub fn moon_day(engine: &Engine, day: &CivilDay, observer: Observer) -> Result<MoonDay> {
    let reference = reference_instant(engine, day, observer)?;

    let noon = engine.illumination(day.noon_jd())?;
    let elongation_start = elongation(engine, day.start_jd)?;
    let elongation_end = elongation(engine, day.end_jd)?;

    // A principal phase belongs to the day containing its exact instant, so the
    // day it is named on is the day it actually happens.
    let principal = phase::principal_phase_in(elongation_start, elongation_end);
    let (phase_name, principal_at) = match principal {
        Some((name, fraction)) => {
            let jd = day.start_jd + fraction * day.length();
            (name, Some(day.moment(jd)?))
        }
        None => (phase::intermediate_phase(elongation_start), None),
    };

    let rise_set = engine.rise_set(day.start_jd, Graha::Chandra, observer)?;
    let nakshatras = nakshatra_spans(engine, Graha::Chandra, day, reference)?;
    let rashis = rashi_spans(engine, Graha::Chandra, day, reference)?;

    let source = Source::weakest(
        [noon.source]
            .into_iter()
            .chain(nakshatras.iter().map(|_| Source::Swieph)),
    );

    Ok(MoonDay {
        date: day.date,
        phase: phase_name,
        principal_at,
        illumination: noon.fraction,
        is_waxing: phase::is_waxing(elongation_start),
        moonrise: rise_set.rise.map(|jd| day.moment(jd)).transpose()?,
        moonset: rise_set.set.map(|jd| day.moment(jd)).transpose()?,
        nakshatras,
        rashis,
        source,
    })
}

pub fn graha_day(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    observer: Observer,
) -> Result<GrahaDay> {
    let reference = reference_instant(engine, day, observer)?;
    let position = engine.position(reference, graha)?;
    let rise_set = engine.rise_set(day.start_jd, graha, observer)?;

    Ok(GrahaDay {
        date: day.date,
        graha,
        longitude: position.longitude,
        degrees_in_rashi: degrees_in_rashi(position.longitude),
        speed: position.speed,
        retrograde: position.is_retrograde(),
        rise: rise_set.rise.map(|jd| day.moment(jd)).transpose()?,
        set: rise_set.set.map(|jd| day.moment(jd)).transpose()?,
        nakshatras: nakshatra_spans(engine, graha, day, reference)?,
        rashis: rashi_spans(engine, graha, day, reference)?,
        source: position.source,
    })
}

fn elongation(engine: &Engine, jd: f64) -> Result<f64> {
    let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
    Ok((bodies[0].longitude - bodies[1].longitude).rem_euclid(360.0))
}

fn nakshatra_spans(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    reference: f64,
) -> Result<Vec<NakshatraSpan>> {
    let spans = divisions_in_day(engine, graha, day, Division::Nakshatra, reference)?;
    let reference_longitude = engine.position(reference, graha)?.longitude;

    spans
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
        .collect()
}

fn rashi_spans(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    reference: f64,
) -> Result<Vec<RashiSpan>> {
    let spans = divisions_in_day(engine, graha, day, Division::Rashi, reference)?;
    Ok(spans
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
        .collect())
}
