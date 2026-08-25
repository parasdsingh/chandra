//! Yoga and karana: the two limbs of a panchanga that are named from a
//! Sun-Moon angle but not from the tithi.
//!
//! Both are found by `angles`, which is the same routine the tithi uses. What is
//! here is the naming, which is the part that differs: a yoga's name is a
//! straight table of 27, and a karana's is a cycle of eleven laid over sixty
//! that does not repeat evenly.

use chandra_ephemeris::Engine;
use chandra_ephemeris::Source;
use serde::{Deserialize, Serialize};

use crate::angles::{self, Divisions, RawSpan};
use crate::error::{Error, Result};
use crate::time::{CivilDay, Moment};

/// The 27 yogas, in order from the zero of the Sun-Moon sum.
const YOGAS: [&str; 27] = [
    "Vishkambha",
    "Priti",
    "Ayushman",
    "Saubhagya",
    "Shobhana",
    "Atiganda",
    "Sukarma",
    "Dhriti",
    "Shula",
    "Ganda",
    "Vriddhi",
    "Dhruva",
    "Vyaghata",
    "Harshana",
    "Vajra",
    "Siddhi",
    "Vyatipata",
    "Variyana",
    "Parigha",
    "Shiva",
    "Siddha",
    "Sadhya",
    "Shubha",
    "Shukla",
    "Brahma",
    "Indra",
    "Vaidhriti",
];

/// The seven movable karanas, which repeat.
const MOVABLE: [&str; 7] = [
    "Bava", "Balava", "Kaulava", "Taitila", "Gara", "Vanija", "Vishti",
];

/// A yoga touching one civil day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YogaSpan {
    /// 1 to 27.
    pub index: u8,
    pub name: String,
    /// The true crossing instants, never clamped to the day. `None` only when
    /// refinement did not converge, in which case no time is guessed.
    pub entry: Option<Moment>,
    pub exit: Option<Moment>,
    /// In force at the day's reference instant. Exactly one span per day.
    pub prevailing: bool,
    pub source: Source,
}

/// A karana touching one civil day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KaranaSpan {
    /// 1 to 60, counted from the start of the lunar month.
    pub index: u8,
    pub name: String,
    /// Whether this is one of the four that occur once a month rather than one
    /// of the seven that repeat. Carried rather than derived from the name,
    /// because the front end should not have to know the four by heart.
    pub fixed: bool,
    pub entry: Option<Moment>,
    pub exit: Option<Moment>,
    pub prevailing: bool,
    pub source: Source,
}

/// The name of the karana at a 1-based index, and whether it is a fixed one.
///
/// Sixty karanas hold eleven names, and the layout is not a repeat of eleven.
/// One fixed karana opens the month, then the seven movable ones run eight times
/// over - 56 of them - and the remaining three fixed ones close it:
///
/// ```text
///  1        Kimstughna                     fixed
///  2 .. 57  Bava .. Vishti, eight times    movable
/// 58        Shakuni                        fixed
/// 59        Chatushpada                    fixed
/// 60        Naga                           fixed
/// ```
///
/// 1 + 56 + 3 is 60 and 56 is 8 x 7, which is the arithmetic that has to hold
/// for the scheme to close; the test below asserts it rather than trusting it.
fn karana_name(index: u8) -> Result<(&'static str, bool)> {
    match index {
        1 => Ok(("Kimstughna", true)),
        2..=57 => Ok((MOVABLE[((index - 2) % 7) as usize], false)),
        58 => Ok(("Shakuni", true)),
        59 => Ok(("Chatushpada", true)),
        60 => Ok(("Naga", true)),
        other => Err(Error::TimeZone(format!(
            "karana index {other} out of range"
        ))),
    }
}

/// The name of the yoga at a 1-based index.
///
/// `checked_sub` rather than `index - 1`: the index comes from an angle, and an
/// angle that failed to produce one would arrive as zero and panic on the
/// subtraction rather than reporting a bad index.
fn yoga_name(index: u8) -> Result<&'static str> {
    index
        .checked_sub(1)
        .and_then(|at| YOGAS.get(at as usize))
        .copied()
        .ok_or_else(|| Error::TimeZone(format!("yoga index {index} out of range")))
}

