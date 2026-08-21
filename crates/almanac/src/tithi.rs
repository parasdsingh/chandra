//! Tithis: the lunar days a panchanga is built on.
//!
//! A tithi is 12 degrees of the Moon's elongation from the Sun, so a lunar month
//! holds exactly thirty of them. It is not a day. The Moon's motion is uneven,
//! so a tithi runs anywhere from about 19 to 26 hours, and against a 24-hour
//! civil day that produces two ordinary outcomes: a tithi that contains no
//! sunrise, which no civil day is named after (kshaya), and a tithi that
//! contains two, which two consecutive civil days share (vriddhi). Roughly one
//! day a month is one or the other. Neither is an error and neither is smoothed
//! over here.
//!
//! Elongation only ever increases - the Moon gains on the Sun at between about
//! 10.9 and 15.4 degrees a day and never loses ground - so unlike a rashi or a
//! nakshatra there is no retrograde case and the index only ever ascends.

use chandra_ephemeris::{Engine, Graha, Source};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::roots::{self, Bracket};
use crate::time::{CivilDay, Moment};

/// Degrees of elongation per tithi.
pub const TITHI_ARC: f64 = 12.0;

/// Tithis in a lunar month.
pub const TITHI_COUNT: u8 = 30;

/// Half a lunar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Paksha {
    /// Waxing: new moon to full.
    Shukla,
    /// Waning: full moon to new.
    Krishna,
}

impl Paksha {
    pub const fn name(self) -> &'static str {
        match self {
            Paksha::Shukla => "Shukla",
            Paksha::Krishna => "Krishna",
        }
    }
}

/// Which instant a day's tithi was taken at.
///
/// Sunrise is the traditional rule. Local noon is the substitute where the Sun
/// does not rise at all, which is a fact about the latitude rather than a
/// failure, and is stated as such.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reference {
    Sunrise,
    LocalNoon,
}

/// A civil day's place in a tithi that spans two sunrises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vriddhi {
    First,
    Second,
}

/// The fourteen numbered tithis. The fifteenth is named for the syzygy it ends
/// at, which differs by paksha, so it is not in the table.
const NUMBERED: [&str; 14] = [
    "Pratipada",
    "Dvitiya",
    "Tritiya",
    "Chaturthi",
    "Panchami",
    "Shashthi",
    "Saptami",
    "Ashtami",
    "Navami",
    "Dashami",
    "Ekadashi",
    "Dvadashi",
    "Trayodashi",
    "Chaturdashi",
];

/// One tithi, identified by its astronomical index.
///
/// The index is always counted from Shukla Pratipada, whatever month system is
/// in force. Amanta and purnimanta months begin at opposite ends of it, and that
/// ordering belongs to the month rather than to the tithi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tithi(u8);

impl Tithi {
    /// The tithi in force at an elongation, which must be in `[0, 360)`.
    pub fn from_elongation(elongation: f64) -> Self {
        let index = (elongation / TITHI_ARC).floor() as i64 + 1;
        Self(index.clamp(1, TITHI_COUNT as i64) as u8)
    }

    pub fn from_index(index: u8) -> Result<Self> {
        if index == 0 || index > TITHI_COUNT {
            return Err(Error::TimeZone(format!("tithi index {index} out of range")));
        }
        Ok(Self(index))
    }

    /// 1 to 30, from Shukla Pratipada.
    pub const fn index(self) -> u8 {
        self.0
    }

    pub const fn paksha(self) -> Paksha {
        if self.0 <= 15 {
            Paksha::Shukla
        } else {
            Paksha::Krishna
        }
    }

    /// 1 to 15, within the paksha.
    pub const fn number(self) -> u8 {
        if self.0 <= 15 {
            self.0
        } else {
            self.0 - 15
        }
    }

