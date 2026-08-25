//! Divisions of a Sun-Moon angle: tithi, karana and yoga.
//!
//! Three of the five limbs of a panchanga are the same calculation with a
//! different arc and, in one case, a plus instead of a minus:
//!
//! | Limb   | Angle          | Arc      | Count |
//! |--------|----------------|----------|-------|
//! | Tithi  | Moon − Sun     | 12°      | 30    |
//! | Karana | Moon − Sun     | 6°       | 60    |
//! | Yoga   | Moon + Sun     | 13°20′   | 27    |
//!
//! What makes one routine enough is that all three angles only ever increase.
//! The Moon gains on the Sun at between about 10.9 and 14.5 degrees a day and
//! never loses ground, and their sum climbs faster still - so unlike a rashi or
//! a nakshatra there is no retrograde case, the index only ascends, and a
//! boundary is crossed exactly once.
//!
//! This module finds the spans. It does not name them: a tithi's name depends on
//! its paksha and a karana's on where it sits in a cycle of eleven, and neither
//! is a fact about an angle.

use chandra_ephemeris::{Engine, Graha, Source};

use crate::error::{Error, Result};
use crate::roots::{self, Bracket};
use crate::time::CivilDay;

/// Which Sun-Moon angle a division is measured on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Angle {
    /// Elongation: the Moon's longitude less the Sun's. Tithi and karana.
    Difference,
    /// The two longitudes added. Yoga.
    Sum,
}

impl Angle {
    fn of(self, moon: f64, sun: f64) -> f64 {
        match self {
            Angle::Difference => (moon - sun).rem_euclid(360.0),
            Angle::Sum => (moon + sun).rem_euclid(360.0),
        }
    }

    /// Fastest the angle can move, in degrees a day, rounded up.
    ///
    /// The Moon runs between about 11.8 and 15.4 degrees a day and the Sun
    /// between about 0.95 and 1.02, so the sum is the faster of the two. Used
    /// only to choose a scan step, so an over-estimate is safe and an
    /// under-estimate is not.
    const fn max_degrees_per_day(self) -> f64 {
        match self {
            Angle::Difference => 15.0,
            Angle::Sum => 17.0,
        }
    }
}

/// A division the angle occupies, with the instants it was entered and left.
///
/// `entry` and `exit` are the true crossing instants and routinely fall outside
/// the day: a tithi that began yesterday evening reports when it actually began.
/// They are `None` only where the crossing did not resolve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawSpan {
    /// 1-based, counted from the angle's zero.
    pub index: u8,
    pub entry: Option<f64>,
    pub exit: Option<f64>,
    /// True for the division in force at the day's reference instant.
    pub prevailing: bool,
    pub source: Source,
}

/// How the zodiac is cut for one limb.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Divisions {
    pub angle: Angle,
    pub count: u8,
    /// What the limb is called in an error, so a failure names the thing that
    /// failed rather than the machinery it failed in.
    pub what: &'static str,
}

impl Divisions {
    pub const TITHI: Self = Self {
        angle: Angle::Difference,
        count: 30,
        what: "tithi boundary",
    };

    pub const KARANA: Self = Self {
        angle: Angle::Difference,
        count: 60,
        what: "karana boundary",
    };

    pub const YOGA: Self = Self {
        angle: Angle::Sum,
        count: 27,
        what: "yoga boundary",
    };

    pub fn arc(self) -> f64 {
        360.0 / self.count as f64
    }

    /// The division in force at an angle, 1-based.
    pub fn index_at(self, angle: f64) -> u8 {
        let index = (angle / self.arc()).floor() as i64 + 1;
        index.clamp(1, self.count as i64) as u8
    }

    /// Angle at which a division begins. The last one ends at 360, not 0:
    /// the search brackets a rising crossing, and zero would not be one.
    fn start_degrees(self, index: u8) -> f64 {
        (index as f64 - 1.0) * self.arc()
    }

    fn end_degrees(self, index: u8) -> f64 {
        index as f64 * self.arc()
    }

    /// Scan step. Fine enough that the angle cannot step over a whole division
    /// and back inside one step, which is what would let two boundaries share a
    /// bracket and one of them be lost.
    ///
    /// A quarter of the time the fastest possible motion needs to cross one
    /// arc. For a karana - the narrowest at 6 degrees - that is about 2.4 hours,
    /// and the scan below runs at an hour, so the margin is real rather than
    /// nominal.
    fn scan_step_days(self) -> f64 {
        (self.arc() / self.angle.max_degrees_per_day() / 4.0).min(1.0 / 24.0)
    }

