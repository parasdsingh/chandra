//! Finding when a graha enters and leaves a zodiacal division.
//!
//! One routine serves rashis and nakshatras for every graha, prograde or
//! retrograde. The shape of the problem is always the same: sample the division
//! index across an interval, and wherever it changes, refine the exact boundary
//! crossing that caused it.

use chandra_ephemeris::{Engine, Graha, Source};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::roots::{self, Bracket};
use crate::time::{CivilDay, Moment};
use crate::zodiac::{NAKSHATRA_ARC, RASHI_ARC};

/// Which division of the zodiac is being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Division {
    Rashi,
    Nakshatra,
}

impl Division {
    pub const fn arc(self) -> f64 {
        match self {
            Division::Rashi => RASHI_ARC,
            Division::Nakshatra => NAKSHATRA_ARC,
        }
    }

    pub const fn count(self) -> usize {
        match self {
            Division::Rashi => 12,
            Division::Nakshatra => 27,
        }
    }

    const fn what(self) -> &'static str {
        match self {
            Division::Rashi => "rashi boundary",
            Division::Nakshatra => "nakshatra boundary",
        }
    }

    fn index_of(self, longitude: f64) -> usize {
        ((longitude.rem_euclid(360.0) / self.arc()) as usize).min(self.count() - 1)
    }
}

/// A division the graha occupies, with the instants it was entered and will be
/// left.
///
/// `entry` and `exit` are the true crossing instants and routinely fall outside
/// the day being displayed - a nakshatra entered two days ago still reports when
/// it was actually entered rather than a value clamped to the day's edge.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Span {
    /// Index within the division: 0-11 for rashi, 0-26 for nakshatra.
    pub index: usize,
    pub entry: Moment,
    pub exit: Moment,
    /// True for the division in force at the day's reference instant, which is
    /// sunrise where there is one. Panchanga names a day after the division
    /// prevailing at sunrise, so this is the one to show first.
    pub prevailing: bool,
    pub source: Source,
}

/// How far to search outwards for a crossing before giving up.
///
/// Generous enough for the slowest case by a wide margin: Shani takes about
/// 2.5 years to cross a rashi, and the nodes about 1.5 years.
const MAX_SEARCH_DAYS: f64 = 1200.0;

/// Every division the graha occupies during `day`, in the order it occupies them.
///
/// `reference_jd` selects which one is marked prevailing.
pub fn divisions_in_day(
    engine: &Engine,
    graha: Graha,
    day: &CivilDay,
    division: Division,
    reference_jd: f64,
) -> Result<Vec<Span>> {
    let step = graha.scan_step_days();
    let samples = sample_indices(engine, graha, day.start_jd, day.end_jd, step, division)?;

    // Consecutive samples sharing an index form one occupancy run. Two runs with
    // the same index can occur in a single day only if the graha crossed out and
    // straight back, which needs a station within hours of a boundary; keeping
    // them separate is correct rather than merging by index.
    let mut spans = Vec::new();
    let mut run_start = 0usize;

    for position in 1..=samples.len() {
        let ends_here =
            position == samples.len() || samples[position].index != samples[run_start].index;
        if !ends_here {
            continue;
        }

        let index = samples[run_start].index;

        let entry_jd = if run_start == 0 {
            search_crossing(
                engine,
                graha,
                division,
                day.start_jd,
                index,
                Direction::Backward,
            )?
        } else {
            refine_between(
                engine,
                graha,
                division,
                &samples[run_start - 1],
                &samples[run_start],
            )?
        };

        let exit_jd = if position == samples.len() {
            search_crossing(
                engine,
                graha,
                division,
                day.end_jd,
                index,
                Direction::Forward,
            )?
        } else {
            refine_between(
                engine,
                graha,
                division,
                &samples[position - 1],
                &samples[position],
            )?
        };

        let prevailing = reference_jd >= samples[run_start].jd
            && (position == samples.len() || reference_jd < samples[position].jd);

        spans.push(Span {
            index,
            entry: day.moment(entry_jd)?,
            exit: day.moment(exit_jd)?,
            prevailing,
            source: Source::weakest(samples[run_start..position].iter().map(|s| s.source)),
        });

        run_start = position;
    }

    // A day is never empty, but if sampling produced nothing the caller must not
    // receive a silently empty list.
    if spans.is_empty() {
        return Err(Error::NoCrossing {
            what: division.what(),
            graha: graha.name(),
            near: day.start_jd,
            window_days: day.length(),
        });
    }

    // Exactly one span must be prevailing. Floating point comparison of the
    // reference against sample times can miss when the reference sits exactly on
    // a boundary sample, so the last span takes it as a defined fallback.
    if !spans.iter().any(|s| s.prevailing) {
        if let Some(last) = spans.last_mut() {
            last.prevailing = true;
        }
    }

    Ok(spans)
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    jd: f64,
    longitude: f64,
    index: usize,
    source: Source,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Forward,
    Backward,
}

