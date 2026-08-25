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

use chandra_ephemeris::{Engine, Source};
use serde::{Deserialize, Serialize};

use crate::angles::{self, Angle, Divisions};
use crate::error::{Error, Result};
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
    ///
    /// `None` where the count is not a fact about the tithi: a boundary that did
    /// not resolve leaves nothing to count between, and a latitude where the Sun
    /// rises on none of the three days has no sunrises to count at all.
    pub sunrises: Option<u8>,
    /// In force at the day's reference instant. Exactly one span per day.
    pub prevailing: bool,
    pub source: Source,
}

/// The Moon's elongation from the Sun, `[0, 360)`.
pub fn elongation(engine: &Engine, jd: f64) -> Result<f64> {
    angles::at(engine, jd, Angle::Difference)
}

/// The tithi in force at an instant.
pub fn at(engine: &Engine, jd: f64) -> Result<Tithi> {
    Ok(Tithi::from_elongation(elongation(engine, jd)?))
}


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
    // The scan, the boundary refinement and the prevailing test are the same
    // ones a karana and a yoga need, and live in `angles`. What is left here is
    // the part that is about tithis rather than about angles: the paksha, the
    // name, and the sunrise count the kshaya and vriddhi states are read off.
    let raw = angles::spans_in_day(engine, day, reference_jd, Divisions::TITHI)?;

    raw.into_iter()
        .map(|span| {
            let tithi = Tithi::from_index(span.index)?;
            Ok(TithiSpan {
                index: tithi.index(),
                number: tithi.number(),
                paksha: tithi.paksha(),
                name: tithi.name().to_string(),
                entry: span.entry.map(|jd| day.moment(jd)).transpose()?,
                exit: span.exit.map(|jd| day.moment(jd)).transpose()?,
                sunrises: count_sunrises(span.entry, span.exit, sunrises),
                prevailing: span.prevailing,
                source: span.source,
            })
        })
        .collect()
}

/// Sunrises inside `[entry, exit)`, or `None` where there is no count to give.
///
/// Zero and unknown are different answers and used to be the same one. Zero says
/// no civil day is named after this tithi, which is a kshaya; unknown says the
/// question could not be put - either a boundary did not resolve, or the Sun
/// rose on none of the three days the window is counted against, which is a fact
/// about the latitude. The day view captioned both as a kshaya, so inside a
/// polar night every tithi of every day claimed to be one while the grid cell
/// beside it, which falls back to local noon, was right.
fn count_sunrises(entry: Option<f64>, exit: Option<f64>, sunrises: &[f64]) -> Option<u8> {
    let (Some(entry), Some(exit)) = (entry, exit) else {
        return None;
    };
    if sunrises.is_empty() {
        return None;
    }
    Some(
        sunrises
            .iter()
            .filter(|&&jd| jd >= entry && jd < exit)
            .count()
            .min(u8::MAX as usize) as u8,
    )
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
