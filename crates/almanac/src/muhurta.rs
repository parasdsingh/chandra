//! Muhurtas: the named windows a day is divided into.
//!
//! Every window here is a fraction of daylight or of night, never a fraction of
//! a clock. A day is divided into fifteen equal muhurtas from sunrise to sunset
//! and the night into fifteen more from sunset to the next sunrise, so a muhurta
//! is 48 minutes only where the day happens to be twelve hours long. Published
//! tables that state these as clock offsets - "10h24m after sunrise" - assume
//! that twelve-hour day; the offsets are converted to indices here so the
//! windows stay correct at a latitude where they are not.
//!
//! Where the Sun does not rise or set, there are no muhurtas. A window defined
//! as a fraction of daylight has no meaning on a day with no daylight, and
//! computing one against a substitute instant would be inventing a boundary the
//! definition does not have.
//!
//! Sources for the tables are in `docs/design/panchanga.md` §7.

use chandra_ephemeris::{Engine, Graha, Observer};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::time::{CivilDay, Moment};

/// Muhurtas in the day, and in the night. Fifteen each, so thirty in a whole
/// day-and-night, which is the classical division.
const PER_HALF: u8 = 15;

/// Parts the day is divided into for the three vara-chosen windows.
///
/// Rahu Kaal, Yamaganda and Gulika divide daylight into eight rather than into
/// fifteen. They are a different scheme that happens to live in the same field,
/// not muhurtas that have been rounded.
const EIGHTHS: u8 = 8;

/// Which half of the day-and-night a window divides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Half {
    /// Sunrise to sunset.
    Day,
    /// Sunset to the next sunrise.
    Night,
}

/// A named window of a day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Muhurta {
    pub name: &'static str,
    pub half: Half,
    pub start: Moment,
    pub end: Moment,
    /// Whether the window is one to avoid. Every window here is except Abhijit
    /// and Brahma Muhurta, and a reader should not have to know which by name.
    pub inauspicious: bool,
}

/// Which eighth of the day each vara-chosen window falls in, indexed from
/// Ravivara. 1-based, counting from sunrise.
///
/// These three tables are settled and not in dispute between panchangas.
const RAHU_KAAL: [u8; 7] = [8, 2, 7, 5, 6, 4, 3];
const YAMAGANDA: [u8; 7] = [5, 4, 3, 2, 1, 7, 6];
const GULIKA: [u8; 7] = [7, 6, 5, 4, 3, 2, 1];

/// Abhijit is the eighth of the fifteen day muhurtas: the one straddling
/// midday.
const ABHIJIT: u8 = 8;

/// Wednesday. Abhijit is held not to apply on a Budhavara, and the reason is
/// visible in the table below: Wednesday's Durmuhurtam is the eighth day
/// muhurta, which is Abhijit's own slot. The two rules are one observation.
const BUDHAVARA: usize = 3;

/// Brahma Muhurta is the fourteenth of the fifteen night muhurtas - the one
/// ending 48 minutes of night before sunrise.
const BRAHMA: u8 = 14;

/// Durmuhurtam windows by vara, as (half, 1-based muhurta of that half).
///
/// Converted from the published clock offsets: the source states each window as
/// hours and minutes after sunrise on a twelve-hour day, where one muhurta is 48
/// minutes, so an offset divided by 48 minutes is the index. Saturday's is
/// stated as a single window lasting 1h36m, which is two muhurtas.
const DURMUHURTAM: [&[(Half, u8)]; 7] = [
    &[(Half::Day, 14)],                 // Ravivara:    10h24m
    &[(Half::Day, 9), (Half::Day, 12)], // Somavara:     6h24m, 8h48m
    &[(Half::Day, 4), (Half::Night, 8)], // Mangalavara: 2h24m; 5h36m after sunset
    &[(Half::Day, 8)],                  // Budhavara:    5h36m
    &[(Half::Day, 6), (Half::Day, 12)], // Guruvara:     4h00m, 8h48m
    &[(Half::Day, 4), (Half::Day, 12)], // Shukravara:   2h24m, 8h48m
    &[(Half::Day, 1), (Half::Day, 2)],  // Shanivara:    from sunrise, 1h36m
];

/// Sunrise and sunset bounding one civil day's light and the night after it.
///
/// All three instants are required. A day missing any of them has no muhurtas
/// rather than muhurtas measured against a guess.
#[derive(Debug, Clone, Copy)]
pub struct Horizon {
    pub sunrise: f64,
    pub sunset: f64,
    /// The following day's sunrise, which closes the night.
    pub next_sunrise: f64,
}

impl Horizon {
    /// Resolves the three instants, or `None` if any is missing.
    ///
    /// The next sunrise is searched in the following civil day rather than the
    /// same one: the night runs past midnight, and looking for it inside today
    /// would find this morning's sunrise, which has already gone.
    pub fn of(engine: &Engine, day: &CivilDay, observer: Observer) -> Result<Option<Self>> {
        let today = engine.rise_set(day.start_jd, day.end_jd, Graha::Surya, observer)?;
        let tomorrow_day = day.next()?;
        let tomorrow = engine.rise_set(
            tomorrow_day.start_jd,
            tomorrow_day.end_jd,
            Graha::Surya,
            observer,
        )?;

        let (Some(sunrise), Some(sunset), Some(next_sunrise)) =
            (today.rise, today.set, tomorrow.rise)
        else {
            return Ok(None);
        };

        // A sunset before the sunrise it is paired with means the two belong to
        // different daylight periods, which happens either side of a polar day.
        // There is no daylight span to divide, so there are no muhurtas.
        if sunset <= sunrise || next_sunrise <= sunset {
            return Ok(None);
        }

        Ok(Some(Self {
            sunrise,
            sunset,
            next_sunrise,
        }))
    }

