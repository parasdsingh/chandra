//! Lunar months.
//!
//! A lunar month is not a Gregorian month with a different label. It runs from
//! one syzygy to the next - new moon to new moon in the amanta reckoning, full
//! moon to full moon in the purnimanta - and takes its name from where the Sun
//! stands at that moment, not from the calendar.

use chandra_ephemeris::{Engine, Graha, Observer, Source};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::roots::{self, Bracket};
use crate::time::{CivilDay, DateKey};
use crate::zodiac::{Rashi, RASHI_ARC};

/// Which month a calendar navigates by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonthSystem {
    /// Gregorian months.
    #[default]
    Solar,
    /// New moon to new moon. Used across western, southern and central India.
    Amanta,
    /// Full moon to full moon. Used across northern India.
    Purnimanta,
}

impl MonthSystem {
    pub const ALL: [MonthSystem; 3] = [
        MonthSystem::Solar,
        MonthSystem::Amanta,
        MonthSystem::Purnimanta,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            MonthSystem::Solar => "solar",
            MonthSystem::Amanta => "amanta",
            MonthSystem::Purnimanta => "purnimanta",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            MonthSystem::Solar => "Solar (Gregorian)",
            MonthSystem::Amanta => "Lunar, amanta (new moon)",
            MonthSystem::Purnimanta => "Lunar, purnimanta (full moon)",
        }
    }

    /// Elongation at which this system's month begins.
    const fn boundary_elongation(self) -> f64 {
        match self {
            // Solar never asks; the value is unused.
            MonthSystem::Solar | MonthSystem::Amanta => 0.0,
            MonthSystem::Purnimanta => 180.0,
        }
    }

    pub const fn is_lunar(self) -> bool {
        !matches!(self, MonthSystem::Solar)
    }
}

/// The twelve lunar month names, indexed by the rashi the Sun occupies at the
/// new moon that begins the month.
///
/// The Sun is in Meena at the new moon beginning Chaitra, in Mesha at the one
/// beginning Vaishakha, and so on. Verified against a known month: the new moon
/// of 12 August 2026 falls with the Sun in Karka, and the amanta month it begins
/// is Shravana.
const NAMES_BY_SOLAR_RASHI: [&str; 12] = [
    "Vaishakha",    // Sun in Mesha
    "Jyeshtha",     // Vrishabha
    "Ashadha",      // Mithuna
    "Shravana",     // Karka
    "Bhadrapada",   // Simha
    "Ashwina",      // Kanya
    "Kartika",      // Tula
    "Margashirsha", // Vrishchika
    "Pausha",       // Dhanu
    "Magha",        // Makara
    "Phalguna",     // Kumbha
    "Chaitra",      // Meena
];

/// One lunar month.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunarMonth {
    pub name: &'static str,
    /// True for an intercalary month: one containing no sankranti, because the
    /// Sun stayed in a single rashi for its whole length. It repeats the name of
    /// the month that follows it.
    pub adhika: bool,
    /// Julian Day of the syzygy that opens the month.
    pub start_jd: f64,
    /// Julian Day of the syzygy that opens the next month.
    pub end_jd: f64,
    /// First civil day: the first day whose sunrise falls after the opening
    /// syzygy, which is the traditional rule.
    pub first_day: DateKey,
    /// Last civil day, the day before the next month's first day.
    pub last_day: DateKey,
    pub source: Source,
}

impl LunarMonth {
    /// Display name, with the intercalary prefix where one applies.
    pub fn display_name(&self) -> String {
        if self.adhika {
            format!("Adhika {}", self.name)
        } else {
            self.name.to_string()
        }
    }
}

/// The Moon's elongation from the Sun in degrees, `[0, 360)`.
fn elongation(engine: &Engine, jd: f64) -> Option<f64> {
    let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya]).ok()?;
    Some((bodies[0].longitude - bodies[1].longitude).rem_euclid(360.0))
}

/// Scan step for syzygy search.
///
/// Elongation grows by 12.2 degrees a day on average and never more than about
/// 15, so a one-day step cannot pass a boundary and come back.
const SYZYGY_STEP: f64 = 1.0;

/// A synodic month is 29.53 days; searching 40 covers the longest one with room
/// to spare.
const SYZYGY_WINDOW: f64 = 40.0;

/// The first syzygy of the given kind at or after `from`.
fn next_syzygy(engine: &Engine, from: f64, target: f64) -> Result<f64> {
    let f = |jd: f64| elongation(engine, jd).map(|e| roots::signed_delta(e, target));

    for bracket in roots::brackets(from, from + SYZYGY_WINDOW, SYZYGY_STEP, f) {
        // Only crossings in the direction of travel count. The wrapped
        // difference also changes sign half a revolution away, which is not a
        // syzygy of this kind.
        if bracket.f_lo > 0.0 && bracket.f_hi < 0.0 {
            continue;
        }
        if let Some(exact) = roots::refine(bracket, f) {
            return Ok(exact);
        }
    }

    Err(Error::NoCrossing {
        what: "syzygy",
        graha: "Chandra",
        near: from,
        window_days: SYZYGY_WINDOW,
    })
}

/// The last syzygy of the given kind at or before `from`.
fn previous_syzygy(engine: &Engine, from: f64, target: f64) -> Result<f64> {
    // Step back a whole synodic month and search forward: searching backwards
    // would need a mirrored bracket routine for no benefit.
    let mut start = from - SYZYGY_WINDOW;
    let mut latest = None;

    while start < from {
        match next_syzygy(engine, start, target) {
            Ok(found) if found <= from => {
                latest = Some(found);
                start = found + 1.0;
            }
            _ => break,
        }
    }

    latest.ok_or(Error::NoCrossing {
        what: "syzygy",
        graha: "Chandra",
        near: from,
        window_days: SYZYGY_WINDOW,
    })
}

