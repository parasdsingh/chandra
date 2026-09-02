//! The Lagna Kundali: where the nine grahas stand right now, and what is rising.
//!
//! Not a birth chart. There is no birth time and no birth place, and nothing
//! here is read against a natal position - calling it a kundali without the
//! `lagna` qualifier, or calling it gochara, would both claim a natal chart the
//! app does not have and will never ask for.
//!
//! One shape serves all three chart formats. North Indian, South Indian and East
//! Indian differ in where on screen a rashi is drawn and what is written in the
//! compartment; they do not differ in what is true. So this says which rashi
//! holds what, and which rashi is rising, and leaves the drawing to the front
//! end.

use chandra_ephemeris::{Engine, Graha, Observer, Source};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::events::combustion_from;
use crate::standing::{dignity_of, Dignity};
use crate::varga::varga_rashi;
pub use crate::varga::Varga;
use crate::zodiac::{degrees_in_rashi, Rashi};

/// The whole chart at one instant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chakra {
    pub unix_ms: i64,
    /// Which division this is. `D1` is the rashi chart.
    pub varga: Varga,
    /// The scheme the division was computed by.
    ///
    /// Always `Parashari`, and printed anyway (D-032). A D9 computed one way
    /// looks exactly like a D9 computed another - same twelve compartments, same
    /// glyphs - so a chart that cannot say what produced it invites the reader to
    /// assume it matches whatever they last saw elsewhere.
    pub scheme: &'static str,
    /// Where the chart was cast for.
    ///
    /// Not decoration. The lagna moves a degree every four minutes, so a
    /// longitude three hundred kilometres out moves it about three degrees - and
    /// a chart computed for the wrong city looks exactly like one computed for
    /// the right one. Both Drik Panchang and Jagannatha Hora print the place
    /// beside every chart for this reason.
    pub place: String,
    /// The rising point. Every format needs it: South and East Indian mark its
    /// cell, and North Indian is built on it - house 1 *is* the lagna's rashi,
    /// so without this there is no North Indian chart at all.
    pub lagna: Lagna,
    /// Twelve, always, in zodiacal order from Mesha. A rashi with nothing in it
    /// is present and empty rather than absent: the chart draws twelve
    /// compartments whatever stands in them.
    pub rashis: Vec<ChakraRashi>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lagna {
    pub rashi: Rashi,
    pub name: String,
    /// Sidereal longitude, 0 to 360.
    pub longitude: f64,
    /// The same, as degrees, arcminutes and arcseconds within the rashi. How far
    /// into the sign it has risen is the part that goes stale fastest - the
    /// ascendant moves about a degree every four minutes.
    pub degrees_in_rashi: (u32, u32, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChakraRashi {
    pub rashi: Rashi,
    pub name: String,
    /// Three letters, for a compartment that has no room for `Vrishchika`.
    pub short: String,
    /// Counted from the lagna's rashi, 1 to 12, whole sign.
    ///
    /// Carried rather than derived in the front end because the count is the one
    /// piece of jyotisha in the chart, and `standing.rs` already owns that
    /// arithmetic. Two implementations of "which house is this" would be two
    /// chances to be off by one.
    pub house: u8,
    pub grahas: Vec<ChakraGraha>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChakraGraha {
    pub graha: Graha,
    /// Two Latin letters: `Su`, `Mo`, `Ma`, `Me`, `Ju`, `Ve`, `Sa`, `Ra`, `Ke`,
    /// which is the published chart convention. Sanskrit does not abbreviate to
    /// two - Shukra and Shani are both `Sh`.
    ///
    /// Owned, like every other string on a payload: a chart is serialised and
    /// `Deserialize` cannot produce a `&'static str`.
    pub short: String,
    pub longitude: f64,
    pub degrees_in_rashi: (u32, u32, f64),
    /// Marked on the chart by writing the name in brackets.
    pub retrograde: bool,
    /// Inside the Sun's rays. Marked by the same warm wash the calendar cell
    /// uses, so one state has one appearance wherever it appears.
    pub combust: bool,
    /// Exalted, debilitated, or in a sign it rules. `None` where the graha has
    /// no relationship to the sign it stands in, and always for the nodes, which
    /// have no agreed dignity table (D-026).
    pub dignity: Option<Dignity>,
    /// The full name, for the hover. An abbreviation is what the compartment has
    /// room for; it is not what anyone should have to decode.
    pub name: String,
}

/// Two-letter chart forms, the published convention.
///
/// `Ma` and `Me` are the one pair at risk of being read for each other. The
/// spoken label never carries an abbreviation and says `Mangala` and `Budha` in
/// full, which is the same mitigation every other abbreviation in the app uses.
const fn chart_short(graha: Graha) -> &'static str {
    match graha {
        Graha::Surya => "Su",
        Graha::Chandra => "Mo",
        Graha::Mangala => "Ma",
        Graha::Budha => "Me",
        Graha::Guru => "Ju",
        Graha::Shukra => "Ve",
        Graha::Shani => "Sa",
        Graha::Rahu => "Ra",
        Graha::Ketu => "Ke",
    }
}

/// The chart at an instant, for an observer.
///
/// The observer matters here in a way it does not for a position: two people
/// reading the same minute in different cities have the same grahas in the same
/// rashis and a different lagna.
pub fn at(
    engine: &Engine,
    unix_ms: i64,
    observer: Observer,
    place: &str,
    varga: Varga,
) -> Result<Chakra> {
    let jd = chandra_ephemeris::unix_seconds_to_jd(unix_ms as f64 / 1000.0);

    // Positions first, then the ascendant, and the order matters less than the
    // fact that both are read before anything is built from either.
    //
    // These take the engine lock separately, so a reconfigure can land between
    // them and give a chart whose lagna is on one ayanamsa and whose grahas are
    // on another - a whole rashi apart between Lahiri and Raman. `Almanac`
    // watches the cache generation across this call and recomputes if it moved,
    // which is what makes the pair atomic; nothing here can do it alone.
    let positions = engine.positions(jd, &Graha::ALL)?;
    let ascendant = engine.ascendant(jd, observer)?;

    // The varga lagna is the ascendant's own longitude put through the same rule
    // as a graha's. Cited five ways in `docs/design/vargas.md` §5 rather than
    // assumed - one implementation instead divides each house midpoint, which
    // gives divisional houses that need not be twelve consecutive signs, and is
    // a different model rather than a different formula.
    let lagna_rashi = varga_rashi(varga, ascendant.degrees);

    let mut rashis: Vec<ChakraRashi> = Rashi::ALL
        .into_iter()
        .map(|rashi| ChakraRashi {
            rashi,
            name: rashi.name().to_string(),
            short: rashi.short().to_string(),
            // Inclusive from the lagna, so the lagna's own rashi is house 1.
            house: ((rashi.index() + 12 - lagna_rashi.index()) % 12) as u8 + 1,
            grahas: Vec::new(),
        })
        .collect();

    // The Sun's longitude, for combustion. Read from the same nine positions
    // rather than asked for again, so every graha's separation is measured
    // against the Sun at the instant its own position was taken.
    let sun = positions[Graha::ALL
        .iter()
        .position(|&g| g == Graha::Surya)
        .expect("Graha::ALL contains Surya")]
    .longitude;

    for (graha, position) in Graha::ALL.into_iter().zip(positions.iter()) {
        // The rule applies to the nodes exactly as it does to everything else.
        // There is no classical text that treats them separately in a varga and
        // no implementation that branches on them, so there is no branch here.
        let rashi = varga_rashi(varga, position.longitude);
        rashis[rashi.index()].grahas.push(ChakraGraha {
            graha,
            short: chart_short(graha).to_string(),
            name: graha.name().to_string(),
            longitude: position.longitude,
            // The body's true position in its *rashi*, in every chart. A degree
            // within a varga part would have to be stretched back across thirty
            // degrees to be printable, and how to do that is disputed - two
            // schemes in one application alone. The honest figure is the one
            // actually computed, and it is the same number in every division.
            degrees_in_rashi: degrees_in_rashi(position.longitude),
            retrograde: position.is_retrograde(),
            combust: combustion_from(graha, position, sun).combust,
            // Read in the sign the body occupies *in this chart*. Dignity is a
            // property of a graha in a sign, and in a varga the graha is in the
            // varga's sign - which is what "exalted in navamsa" means. Retrograde
            // and combustion are properties of the body rather than of a sign, so
            // they do not change between divisions.
            dignity: dignity_of(graha, rashi),
        });
    }

    // Within a compartment, in the order they have travelled into it. A chart
    // that listed them in `Graha::ALL` order would put Surya above Chandra
    // whichever was further through the sign, which is not what a reader
    // checking a conjunction wants to see.
    for rashi in &mut rashis {
        rashi
            .grahas
            .sort_by(|a, b| a.longitude.total_cmp(&b.longitude));
    }

    Ok(Chakra {
        unix_ms,
        varga,
        scheme: "Parashari",
        place: place.to_string(),
        lagna: Lagna {
            rashi: lagna_rashi,
            name: lagna_rashi.name().to_string(),
            longitude: ascendant.degrees,
            degrees_in_rashi: degrees_in_rashi(ascendant.degrees),
        },
        rashis,
        source: Source::weakest(
            [ascendant.source]
                .into_iter()
                .chain(positions.iter().map(|p| p.source)),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Houses count inclusively from the lagna, so the lagna's own rashi is the
    /// first and the twelfth is the one behind it.
    #[test]
    fn the_lagna_rashi_is_the_first_house() {
        for lagna in Rashi::ALL {
            let houses: Vec<u8> = Rashi::ALL
                .into_iter()
                .map(|rashi| ((rashi.index() + 12 - lagna.index()) % 12) as u8 + 1)
                .collect();

            assert_eq!(
                houses[lagna.index()],
                1,
                "{lagna:?} rising must be the first house"
            );

            let mut seen: Vec<u8> = houses.clone();
            seen.sort_unstable();
            assert_eq!(
                seen,
                (1..=12).collect::<Vec<u8>>(),
                "{lagna:?} rising does not produce each house exactly once"
            );

            // The seventh from the lagna is the sign opposite it, which is the
            // check a reader would make first.
            let opposite = Rashi::ALL[(lagna.index() + 6) % 12];
            assert_eq!(houses[opposite.index()], 7);
        }
    }

    /// Nine distinct two-letter forms. `Shukra` and `Shani` are why these are
    /// Latin rather than Sanskrit, so a collision here would defeat the reason
    /// they exist.
    #[test]
    fn every_graha_has_its_own_two_letter_form() {
        let forms: std::collections::BTreeSet<&str> =
            Graha::ALL.iter().map(|&g| chart_short(g)).collect();
        assert_eq!(forms.len(), 9, "two grahas share a chart abbreviation");
        for graha in Graha::ALL {
            assert_eq!(chart_short(graha).chars().count(), 2, "{graha:?}");
        }
    }
}