    fn bounds(&self, half: Half) -> (f64, f64) {
        match half {
            Half::Day => (self.sunrise, self.sunset),
            Half::Night => (self.sunset, self.next_sunrise),
        }
    }

    /// The `index`th of `parts` equal divisions of `half`, 1-based.
    fn part(&self, half: Half, index: u8, parts: u8) -> (f64, f64) {
        let (start, end) = self.bounds(half);
        let width = (end - start) / parts as f64;
        let from = start + width * (index.saturating_sub(1) as f64);
        (from, from + width)
    }
}

/// Every muhurta of `day`, in the order they begin.
///
/// Empty where the Sun does not rise and set, which is a fact about the latitude
/// rather than a failure.
pub fn muhurtas_in_day(
    engine: &Engine,
    day: &CivilDay,
    observer: Observer,
    vara: u8,
) -> Result<Vec<Muhurta>> {
    let Some(horizon) = Horizon::of(engine, day, observer)? else {
        return Ok(Vec::new());
    };
    let vara = (vara % 7) as usize;

    let mut windows: Vec<(&'static str, Half, u8, u8, bool)> = vec![
        ("Rahu Kaal", Half::Day, RAHU_KAAL[vara], EIGHTHS, true),
        ("Yamaganda", Half::Day, YAMAGANDA[vara], EIGHTHS, true),
        ("Gulika", Half::Day, GULIKA[vara], EIGHTHS, true),
        ("Brahma Muhurta", Half::Night, BRAHMA, PER_HALF, false),
    ];

    // Abhijit and Wednesday's Durmuhurtam are the same slot, so on a Budhavara
    // the auspicious reading is the one that gives way. Printing both would put
    // two windows with opposite meanings on identical times.
    if vara != BUDHAVARA {
        windows.push(("Abhijit", Half::Day, ABHIJIT, PER_HALF, false));
    }

    for &(half, index) in DURMUHURTAM[vara] {
        windows.push(("Durmuhurtam", half, index, PER_HALF, true));
    }

    let mut muhurtas = Vec::with_capacity(windows.len());
    for (name, half, index, parts, auspicious) in windows {
        let (start, end) = horizon.part(half, index, parts);
        muhurtas.push(Muhurta {
            name,
            half,
            start: day.moment(start)?,
            end: day.moment(end)?,
            inauspicious: auspicious,
        });
    }

    muhurtas.sort_by(|a, b| a.start.unix_ms.cmp(&b.start.unix_ms));
    Ok(muhurtas)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each of the three eighth-day windows falls in a different eighth on every
    /// vara. Two of them sharing a slot would mean one table is wrong.
    #[test]
    fn the_three_vara_windows_never_collide() {
        for vara in 0..7usize {
            let slots = [RAHU_KAAL[vara], YAMAGANDA[vara], GULIKA[vara]];
            assert!(
                slots.iter().all(|&slot| (1..=EIGHTHS).contains(&slot)),
                "vara {vara} has a slot outside the eight parts of the day"
            );
            let distinct: std::collections::BTreeSet<u8> = slots.into_iter().collect();
            assert_eq!(distinct.len(), 3, "vara {vara} puts two windows in one eighth");
        }
    }

    /// Gulika runs backwards from the seventh eighth on Sunday to the first on
    /// Saturday, one step a day. A transcription slip in the middle of the table
    /// is invisible to a range check and obvious to this one.
    #[test]
    fn gulika_steps_back_one_eighth_a_day() {
        for vara in 1..7usize {
            assert_eq!(
                GULIKA[vara],
                GULIKA[vara - 1] - 1,
                "gulika does not step back between vara {} and {vara}",
                vara - 1
            );
        }
    }

    #[test]
    fn every_durmuhurtam_window_is_inside_its_half() {
        for vara in 0..7usize {
            let windows = DURMUHURTAM[vara];
            assert!(
                !windows.is_empty(),
                "every vara has at least one durmuhurtam"
            );
            for &(_, index) in windows {
                assert!(
                    (1..=PER_HALF).contains(&index),
                    "vara {vara} has a durmuhurtam outside the fifteen muhurtas"
                );
            }
        }
    }

    /// The reason Abhijit does not apply on a Wednesday: the slot is already
    /// taken. If the Durmuhurtam table were ever edited so this stopped holding,
    /// the suppression above would be arbitrary rather than derived.
    #[test]
    fn wednesdays_durmuhurtam_is_abhijits_own_slot() {
        assert_eq!(DURMUHURTAM[BUDHAVARA], &[(Half::Day, ABHIJIT)]);
    }

    /// Equal parts, covering the half exactly, in order and without gaps.
    #[test]
    fn the_parts_of_a_half_tile_it() {
        let horizon = Horizon {
            sunrise: 100.25,
            sunset: 100.75,
            next_sunrise: 101.25,
        };

        for (half, parts) in [(Half::Day, PER_HALF), (Half::Night, EIGHTHS)] {
            let (start, end) = horizon.bounds(half);
            let (first_from, _) = horizon.part(half, 1, parts);
            let (_, last_to) = horizon.part(half, parts, parts);
            assert!((first_from - start).abs() < 1e-9, "the first part opens the half");
            assert!((last_to - end).abs() < 1e-9, "the last part closes the half");

            for index in 2..=parts {
                let (_, previous_to) = horizon.part(half, index - 1, parts);
                let (from, _) = horizon.part(half, index, parts);
                assert!(
                    (from - previous_to).abs() < 1e-9,
                    "part {index} does not begin where {} ended",
                    index - 1
                );
            }
        }
    }
}