    /// `Ashtami`, `Purnima`, `Amavasya`. The paksha is carried separately.
    pub const fn name(self) -> &'static str {
        let number = self.number();
        if number == 15 {
            match self.paksha() {
                Paksha::Shukla => "Purnima",
                Paksha::Krishna => "Amavasya",
            }
        } else {
            NUMBERED[number as usize - 1]
        }
    }

    /// `Shukla Ashtami`. What the day view and the spoken label say.
    pub fn full_name(self) -> String {
        format!("{} {}", self.paksha().name(), self.name())
    }

    /// Elongation at which this tithi begins.
    pub fn start_degrees(self) -> f64 {
        (self.0 as f64 - 1.0) * TITHI_ARC
    }

    /// Elongation at which it ends. 360 for the last, which is 0 wrapped.
    pub fn end_degrees(self) -> f64 {
        self.0 as f64 * TITHI_ARC
    }

    /// The next tithi, wrapping 30 to 1.
    pub const fn next(self) -> Self {
        Self(if self.0 == TITHI_COUNT { 1 } else { self.0 + 1 })
    }

    /// How many tithis separate `self` from `other`, going forwards.
    ///
    /// Used to find what the grid skipped between two sunrises. Elongation only
    /// increases, so forwards is the only direction there is.
    pub const fn steps_to(self, other: Self) -> u8 {
        (other.0 + TITHI_COUNT - self.0) % TITHI_COUNT
    }
}

/// A tithi the grid never names, because it began and ended between two
/// sunrises.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedTithi {
    pub index: u8,
    pub paksha: Paksha,
    pub number: u8,
    /// `Shukla Shashthi`: the full form, because this is only ever read aloud or
    /// printed as a sentence, never as a cell.
    pub name: String,
}

impl SkippedTithi {
    pub fn new(tithi: Tithi) -> Self {
        Self {
            index: tithi.index(),
            paksha: tithi.paksha(),
            number: tithi.number(),
            name: tithi.full_name(),
        }
    }
}

/// The lunar day a grid cell is named after.
///
/// Everything the cell draws and nothing it does not: no boundary times, which
/// belong to the day view and cost root-finding that 42 cells would pay 42 times
/// over for a number nobody reads until they open a day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellTithi {
    /// 1 to 30, from Shukla Pratipada, in both month systems. For sorting and
    /// equality; never for display, because amanta and purnimanta count from
    /// opposite ends and the cell shows the number within the paksha.
    pub index: u8,
    /// 1 to 15, within the paksha. What the cell prints.
    pub number: u8,
    pub paksha: Paksha,
    /// `Ashtami`, `Purnima`, `Amavasya`.
    pub name: String,
    /// Which instant the tithi was taken at.
    pub reference: Reference,
    /// The reference sunrise. `None` where the Sun does not rise.
    pub sunrise: Option<Moment>,
    /// Tithis that began and ended inside this civil day, so no day is named
    /// after them and the grid's numbers jump. Empty on almost every day.
    pub kshaya: Vec<SkippedTithi>,
    /// Set when this tithi spans two sunrises and two days carry the number.
    pub vriddhi: Option<Vriddhi>,
}

/// A tithi touching one civil day, with the instants it began and will end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TithiSpan {
    /// 1 to 30, from Shukla Pratipada, in both month systems.
    pub index: u8,
    /// 1 to 15, within the paksha.
    pub number: u8,
    pub paksha: Paksha,
    /// `Ashtami`, `Purnima`, `Amavasya`.
    pub name: String,
    /// The true crossing instants, never clamped to the day. `None` only when
    /// refinement did not converge, in which case no time is guessed.
    pub entry: Option<Moment>,
    pub exit: Option<Moment>,
    /// Sunrises inside `[entry, exit)`. The primitive the other two states are
    /// read off: 0 is a kshaya, 2 a vriddhi. Supplied instead of two booleans so
    /// a payload claiming both cannot be constructed.
    pub sunrises: u8,
    /// In force at the day's reference instant. Exactly one span per day.
    pub prevailing: bool,
    pub source: Source,
}

/// The Moon's elongation from the Sun, `[0, 360)`.
pub fn elongation(engine: &Engine, jd: f64) -> Result<f64> {
    let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
    Ok((bodies[0].longitude - bodies[1].longitude).rem_euclid(360.0))
}