fn sample_indices(
    engine: &Engine,
    graha: Graha,
    start: f64,
    end: f64,
    step: f64,
    division: Division,
) -> Result<Vec<Sample>> {
    let mut samples = Vec::new();
    let mut jd = start;
    loop {
        samples.push(sample(engine, graha, jd, division)?);
        if jd >= end {
            break;
        }
        jd = (jd + step).min(end);
    }
    Ok(samples)
}

fn sample(engine: &Engine, graha: Graha, jd: f64, division: Division) -> Result<Sample> {
    let position = engine.position(jd, graha)?;
    Ok(Sample {
        jd,
        longitude: position.longitude,
        index: division.index_of(position.longitude),
        source: position.source,
    })
}

/// The boundary lying between two samples of differing index.
///
/// Which of the two boundaries was crossed depends on the direction of travel,
/// and a retrograde graha crosses the lower one going backwards. Taking the
/// boundary from the arithmetic mean of the two indices would be wrong at the
/// 26-to-0 wraparound, so it is derived from the direction instead.
fn boundary_between(division: Division, from: &Sample, to: &Sample) -> f64 {
    let count = division.count();
    let ascending = (to.index + count - from.index) % count == 1;
    let boundary_index = if ascending { to.index } else { from.index };
    boundary_index as f64 * division.arc()
}

fn refine_between(
    engine: &Engine,
    graha: Graha,
    division: Division,
    from: &Sample,
    to: &Sample,
) -> Result<f64> {
    let target = boundary_between(division, from, to);
    let bracket = Bracket {
        lo: from.jd,
        hi: to.jd,
        f_lo: roots::signed_delta(from.longitude, target),
        f_hi: roots::signed_delta(to.longitude, target),
    };

    roots::refine(bracket, |jd| {
        engine
            .position(jd, graha)
            .ok()
            .map(|p| roots::signed_delta(p.longitude, target))
    })
    .ok_or(Error::NoCrossing {
        what: division.what(),
        graha: graha.name(),
        near: from.jd,
        window_days: to.jd - from.jd,
    })
}

/// Searches outwards from `origin` for the crossing that bounds occupancy of
/// `index`, stepping until the division index differs and then refining.
fn search_crossing(
    engine: &Engine,
    graha: Graha,
    division: Division,
    origin: f64,
    index: usize,
    direction: Direction,
) -> Result<f64> {
    let step = match direction {
        Direction::Forward => graha.scan_step_days(),
        Direction::Backward => -graha.scan_step_days(),
    };

    let mut previous = sample(engine, graha, origin, division)?;
    let mut travelled = 0.0f64;

    while travelled < MAX_SEARCH_DAYS {
        let next = sample(engine, graha, previous.jd + step, division)?;
        travelled += step.abs();

        if next.index != index {
            // Order the pair by time so the bracket is well formed regardless of
            // the search direction.
            let (from, to) = match direction {
                Direction::Forward => (previous, next),
                Direction::Backward => (next, previous),
            };
            return refine_between(engine, graha, division, &from, &to);
        }
        previous = next;
    }

    Err(Error::NoCrossing {
        what: division.what(),
        graha: graha.name(),
        near: origin,
        window_days: MAX_SEARCH_DAYS,
    })
}
