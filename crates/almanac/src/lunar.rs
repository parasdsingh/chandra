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
use crate::time::{date_at, CivilDay, DateKey};
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
    /// Sankrantis inside the month: `0` intercalary, `1` ordinary, `2` a kshaya
    /// masa, where the Sun crosses two rashis and a month name is skipped
    /// entirely. Carried as a count rather than as two flags so the three states
    /// cannot contradict each other.
    pub sankranti_count: u8,
    /// The second name when `sankranti_count == 2`. `None` otherwise.
    pub kshaya_masa_name: Option<&'static str>,
    /// Vikram Samvat year. The era begins at Chaitra, so it does not line up
    /// with the Gregorian year the month happens to fall in.
    pub vikram_year: i16,
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
    ///
    /// A kshaya masa carries both names joined by an en dash: the Sun crossed
    /// two rashis inside one lunar month, so one name has no month of its own
    /// and is not simply dropped.
    pub fn display_name(&self) -> String {
        match (self.adhika, self.kshaya_masa_name) {
            (true, _) => format!("Adhika {}", self.name),
            (false, Some(second)) => format!("{}\u{2013}{}", self.name, second),
            (false, None) => self.name.to_string(),
        }
    }
}

/// Vikram Samvat year for a month.
///
/// The era begins at Chaitra, in March or April, so it runs 57 ahead of the
/// Gregorian year for most of its length and 56 ahead for the part that falls
/// after 1 January. Deciding by the Gregorian year alone would be wrong for a
/// quarter of every year; deciding by the month name alone would be wrong for
/// Pausha, which starts in December in some years and in January in others.
fn vikram_year(name_index: usize, first_day: DateKey) -> i16 {
    // Chaitra opens the era; Pausha, Magha and Phalguna close it.
    let position_in_era = (name_index + 1) % 12;
    let closing_months = position_in_era >= 9;
    let after_new_year = closing_months && first_day.month <= 3;

    first_day.year + if after_new_year { 56 } else { 57 }
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

/// Mean synodic month, in days.
///
/// The long-run average interval between syzygies of the same kind. Used only to
/// *estimate* where a distant month lies, never to report a time: every boundary
/// this crate hands out is a refined root of the true elongation.
///
/// What makes the estimate safe is that the true syzygy does not drift away from
/// the mean one. Writing the kth syzygy after a base as `base + k * MEAN + d(k)`,
/// the deviation `d` is periodic rather than cumulative - the mean is the average
/// by construction - and stays inside about a day and a half. Half a synodic
/// month is 14.8 days, so `round((found - base) / MEAN)` recovers `k` exactly
/// with an order of magnitude to spare. `the_mean_month_never_drifts_far_enough_to_miscount`
/// asserts that margin over two centuries rather than trusting this paragraph.
const MEAN_SYNODIC: f64 = 29.530_588_853;

/// How far the seek in [`shift`] may be wrong before it is a bug rather than a
/// correction. Two steps is already past the measured worst case of one.
const SEEK_SLACK: i32 = 3;

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
    // Step back `SYZYGY_WINDOW` - 40 days, which is a synodic month with room
    // to spare - and search forward: searching backwards would need a mirrored
    // bracket routine for no benefit.
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
    let mut date = date_at(syzygy_jd, zone)?;

    // At most two steps: sunrise is within a day of any instant.
    for _ in 0..3 {
        let day = CivilDay::new(date, zone)?;
        // The same instant the cell and the day view read the day at. Taking
        // the raw rise instead admitted the *next* day's sunrise on a day the
        // Sun does not rise, and started the month a day early.
        let sunrise = crate::day::reference_instant(engine, &day, observer)?;

        if sunrise >= syzygy_jd {
            return Ok(date);
        }
        date = day.date_of(day.end_jd + 0.5)?;
    }

    Ok(date)
}

