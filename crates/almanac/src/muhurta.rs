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
    /// Owned rather than borrowed: a day is cached, and the cache round-trips
    /// through `Deserialize`, which cannot produce a `&'static str`.
    pub name: String,
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
    &[(Half::Day, 14)],                  // Ravivara:    10h24m
    &[(Half::Day, 9), (Half::Day, 12)],  // Somavara:     6h24m, 8h48m
    &[(Half::Day, 4), (Half::Night, 8)], // Mangalavara: 2h24m; 5h36m after sunset
    &[(Half::Day, 8)],                   // Budhavara:    5h36m
    &[(Half::Day, 6), (Half::Day, 12)],  // Guruvara:     4h00m, 8h48m
    &[(Half::Day, 4), (Half::Day, 12)],  // Shukravara:   2h24m, 8h48m
    &[(Half::Day, 1), (Half::Day, 2)],   // Shanivara:    from sunrise, 1h36m
];

/// Which night a window divides.
///
/// A civil day touches two of them, and the rules do not agree on which they
/// mean. Brahma Muhurta ends shortly before sunrise, so "today's" is the one in
/// this morning's small hours - which is what a published panchanga prints, and
/// what someone planning to be up for it needs. Tuesday's second Durmuhurtam is
/// stated as an offset after sunset, so it is tonight's.
///
/// Not in the payload: `Half` is, and day-or-night is all the front end has to
/// draw. Which night a window belongs to is answered by the time it carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Night {
    /// Last night's sunset to this morning's sunrise.
    Before,
    /// This evening's sunset to tomorrow's sunrise.
    After,
}

/// The four instants bounding a civil day's light and the two nights it touches.
///
/// All four are required. A day missing any of them has no muhurtas rather than
/// muhurtas measured against a guess.
#[derive(Debug, Clone, Copy)]
pub struct Horizon {
    /// The previous evening's sunset, which opens the night this day dawns from.
    pub previous_sunset: f64,
    pub sunrise: f64,
    pub sunset: f64,
    /// The following day's sunrise, which closes the night this day ends in.
    pub next_sunrise: f64,
}

impl Horizon {
    /// Resolves the four instants, or `None` if any is missing.
    ///
    /// The neighbouring days are searched in their own windows rather than in
    /// this one: a night runs across midnight, and looking for either end of it
    /// inside today finds this day's own sunrise and sunset, which bound the
    /// daylight rather than the dark.
    pub fn of(engine: &Engine, day: &CivilDay, observer: Observer) -> Result<Option<Self>> {
        let today = engine.rise_set(day.start_jd, day.end_jd, Graha::Surya, observer)?;
        let yesterday_day = day.previous()?;
        let yesterday = engine.rise_set(
            yesterday_day.start_jd,
            yesterday_day.end_jd,
            Graha::Surya,
            observer,
        )?;
        let tomorrow_day = day.next()?;
        let tomorrow = engine.rise_set(
            tomorrow_day.start_jd,
            tomorrow_day.end_jd,
            Graha::Surya,
            observer,
        )?;

        let (Some(previous_sunset), Some(sunrise), Some(sunset), Some(next_sunrise)) =
            (yesterday.set, today.rise, today.set, tomorrow.rise)
        else {
            return Ok(None);
        };

        // Each span must run forwards. A sunset that does not precede the
        // sunrise it is paired with means the two belong to different daylight
        // periods, which happens either side of a polar day - there is nothing
        // to divide, so there are no muhurtas.
        if previous_sunset >= sunrise || sunset <= sunrise || next_sunrise <= sunset {
            return Ok(None);
        }

        Ok(Some(Self {
            previous_sunset,
            sunrise,
            sunset,
            next_sunrise,
        }))
    }

    fn bounds(&self, half: Half, night: Night) -> (f64, f64) {
        match (half, night) {
            (Half::Day, _) => (self.sunrise, self.sunset),
            (Half::Night, Night::Before) => (self.previous_sunset, self.sunrise),
            (Half::Night, Night::After) => (self.sunset, self.next_sunrise),
        }
    }

    /// The `index`th of `parts` equal divisions of a half, 1-based.
    fn part(&self, half: Half, night: Night, index: u8, parts: u8) -> (f64, f64) {
        let (start, end) = self.bounds(half, night);
        let width = (end - start) / parts as f64;
        let from = start + width * (index.saturating_sub(1) as f64);
        (from, from + width)
    }
}