    /// How far out to search for a boundary lying outside the day.
    ///
    /// A tithi is at most about 26 hours, so a day's first began less than that
    /// before the day did. Twice the longest possible division is clear of the
    /// worst case without letting the wrapped difference change sign twice.
    fn search_days(self) -> f64 {
        // The slowest the angle moves is roughly two thirds of its fastest.
        (self.arc() / (self.angle.max_degrees_per_day() * 0.6) * 2.0).max(1.0)
    }
}

/// The angle in force at an instant.
pub fn at(engine: &Engine, jd: f64, angle: Angle) -> Result<f64> {
    let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
    Ok(angle.of(bodies[0].longitude, bodies[1].longitude))
}

/// Every division touching `day`, in the order it occupies them.
///
/// `reference_jd` selects which one is marked prevailing - sunrise, where the
/// Sun rises, because that is the instant a panchanga names a day from.
pub fn spans_in_day(
    engine: &Engine,
    day: &CivilDay,
    reference_jd: f64,
    divisions: Divisions,
) -> Result<Vec<RawSpan>> {
    let samples = sample(engine, day.start_jd, day.end_jd, divisions)?;

    let mut spans = Vec::new();
    let mut run_start = 0usize;

    for position in 1..=samples.len() {
        let ends_here =
            position == samples.len() || samples[position].index != samples[run_start].index;
        if !ends_here {
            continue;
        }

        let index = samples[run_start].index;

        let entry = if run_start == 0 {
            search(
                engine,
                day.start_jd,
                divisions.start_degrees(index),
                divisions,
                Backward,
            )?
        } else {
            refine_between(
                engine,
                &samples[run_start - 1],
                &samples[run_start],
                divisions.start_degrees(index),
                divisions.angle,
            )?
        };

        let exit = if position == samples.len() {
            search(
                engine,
                day.end_jd,
                divisions.end_degrees(index),
                divisions,
                Forward,
            )?
        } else {
            refine_between(
                engine,
                &samples[position - 1],
                &samples[position],
                divisions.end_degrees(index),
                divisions.angle,
            )?
        };

        // Decided on the refined boundaries, not on the samples they were found
        // between. The scan grid is up to an hour wide, and a boundary crossed
        // inside that hour before sunrise would otherwise be attributed to the
        // wrong side of it. The sample comparison survives only as the fallback
        // for a boundary that did not resolve, where there is nothing finer.
        let prevailing = match (entry, exit) {
            (Some(entry), Some(exit)) => reference_jd >= entry && reference_jd < exit,
            _ => {
                reference_jd >= samples[run_start].jd
                    && (position == samples.len() || reference_jd < samples[position].jd)
            }
        };

        spans.push(RawSpan {
            index,
            entry,
            exit,
            prevailing,
            source: Source::weakest(samples[run_start..position].iter().map(|s| s.source)),
        });

        run_start = position;
    }

    if spans.is_empty() {
        return Err(Error::NoCrossing {
            what: divisions.what,
            graha: "Chandra",
            near: day.start_jd,
            window_days: day.length(),
        });
    }

    // Exactly one span is prevailing. A reference sitting exactly on a sample
    // boundary can miss every comparison, so the last span takes it as a defined
    // fallback rather than leaving the day with none.
    if !spans.iter().any(|span| span.prevailing) {
        if let Some(last) = spans.last_mut() {
            last.prevailing = true;
        }
    }

    Ok(spans)
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    jd: f64,
    angle: f64,
    index: u8,
    source: Source,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Forward,
    Backward,
}
use Direction::{Backward, Forward};

fn sample(engine: &Engine, start: f64, end: f64, divisions: Divisions) -> Result<Vec<Sample>> {
    let step = divisions.scan_step_days();
    let mut samples = Vec::new();
    let mut jd = start;
    loop {
        let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
        let angle = divisions.angle.of(bodies[0].longitude, bodies[1].longitude);
        samples.push(Sample {
            jd,
            angle,
            index: divisions.index_at(angle),
            source: Source::weakest([bodies[0].source, bodies[1].source]),
        });
        if jd >= end {
            break;
        }
        jd = (jd + step).min(end);
    }
    Ok(samples)
}

fn crossing(engine: &Engine, target: f64, angle: Angle) -> impl Fn(f64) -> Option<f64> + '_ {
    move |jd| {
        at(engine, jd, angle)
            .ok()
            .map(|value| roots::signed_delta(value, target))
    }
}