/// The lunar month containing `jd`.
///
/// Selected by the civil days the month holds, not by the syzygy that opens it.
/// The two disagree by up to a day - the month begins at the first sunrise after
/// the syzygy - so an instant between them belongs to the month that is still
/// running. Choosing by the syzygy alone opened a grid in which today was a
/// dimmed out-of-month cell, and on 31 May 2026 amanta today was not among the
/// 42 cells at all.
pub fn month_containing(
    engine: &Engine,
    jd: f64,
    system: MonthSystem,
    observer: Observer,
    zone: &jiff::tz::TimeZone,
) -> Result<LunarMonth> {
    let target = system.boundary_elongation();
    let month = build(
        engine,
        previous_syzygy(engine, jd, target)?,
        system,
        observer,
        zone,
    )?;

    // At most one month out, because the two rules never differ by more than a
    // single civil day.
    let date = date_at(jd, zone)?;
    let step = if date < month.first_day {
        -1
    } else if date > month.last_day {
        1
    } else {
        return Ok(month);
    };
    shift(engine, &month, step, system, observer, zone)
}

/// The lunar month `offset` months away from the one containing `jd`.
///
/// Sought, not walked. Stepping one syzygy at a time costs a 40-day bracket
/// search per month, so reaching a distant month cost time proportional to the
/// distance: the month-jump overlay grows its offset by about twelve per click
/// on the year arrow and never moves its anchor, which made the twelfth click
/// 212 ms and the fiftieth about 800 ms. Scrolling far out by wheel or keyboard
/// paid the same price.
///
/// So: land near the answer arithmetically, find the real syzygy there, work out
/// which month that actually is, and walk the remainder - which is zero or one
/// step. The cost no longer depends on `offset`.
pub fn shift(
    engine: &Engine,
    month: &LunarMonth,
    offset: i32,
    system: MonthSystem,
    observer: Observer,
    zone: &jiff::tz::TimeZone,
) -> Result<LunarMonth> {
    let target = system.boundary_elongation();
    let base = month.start_jd;

    // A neighbour is already one search away, and seeking costs one search plus
    // a possible correction. Walking is never slower here and is simpler to
    // read, so the hot path - `month_index` stepping month by month - keeps it.
    if offset.abs() <= 1 {
        let start = match offset {
            0 => base,
            1 => next_syzygy(engine, base + 1.0, target)?,
            _ => previous_syzygy(engine, base - 1.0, target)?,
        };
        return build(engine, start, system, observer, zone);
    }

    let mut start = seek(engine, base, offset, target)?;
    let mut at = index_of(start, base);

    // Zero or one iteration in practice. Bounded so that a seek which lands
    // somewhere unexpected fails loudly instead of walking the whole distance
    // and hiding the fact that the estimate was wrong.
    let mut corrections = 0;
    while at != offset {
        corrections += 1;
        if corrections > SEEK_SLACK {
            return Err(Error::NoCrossing {
                what: "lunar month by seek",
                graha: "Chandra",
                near: start,
                window_days: MEAN_SYNODIC,
            });
        }
        if at < offset {
            start = next_syzygy(engine, start + 1.0, target)?;
            at += 1;
        } else {
            start = previous_syzygy(engine, start - 1.0, target)?;
            at -= 1;
        }
    }

    build(engine, start, system, observer, zone)
}

/// The true syzygy nearest to where the `offset`th one is expected.
///
/// `previous_syzygy` rather than `next_syzygy`, so the answer is the start of the
/// month the estimate falls inside. Which month that is may be `offset` or its
/// neighbour - the estimate can land either side of the true boundary - and
/// [`index_of`] is what settles it.
fn seek(engine: &Engine, base: f64, offset: i32, target: f64) -> Result<f64> {
    previous_syzygy(engine, base + f64::from(offset) * MEAN_SYNODIC, target)
}