/// The tithi in force at an instant.
pub fn at(engine: &Engine, jd: f64) -> Result<Tithi> {
    Ok(Tithi::from_elongation(elongation(engine, jd)?))
}

/// How far out to search for a boundary that lies outside the day.
///
/// A tithi is at most about 26 hours, so a day's first tithi began less than
/// that before the day did. Two and a half days is comfortably clear of the
/// worst case without letting the wrapped difference change sign twice.
const SEARCH_DAYS: f64 = 2.5;

/// Scan step. The Moon moves at most about 15 degrees a day, so an hour cannot
/// step over a 12 degree boundary and back.
const SCAN_STEP_DAYS: f64 = 1.0 / 24.0;

/// Every tithi touching `day`, in the order it occupies them.
///
/// `sunrises` are the sunrise instants of the civil days the spans may reach
/// into - the day itself and its two neighbours - and are what `sunrises` on
/// each span is counted from. Counting them directly rather than comparing tithi
/// numbers between consecutive days is deliberate: a comparison cannot tell a
/// genuine kshaya from a day whose tithi failed to compute.
pub fn spans_in_day(
    engine: &Engine,
    day: &CivilDay,
    reference_jd: f64,
    sunrises: &[f64],
) -> Result<Vec<TithiSpan>> {
    let samples = sample(engine, day.start_jd, day.end_jd)?;

    let mut spans = Vec::new();
    let mut run_start = 0usize;

    for position in 1..=samples.len() {
        let ends_here =
            position == samples.len() || samples[position].tithi != samples[run_start].tithi;
        if !ends_here {
            continue;
        }

        let tithi = samples[run_start].tithi;

        let entry = if run_start == 0 {
            search(engine, day.start_jd, tithi.start_degrees(), Backward)?
        } else {
            refine_between(
                engine,
                &samples[run_start - 1],
                &samples[run_start],
                tithi.start_degrees(),
            )?
        };

        let exit = if position == samples.len() {
            search(engine, day.end_jd, tithi.end_degrees(), Forward)?
        } else {
            refine_between(
                engine,
                &samples[position - 1],
                &samples[position],
                tithi.end_degrees(),
            )?
        };

        let prevailing = reference_jd >= samples[run_start].jd
            && (position == samples.len() || reference_jd < samples[position].jd);

        spans.push(TithiSpan {
            index: tithi.index(),
            number: tithi.number(),
            paksha: tithi.paksha(),
            name: tithi.name().to_string(),
            entry: entry.map(|jd| day.moment(jd)).transpose()?,
            exit: exit.map(|jd| day.moment(jd)).transpose()?,
            sunrises: count_sunrises(entry, exit, sunrises),
            prevailing,
            source: Source::weakest(samples[run_start..position].iter().map(|s| s.source)),
        });

        run_start = position;
    }

    if spans.is_empty() {
        return Err(Error::NoCrossing {
            what: "tithi boundary",
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

/// Sunrises inside `[entry, exit)`.
///
/// An unresolved boundary yields zero rather than a guess, and the day view
/// prints the boundary as unavailable rather than inferring a state from it.
fn count_sunrises(entry: Option<f64>, exit: Option<f64>, sunrises: &[f64]) -> u8 {
    let (Some(entry), Some(exit)) = (entry, exit) else {
        return 0;
    };
    sunrises
        .iter()
        .filter(|&&jd| jd >= entry && jd < exit)
        .count()
        .min(u8::MAX as usize) as u8
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    jd: f64,
    elongation: f64,
    tithi: Tithi,
    source: Source,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Forward,
    Backward,
}
use Direction::{Backward, Forward};

fn sample(engine: &Engine, start: f64, end: f64) -> Result<Vec<Sample>> {
    let mut samples = Vec::new();
    let mut jd = start;
    loop {
        let bodies = engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
        let elongation = (bodies[0].longitude - bodies[1].longitude).rem_euclid(360.0);
        samples.push(Sample {
            jd,
            elongation,
            tithi: Tithi::from_elongation(elongation),
            source: Source::weakest([bodies[0].source, bodies[1].source]),
        });
        if jd >= end {
            break;
        }
        jd = (jd + SCAN_STEP_DAYS).min(end);
    }
    Ok(samples)
}

fn crossing(engine: &Engine, target: f64) -> impl Fn(f64) -> Option<f64> + '_ {
    move |jd| {
        elongation(engine, jd)
            .ok()
            .map(|e| roots::signed_delta(e, target))
    }
}

fn refine_between(engine: &Engine, from: &Sample, to: &Sample, target: f64) -> Result<Option<f64>> {
    let bracket = Bracket {
        lo: from.jd,
        hi: to.jd,
        f_lo: roots::signed_delta(from.elongation, target),
        f_hi: roots::signed_delta(to.elongation, target),
    };
    Ok(roots::refine(bracket, crossing(engine, target)))
}

/// The boundary at `target` nearest to `from`, in the given direction.
fn search(engine: &Engine, from: f64, target: f64, direction: Direction) -> Result<Option<f64>> {
    let f = crossing(engine, target);
    let (start, end) = match direction {
        Forward => (from, from + SEARCH_DAYS),
        Backward => (from - SEARCH_DAYS, from),
    };

    let found = roots::brackets(start, end, SCAN_STEP_DAYS, &f)
        .into_iter()
        // Elongation only increases, so the crossing is always negative to
        // positive. The wrapped difference also changes sign half a revolution
        // away, and that is a different tithi entirely.
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
    fn an_elongation_maps_to_the_tithi_a_panchanga_would_print() {
        // RESEARCH.md R-03: 20 August 2026, elongation 88.72 degrees.
        let tithi = Tithi::from_elongation(88.72);
        assert_eq!(tithi.index(), 8);
        assert_eq!(tithi.paksha(), Paksha::Shukla);
        assert_eq!(tithi.number(), 8);
        assert_eq!(tithi.name(), "Ashtami");
        assert_eq!(tithi.full_name(), "Shukla Ashtami");
    }

    #[test]
    fn the_two_syzygies_are_the_fifteenth_of_each_paksha() {
        let purnima = Tithi::from_elongation(180.0);
        assert_eq!(purnima.index(), 16);
        // 180 degrees exactly is the first instant of Krishna Pratipada: the
        // full moon ends Shukla Purnima, it does not fall inside it.
        assert_eq!(purnima.paksha(), Paksha::Krishna);
        assert_eq!(purnima.number(), 1);

        assert_eq!(Tithi::from_elongation(179.9).name(), "Purnima");
        assert_eq!(Tithi::from_elongation(179.9).paksha(), Paksha::Shukla);
        assert_eq!(Tithi::from_elongation(359.9).name(), "Amavasya");
        assert_eq!(Tithi::from_elongation(359.9).paksha(), Paksha::Krishna);
        assert_eq!(Tithi::from_elongation(0.0).name(), "Pratipada");
    }

    #[test]
    fn every_index_has_a_name_and_a_paksha() {
        for index in 1..=TITHI_COUNT {
            let tithi = Tithi::from_index(index).expect("in range");
            assert!(!tithi.name().is_empty());
            assert!((1..=15).contains(&tithi.number()));
            assert_eq!(tithi.index(), index);
        }
        assert!(Tithi::from_index(0).is_err());
        assert!(Tithi::from_index(31).is_err());
    }

    #[test]
    fn steps_wrap_at_the_end_of_the_month() {
        let amavasya = Tithi::from_index(30).unwrap();
        let pratipada = Tithi::from_index(1).unwrap();
        assert_eq!(amavasya.steps_to(pratipada), 1);
        assert_eq!(amavasya.next(), pratipada);
        assert_eq!(pratipada.steps_to(Tithi::from_index(3).unwrap()), 2);
        assert_eq!(pratipada.steps_to(pratipada), 0);
    }

    #[test]
    fn a_boundary_is_where_the_arc_says_it_is() {
        let tithi = Tithi::from_index(8).unwrap();
        assert_eq!(tithi.start_degrees(), 84.0);
        assert_eq!(tithi.end_degrees(), 96.0);
        assert_eq!(Tithi::from_index(30).unwrap().end_degrees(), 360.0);
    }
}