/// The first civil day of a month opening at `syzygy_jd`.
///
/// Traditionally the month begins on the first day whose sunrise falls after the
/// syzygy: a new moon at ten in the morning does not make that morning the first
/// day of the month, because at sunrise the old month was still running.
fn first_civil_day(
    engine: &Engine,
    syzygy_jd: f64,
    observer: Observer,
    zone: &jiff::tz::TimeZone,
) -> Result<DateKey> {
    let mut date = CivilDay::new(DateKey::new(1, 1, 1)?, zone)?.date_of(syzygy_jd)?;

    // At most two steps: sunrise is within a day of any instant.
    for _ in 0..3 {
        let day = CivilDay::new(date, zone)?;
        let sunrise = engine
            .rise_set(day.start_jd, Graha::Surya, observer)?
            .rise
            .unwrap_or_else(|| day.noon_jd());

        if sunrise >= syzygy_jd {
            return Ok(date);
        }
        date = day.date_of(day.end_jd + 0.5)?;
    }

    Ok(date)
}

/// The lunar month containing `jd`.
pub fn month_containing(
    engine: &Engine,
    jd: f64,
    system: MonthSystem,
    observer: Observer,
    zone: &jiff::tz::TimeZone,
) -> Result<LunarMonth> {
    let target = system.boundary_elongation();
    let start_jd = previous_syzygy(engine, jd, target)?;
    build(engine, start_jd, system, observer, zone)
}

/// The lunar month `offset` months away from the one containing `jd`.
pub fn shift(
    engine: &Engine,
    month: &LunarMonth,
    offset: i32,
    system: MonthSystem,
    observer: Observer,
    zone: &jiff::tz::TimeZone,
) -> Result<LunarMonth> {
    let target = system.boundary_elongation();
    let mut start = month.start_jd;

    for _ in 0..offset.abs() {
        start = if offset > 0 {
            next_syzygy(engine, start + 1.0, target)?
        } else {
            previous_syzygy(engine, start - 1.0, target)?
        };
    }

    build(engine, start, system, observer, zone)
}

fn build(
    engine: &Engine,
    start_jd: f64,
    system: MonthSystem,
    observer: Observer,
    zone: &jiff::tz::TimeZone,
) -> Result<LunarMonth> {
    let target = system.boundary_elongation();
    let end_jd = next_syzygy(engine, start_jd + 1.0, target)?;

    // Naming always follows the new moon, in both reckonings. For a purnimanta
    // month that is the new moon falling inside it, which begins the amanta
    // month of the same name.
    let naming_jd = match system {
        MonthSystem::Purnimanta => next_syzygy(engine, start_jd + 1.0, 0.0)?,
        _ => start_jd,
    };

    let sun_at_start = engine.position(naming_jd, Graha::Surya)?;
    let rashi_at_start = Rashi::from_longitude(sun_at_start.longitude).index();

    // The month that follows the naming new moon, to detect an intercalary
    // month: one in which the Sun never leaves its rashi, so no sankranti falls
    // inside it.
    let next_new_moon = next_syzygy(engine, naming_jd + 1.0, 0.0)?;
    let sun_at_end = engine.position(next_new_moon, Graha::Surya)?;
    let rashi_at_end = Rashi::from_longitude(sun_at_end.longitude).index();
    let adhika = rashi_at_start == rashi_at_end;

    // The name always comes from the rashi the Sun occupies at the new moon.
    // An intercalary month needs no adjustment: because the Sun does not leave
    // that rashi, the regular month following it derives the same name, which is
    // exactly why the pair reads "Adhika Shravana" then "Shravana". Deriving the
    // intercalary name from the next rashi instead names it after the month
    // after the one it doubles.
    let name_index = rashi_at_start;

    let first_day = first_civil_day(engine, start_jd, observer, zone)?;
    let next_first = first_civil_day(engine, end_jd, observer, zone)?;
    let last_day = CivilDay::new(next_first, zone)?
        .date_of(CivilDay::new(next_first, zone)?.start_jd - 0.5)?;

    Ok(LunarMonth {
        name: NAMES_BY_SOLAR_RASHI[name_index],
        adhika,
        start_jd,
        end_jd,
        first_day,
        last_day,
        source: Source::weakest([sun_at_start.source, sun_at_end.source]),
    })
}

/// Sankrantis - the Sun entering a rashi - between two instants.
///
/// Exposed because the intercalary rule is stated in terms of sankrantis, and a
/// caller checking the rule directly should not have to re-derive them.
pub fn sankrantis(engine: &Engine, from: f64, to: f64) -> Result<Vec<(f64, Rashi)>> {
    let mut found = Vec::new();
    let mut jd = from;

    while jd < to {
        let next = (jd + 1.0).min(to);
        let (a, b) = (
            engine.position(jd, Graha::Surya)?.longitude,
            engine.position(next, Graha::Surya)?.longitude,
        );

        let index_a = (a / RASHI_ARC) as usize;
        let index_b = (b / RASHI_ARC) as usize;

        if index_a != index_b {
            let boundary = index_b as f64 * RASHI_ARC;
            let bracket = Bracket {
                lo: jd,
                hi: next,
                f_lo: roots::signed_delta(a, boundary),
                f_hi: roots::signed_delta(b, boundary),
            };
            if let Some(exact) = roots::refine(bracket, |t| {
                engine
                    .position(t, Graha::Surya)
                    .ok()
                    .map(|p| roots::signed_delta(p.longitude, boundary))
            }) {
                found.push((exact, Rashi::ALL[index_b]));
            }
        }
        jd = next;
    }
    Ok(found)
}