/// Every yoga touching `day`, in order.
pub fn yogas_in_day(engine: &Engine, day: &CivilDay, reference_jd: f64) -> Result<Vec<YogaSpan>> {
    angles::spans_in_day(engine, day, reference_jd, Divisions::YOGA)?
        .into_iter()
        .map(|span| {
            Ok(YogaSpan {
                index: span.index,
                name: yoga_name(span.index)?.to_string(),
                entry: moment(day, span.entry)?,
                exit: moment(day, span.exit)?,
                prevailing: span.prevailing,
                source: span.source,
            })
        })
        .collect()
}

/// Every karana touching `day`, in order.
///
/// A karana is half a tithi, so a day holds two of them and sometimes three.
pub fn karanas_in_day(
    engine: &Engine,
    day: &CivilDay,
    reference_jd: f64,
) -> Result<Vec<KaranaSpan>> {
    angles::spans_in_day(engine, day, reference_jd, Divisions::KARANA)?
        .into_iter()
        .map(|span: RawSpan| {
            let (name, fixed) = karana_name(span.index)?;
            Ok(KaranaSpan {
                index: span.index,
                name: name.to_string(),
                fixed,
                entry: moment(day, span.entry)?,
                exit: moment(day, span.exit)?,
                prevailing: span.prevailing,
                source: span.source,
            })
        })
        .collect()
}

fn moment(day: &CivilDay, jd: Option<f64>) -> Result<Option<Moment>> {
    jd.map(|jd| day.moment(jd)).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_karana_cycle_closes_on_sixty() {
        let mut fixed = 0;
        let mut movable = 0;
        for index in 1..=60u8 {
            let (name, is_fixed) = karana_name(index).expect("in range");
            assert!(!name.is_empty());
            if is_fixed {
                fixed += 1;
            } else {
                movable += 1;
            }
        }
        assert_eq!(fixed, 4, "four karanas occur once a month");
        assert_eq!(movable, 56, "the seven movable ones run eight times over");
        assert_eq!(
            movable % MOVABLE.len(),
            0,
            "the movable run must be whole cycles"
        );

        assert!(karana_name(0).is_err());
        assert!(karana_name(61).is_err());
    }

    /// The four fixed karanas each occur once, and the seven movable ones each
    /// occur exactly eight times. A misplaced boundary in the match above would
    /// leave one name short and another long while still totalling sixty.
    #[test]
    fn every_karana_name_occurs_the_number_of_times_it_should() {
        let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
        for index in 1..=60u8 {
            *counts.entry(karana_name(index).unwrap().0).or_default() += 1;
        }
        assert_eq!(counts.len(), 11, "eleven names over sixty karanas");
        for name in MOVABLE {
            assert_eq!(
                counts[name], 8,
                "{name} is movable and should occur eight times"
            );
        }
        for name in ["Kimstughna", "Shakuni", "Chatushpada", "Naga"] {
            assert_eq!(counts[name], 1, "{name} is fixed and should occur once");
        }
    }

    /// Vishti, the seventh movable karana, is the one a panchanga marks.
    /// Its position in the cycle is what a reader would check first.
    #[test]
    fn vishti_falls_where_the_cycle_puts_it() {
        // Karana 2 opens the movable run at Bava, so Vishti is karana 8.
        assert_eq!(karana_name(2).unwrap().0, "Bava");
        assert_eq!(karana_name(8).unwrap().0, "Vishti");
        assert_eq!(karana_name(9).unwrap().0, "Bava");
        // Eight cycles later the run ends on Vishti, and the fixed ones follow.
        assert_eq!(karana_name(57).unwrap().0, "Vishti");
        assert_eq!(karana_name(58).unwrap().0, "Shakuni");
    }

    #[test]
    fn every_yoga_index_has_a_name() {
        for index in 1..=27u8 {
            assert!(!yoga_name(index).expect("in range").is_empty());
        }
        assert!(yoga_name(0).is_err());
        assert!(yoga_name(28).is_err());
        assert_eq!(yoga_name(1).unwrap(), "Vishkambha");
        assert_eq!(yoga_name(27).unwrap(), "Vaidhriti");
    }
}