/// A window before it is resolved against a horizon.
#[derive(Debug, Clone, Copy)]
struct Window {
    name: &'static str,
    half: Half,
    /// Ignored when `half` is `Day`.
    night: Night,
    /// 1-based, within `parts`.
    index: u8,
    parts: u8,
    inauspicious: bool,
}

impl Window {
    const fn day(name: &'static str, index: u8, parts: u8, inauspicious: bool) -> Self {
        Self {
            name,
            half: Half::Day,
            night: Night::After,
            index,
            parts,
            inauspicious,
        }
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

    let mut windows: Vec<Window> = vec![
        Window::day("Rahu Kaal", RAHU_KAAL[vara], EIGHTHS, true),
        Window::day("Yamaganda", YAMAGANDA[vara], EIGHTHS, true),
        Window::day("Gulika", GULIKA[vara], EIGHTHS, true),
        // The night before, not the night after. Brahma Muhurta ends shortly
        // before sunrise, so today's is the one in this morning's small hours -
        // which is what a published panchanga prints, and the only reading that
        // is any use to someone planning to be awake for it. Taken from the
        // night after, it printed at the foot of the day's list at 03:45 the
        // following morning, which reads as a sorting fault.
        Window {
            name: "Brahma Muhurta",
            half: Half::Night,
            night: Night::Before,
            index: BRAHMA,
            parts: PER_HALF,
            inauspicious: false,
        },
    ];

    // Abhijit and Wednesday's Durmuhurtam are the same slot, so on a Budhavara
    // the auspicious reading is the one that gives way. Printing both would put
    // two windows with opposite meanings on identical times.
    if vara != BUDHAVARA {
        windows.push(Window::day("Abhijit", ABHIJIT, PER_HALF, false));
    }

    // Tuesday's second window is stated as an offset after sunset, so it is
    // tonight's - the other night from Brahma Muhurta's, and deliberately so.
    for &(half, index) in DURMUHURTAM[vara] {
        windows.push(Window {
            name: "Durmuhurtam",
            half,
            night: Night::After,
            index,
            parts: PER_HALF,
            inauspicious: true,
        });
    }

    let mut muhurtas = Vec::with_capacity(windows.len());
    for window in windows {
        let (start, end) = horizon.part(window.half, window.night, window.index, window.parts);
        muhurtas.push(Muhurta {
            name: window.name.to_string(),
            half: window.half,
            start: day.moment(start)?,
            end: day.moment(end)?,
            inauspicious: window.inauspicious,
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
            assert_eq!(
                distinct.len(),
                3,
                "vara {vara} puts two windows in one eighth"
            );
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
        for (vara, windows) in DURMUHURTAM.iter().enumerate() {
            assert!(
                !windows.is_empty(),
                "every vara has at least one durmuhurtam"
            );
            for &(_, index) in *windows {
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
            previous_sunset: 99.75,
            sunrise: 100.25,
            sunset: 100.75,
            next_sunrise: 101.25,
        };

        for (half, night, parts) in [
            (Half::Day, Night::After, PER_HALF),
            (Half::Night, Night::Before, PER_HALF),
            (Half::Night, Night::After, EIGHTHS),
        ] {
            let (start, end) = horizon.bounds(half, night);
            let (first_from, _) = horizon.part(half, night, 1, parts);
            let (_, last_to) = horizon.part(half, night, parts, parts);
            assert!(
                (first_from - start).abs() < 1e-9,
                "the first part opens the half"
            );
            assert!(
                (last_to - end).abs() < 1e-9,
                "the last part closes the half"
            );

            for index in 2..=parts {
                let (_, previous_to) = horizon.part(half, night, index - 1, parts);
                let (from, _) = horizon.part(half, night, index, parts);
                assert!(
                    (from - previous_to).abs() < 1e-9,
                    "part {index} does not begin where {} ended",
                    index - 1
                );
            }
        }

        // The two nights are different spans, and Brahma Muhurta has to land in
        // the one that ends at this day's sunrise.
        let (brahma_from, brahma_to) = horizon.part(Half::Night, Night::Before, BRAHMA, PER_HALF);
        assert!(
            brahma_from > horizon.previous_sunset,
            "Brahma Muhurta begins after the previous sunset"
        );
        assert!(
            brahma_to < horizon.sunrise,
            "and ends before this day's sunrise, not at it"
        );
    }
}