fn refine_between(
    engine: &Engine,
    from: &Sample,
    to: &Sample,
    target: f64,
    angle: Angle,
) -> Result<Option<f64>> {
    let bracket = Bracket {
        lo: from.jd,
        hi: to.jd,
        f_lo: roots::signed_delta(from.angle, target),
        f_hi: roots::signed_delta(to.angle, target),
    };
    Ok(roots::refine(bracket, crossing(engine, target, angle)))
}

/// The boundary at `target` nearest to `from`, in the given direction.
fn search(
    engine: &Engine,
    from: f64,
    target: f64,
    divisions: Divisions,
    direction: Direction,
) -> Result<Option<f64>> {
    let f = crossing(engine, target, divisions.angle);
    let reach = divisions.search_days();
    let (start, end) = match direction {
        Forward => (from, from + reach),
        Backward => (from - reach, from),
    };

    let found = roots::brackets(start, end, divisions.scan_step_days(), &f)
        .into_iter()
        // The angle only increases, so the crossing is always negative to
        // positive. The wrapped difference also changes sign half a revolution
        // away, and that is a different division entirely.
        .filter(|bracket| !(bracket.f_lo > 0.0 && bracket.f_hi < 0.0))
        .filter_map(|bracket| roots::refine(bracket, &f))
        .collect::<Vec<_>>();

    Ok(match direction {
        Forward => found.into_iter().next(),
        Backward => found.into_iter().next_back(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_limb_cuts_the_circle_into_the_divisions_it_is_defined_as() {
        assert_eq!(Divisions::TITHI.arc(), 12.0);
        assert_eq!(Divisions::KARANA.arc(), 6.0);
        assert!((Divisions::YOGA.arc() - 360.0 / 27.0).abs() < 1e-12);
    }

    #[test]
    fn an_index_is_one_based_and_never_leaves_its_range() {
        assert_eq!(Divisions::TITHI.index_at(0.0), 1);
        assert_eq!(Divisions::TITHI.index_at(11.99), 1);
        assert_eq!(Divisions::TITHI.index_at(12.0), 2);
        // RESEARCH.md R-03: 20 August 2026, elongation 88.72 degrees.
        assert_eq!(Divisions::TITHI.index_at(88.72), 8);
        assert_eq!(Divisions::TITHI.index_at(359.99), 30);
        // Clamped rather than wrapping to 31: an angle of exactly 360 is the
        // same point as 0 and belongs to the division that ends there.
        assert_eq!(Divisions::TITHI.index_at(360.0), 30);

        assert_eq!(Divisions::KARANA.index_at(0.0), 1);
        assert_eq!(Divisions::KARANA.index_at(6.0), 2);
        assert_eq!(Divisions::KARANA.index_at(359.99), 60);

        assert_eq!(Divisions::YOGA.index_at(0.0), 1);
        assert_eq!(Divisions::YOGA.index_at(359.99), 27);
    }

    /// A karana is exactly half a tithi, so the two indices must agree at every
    /// angle. If they ever disagree, one of the two arcs is wrong.
    #[test]
    fn a_karana_is_half_a_tithi_at_every_angle() {
        let mut angle = 0.0;
        while angle < 360.0 {
            let tithi = Divisions::TITHI.index_at(angle);
            let karana = Divisions::KARANA.index_at(angle);
            assert_eq!(
                tithi,
                karana.div_ceil(2),
                "karana {karana} and tithi {tithi} disagree at {angle} degrees"
            );
            angle += 0.37;
        }
    }

    #[test]
    fn the_scan_step_cannot_step_over_a_division() {
        for divisions in [Divisions::TITHI, Divisions::KARANA, Divisions::YOGA] {
            let travelled = divisions.scan_step_days() * divisions.angle.max_degrees_per_day();
            assert!(
                travelled < divisions.arc(),
                "{}: one step covers {travelled} degrees of a {} degree arc",
                divisions.what,
                divisions.arc()
            );
        }
    }

    /// The outward search has to reach a boundary that lies outside the day, or
    /// the first and last spans of every day report no entry and no exit.
    #[test]
    fn the_search_reaches_past_the_longest_the_division_can_last() {
        for divisions in [Divisions::TITHI, Divisions::KARANA, Divisions::YOGA] {
            // Slowest realistic motion, well below anything observed.
            let longest = divisions.arc() / (divisions.angle.max_degrees_per_day() * 0.65);
            assert!(
                divisions.search_days() > longest,
                "{}: searches {} days for something up to {longest} days long",
                divisions.what,
                divisions.search_days()
            );
        }
    }
}