/// Which month `start` is, counted from the syzygy `base`.
///
/// Rounding is exact here, not approximate: see [`MEAN_SYNODIC`].
fn index_of(start: f64, base: f64) -> i32 {
    ((start - base) / MEAN_SYNODIC).round() as i32
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

    // How many rashi boundaries the Sun crossed inside the month. None is an
    // intercalary month; two is a kshaya masa, where a name is skipped.
    let sankranti_count = ((rashi_at_end + 12 - rashi_at_start) % 12) as u8;
    let adhika = sankranti_count == 0;

    // The name always comes from the rashi the Sun occupies at the new moon.
    // An intercalary month needs no adjustment: because the Sun does not leave
    // that rashi, the regular month following it derives the same name, which is
    // exactly why the pair reads "Adhika Shravana" then "Shravana". Deriving the
    // intercalary name from the next rashi instead names it after the month
    // after the one it doubles.
    let name_index = rashi_at_start;

    let first_day = first_civil_day(engine, start_jd, observer, zone)?;
    let next_month = CivilDay::new(first_civil_day(engine, end_jd, observer, zone)?, zone)?;
    let last_day = date_at(next_month.start_jd - 0.5, zone)?;

    Ok(LunarMonth {
        name: NAMES_BY_SOLAR_RASHI[name_index],
        adhika,
        sankranti_count,
        kshaya_masa_name: (sankranti_count == 2)
            .then(|| NAMES_BY_SOLAR_RASHI[(name_index + 1) % 12]),
        vikram_year: vikram_year(name_index, first_day),
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

        // Guarded, like every other place that turns a longitude into an index
        // into `Rashi::ALL`. This one was not: a longitude of exactly 360.0
        // gives 12 and indexes past the end of a twelve element array. The
        // ephemeris does not return that today, but this takes whatever the
        // engine hands it and the array has no room for the mistake.
        let index_a = ((a / RASHI_ARC) as usize).min(11);
        let index_b = ((b / RASHI_ARC) as usize).min(11);

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

#[cfg(test)]
mod seek_tests {
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use chandra_ephemeris::{Ayanamsa, NodeType, SiderealConfig};

    use super::*;

    /// The engine, built once.
    ///
    /// `Engine::new` is a process singleton, so this is the only test in this
    /// crate's lib target that may build one. The integration suite has its own
    /// in its own binary.
    fn engine() -> MutexGuard<'static, Engine> {
        static ENGINE: OnceLock<Mutex<Engine>> = OnceLock::new();
        ENGINE
            .get_or_init(|| {
                let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../src-tauri/resources/ephe");
                Mutex::new(
                    Engine::new(
                        &path,
                        SiderealConfig {
                            ayanamsa: Ayanamsa::Lahiri,
                            node_type: NodeType::Mean,
                        },
                    )
                    .expect("engine must build"),
                )
            })
            .lock()
            .expect("engine lock poisoned by an earlier failure")
    }

    fn bengaluru() -> Observer {
        Observer::new(12.97, 77.59, 920.0)
    }

    fn kolkata() -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::get("Asia/Kolkata").expect("tz database")
    }

    /// A month reached by seeking is the month the walk would have reached.
    ///
    /// The whole of `shift`'s speed rests on one claim: that the true syzygy
    /// never wanders far enough from the mean one for `index_of` to miscount. So
    /// this walks month by month - the old behaviour, still the path taken for a
    /// single step - and asserts the seek agrees at every offset along the way,
    /// in both directions and for both boundary conventions.
    ///
    /// It also asserts the margin the claim depends on, rather than leaving it in
    /// a comment: how far the true syzygy strays from `base + k * MEAN_SYNODIC`,
    /// against the 14.77 days that would be needed to round to the wrong month.
    #[test]
    fn a_sought_month_is_the_month_the_walk_would_reach() {
        let engine = engine();
        let (observer, zone) = (bengaluru(), kolkata());

        // Twelve years either way covers the reported case - the year arrow,
        // twelve clicks out, was at offset -142 - with room over it.
        const REACH: i32 = 150;

        for system in [MonthSystem::Amanta, MonthSystem::Purnimanta] {
            let target = system.boundary_elongation();
            let base = month_containing(&engine, 2_460_000.5, system, observer, &zone)
                .expect("a month to start from");

            let mut worst_drift = 0.0f64;
            let mut worst_correction = 0i32;

            for direction in [1i32, -1] {
                let mut walked = base.start_jd;

                for step in 1..=REACH {
                    let offset = direction * step;
                    walked = if direction > 0 {
                        next_syzygy(&engine, walked + 1.0, target).expect("next")
                    } else {
                        previous_syzygy(&engine, walked - 1.0, target).expect("previous")
                    };
                    let walked_month =
                        build(&engine, walked, system, observer, &zone).expect("walked month");

                    let sought = shift(&engine, &base, offset, system, observer, &zone)
                        .expect("seek must reach it");

                    // The month, which is what a reader sees.
                    assert_eq!(
                        (sought.name, sought.adhika, sought.vikram_year),
                        (
                            walked_month.name,
                            walked_month.adhika,
                            walked_month.vikram_year
                        ),
                        "{system:?} offset {offset}: the seek found a different month"
                    );

                    // And its boundary, to the refiner's tolerance rather than to
                    // the bit. Two independent refinements of one root converge
                    // from different brackets and so differ in the last places -
                    // 3e-10 days, which is 0.03 milliseconds against a printed
                    // resolution of one minute. Asserting exact equality here is
                    // asserting that the arithmetic took the same route, not that
                    // it found the same answer.
                    assert!(
                        (sought.start_jd - walked).abs() < roots::TOLERANCE_DAYS * 10.0,
                        "{system:?} offset {offset}: boundary differs by {} days",
                        (sought.start_jd - walked).abs()
                    );

                    // The margin the design rests on.
                    let drift = (walked - (base.start_jd + f64::from(offset) * MEAN_SYNODIC)).abs();
                    worst_drift = worst_drift.max(drift);

                    // And how far the estimate actually was, in months.
                    let estimated = index_of(
                        seek(&engine, base.start_jd, offset, target).expect("seek"),
                        base.start_jd,
                    );
                    worst_correction = worst_correction.max((estimated - offset).abs());
                }
            }

            assert!(
                worst_drift < MEAN_SYNODIC / 4.0,
                "{system:?}: the true syzygy strayed {worst_drift:.2} days from the mean, \
                 and {:.2} would be needed to round to the wrong month - the margin this \
                 depends on is gone",
                MEAN_SYNODIC / 2.0
            );
            assert!(
                worst_correction <= 1,
                "{system:?}: the seek was {worst_correction} months out, so the walk after \
                 it is no longer a correction"
            );
        }
    }

    /// At the far end of the clamp, consecutive offsets are consecutive months.
    ///
    /// Walking out to 2,400 to check it directly is the cost this change exists
    /// to remove. Contiguity catches the same failure for a constant price: if a
    /// seek at any distance landed a month out, the two neighbours would not be
    /// one syzygy apart.
    #[test]
    fn a_sought_month_at_the_clamp_is_contiguous_with_its_neighbour() {
        let engine = engine();
        let (observer, zone) = (bengaluru(), kolkata());
        let system = MonthSystem::Amanta;
        let target = system.boundary_elongation();
        let base = month_containing(&engine, 2_460_000.5, system, observer, &zone)
            .expect("a month to start from");

        // `MONTH_OFFSET_LIMIT` in the app is 2,400 either way.
        for offset in [-2400, -1200, -600, -13, 13, 600, 1200, 2400] {
            let here = shift(&engine, &base, offset, system, observer, &zone).expect("here");
            let next = shift(&engine, &base, offset + 1, system, observer, &zone).expect("next");

            assert_eq!(
                next.start_jd,
                next_syzygy(&engine, here.start_jd + 1.0, target).expect("the syzygy after"),
                "offset {offset} and {} are not consecutive months",
                offset + 1
            );
            assert_eq!(
                index_of(here.start_jd, base.start_jd),
                offset,
                "offset {offset} did not round back to itself"
            );
        }
    }
}
