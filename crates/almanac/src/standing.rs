//! Where a graha stands in relation to the others, at one instant.
//!
//! Dignity, drishti and planetary war all read the same nine longitudes, so all
//! three are computed from one positions call. The nakshatra lord joins them
//! because it answers the same kind of question - whose ground the graha is
//! standing on - even though it costs nothing to find.
//!
//! Everything here is a classical rule applied to a computed position. Where the
//! rule itself is disputed, the disagreement is named rather than resolved: this
//! app reports what a graha is doing, not what it means.

use chandra_ephemeris::{Engine, Graha};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::zodiac::{Nakshatra, Rashi};

/// A graha's relationship to the rashi it occupies.
///
/// Moolatrikona is deliberately absent. Its degree ranges differ between
/// authorities, and unlike exaltation and own sign it cannot be stated without
/// choosing one - so it would be this app asserting an interpretation. It was
/// also not among the states asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dignity {
    Exalted,
    Debilitated,
    /// In a rashi it rules. Two for every graha except the luminaries: Surya
    /// rules Simha alone and Chandra rules Karka alone.
    OwnSign,
}

impl Dignity {
    pub const fn label(self) -> &'static str {
        match self {
            Dignity::Exalted => "Exalted",
            Dignity::Debilitated => "Debilitated",
            Dignity::OwnSign => "Own sign",
        }
    }
}

/// Two grahas within a degree of each other.
///
/// The winner is not reported. Which graha wins a graha yuddha is decided
/// differently by different authorities - by celestial latitude, by brightness,
/// by whose longitude is greater - and printing one would be this app asserting
/// an interpretation over the observation. The pairing and the separation are
/// the observation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct War {
    pub with: Graha,
    /// Angular separation in degrees, always positive.
    pub separation: f64,
}

/// How a graha stands, at the day's reference instant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Standing {
    /// `None` for Rahu and Ketu, which have no agreed dignity table.
    pub dignity: Option<Dignity>,
    /// The lord of the nakshatra the graha occupies - whose ground it stands on.
    pub nakshatra_lord: Graha,
    /// Grahas this one casts a full drishti on.
    pub aspects: Vec<Graha>,
    /// Grahas casting a full drishti on this one.
    pub aspected_by: Vec<Graha>,
    /// `None` unless this graha is one of the five that can be at war, and is.
    pub war: Option<War>,
}

/// Separation within which two grahas are at war, in degrees.
const WAR_ORB: f64 = 1.0;

/// The five tara grahas. A graha yuddha is between two of these and no others:
/// the Sun and Moon are luminaries rather than stars, and the nodes are points
/// with nothing to occlude.
const AT_WAR: [Graha; 5] = [
    Graha::Mangala,
    Graha::Budha,
    Graha::Guru,
    Graha::Shukra,
    Graha::Shani,
];

/// The houses a graha casts a full drishti on, counted inclusively from the one
/// it occupies - so 1 would be itself and every graha has 7.
///
/// Whole-sign, because that is what the classical rule is: a graha aspects a
/// house, and whatever stands in it.
///
/// Rahu and Ketu are given 5, 7 and 9. Their drishti is not universally agreed -
/// some authorities give them none at all - and this is the common reading. The
/// disagreement is recorded here rather than resolved.
const fn drishti(graha: Graha) -> &'static [u8] {
    match graha {
        Graha::Mangala => &[4, 7, 8],
        Graha::Guru => &[5, 7, 9],
        Graha::Shani => &[3, 7, 10],
        Graha::Rahu | Graha::Ketu => &[5, 7, 9],
        _ => &[7],
    }
}

/// Exaltation rashi, debilitation rashi, and the rashis a graha rules.
///
/// `None` for the nodes. Rahu and Ketu are shadow points, and every dignity
/// assigned to them - exalted in Vrishabha, or in Mithuna, or nowhere - is one
/// authority's reading rather than a settled table.
const fn dignities(graha: Graha) -> Option<(Rashi, Rashi, &'static [Rashi])> {
    match graha {
        Graha::Surya => Some((Rashi::Mesha, Rashi::Tula, &[Rashi::Simha])),
        Graha::Chandra => Some((Rashi::Vrishabha, Rashi::Vrishchika, &[Rashi::Karka])),
        Graha::Mangala => Some((
            Rashi::Makara,
            Rashi::Karka,
            &[Rashi::Mesha, Rashi::Vrishchika],
        )),
        Graha::Budha => Some((Rashi::Kanya, Rashi::Meena, &[Rashi::Mithuna, Rashi::Kanya])),
        Graha::Guru => Some((Rashi::Karka, Rashi::Makara, &[Rashi::Dhanu, Rashi::Meena])),
        Graha::Shukra => Some((Rashi::Meena, Rashi::Kanya, &[Rashi::Vrishabha, Rashi::Tula])),
        Graha::Shani => Some((Rashi::Tula, Rashi::Mesha, &[Rashi::Makara, Rashi::Kumbha])),
        Graha::Rahu | Graha::Ketu => None,
    }
}

/// The dignity of a graha standing in a rashi.
///
/// Exaltation is read at the sign, not at the degree. The exact degree is the
/// point of *deep* exaltation; a graha anywhere in the sign is exalted, which is
/// the state being reported.
pub fn dignity_of(graha: Graha, rashi: Rashi) -> Option<Dignity> {
    let (exalted, debilitated, own) = dignities(graha)?;
    if rashi == exalted {
        Some(Dignity::Exalted)
    } else if rashi == debilitated {
        Some(Dignity::Debilitated)
    } else if own.contains(&rashi) {
        Some(Dignity::OwnSign)
    } else {
        None
    }
}

/// Houses from `from` to `to`, counted inclusively - 1 for the same rashi.
/// Whole-sign house count, inclusive: `from` to `from` is house 1.
///
/// `pub(crate)` because `chakra.rs` needs the same answer. It said so - "two
/// implementations of 'which house is this' would be two chances to be off by
/// one" - and then wrote the expression out again, and the test wrote it a
/// third time.
pub(crate) fn houses_between(from: Rashi, to: Rashi) -> u8 {
    ((to.index() + 12 - from.index()) % 12) as u8 + 1
}

/// Whether `caster` standing in one rashi casts a full drishti on another.
pub fn aspects_rashi(caster: Graha, from: Rashi, to: Rashi) -> bool {
    // A graha does not aspect its own house, which the 7th-from-itself rule
    // never produces anyway; the guard is for the 1 that inclusive counting
    // gives for the same sign.
    let house = houses_between(from, to);
    house != 1 && drishti(caster).contains(&house)
}

/// How `graha` stands among the nine, at `jd`.
pub fn at(engine: &Engine, graha: Graha, jd: f64) -> Result<Standing> {
    // One call for all nine. Dignity needs one longitude, drishti needs every
    // other, and war needs four - asking three times would triple the cost of
    // the most expensive thing a day view does.
    let positions = engine.positions(jd, &Graha::ALL)?;
    let longitude_of = |subject: Graha| {
        positions[Graha::ALL
            .iter()
            .position(|&g| g == subject)
            .expect("Graha::ALL is complete")]
        .longitude
    };

    let longitude = longitude_of(graha);
    let rashi = Rashi::from_longitude(longitude);

    let mut aspects = Vec::new();
    let mut aspected_by = Vec::new();
    for other in Graha::ALL {
        if other == graha {
            continue;
        }
        let their_rashi = Rashi::from_longitude(longitude_of(other));
        if aspects_rashi(graha, rashi, their_rashi) {
            aspects.push(other);
        }
        if aspects_rashi(other, their_rashi, rashi) {
            aspected_by.push(other);
        }
    }

    // The closest opponent, not the first found. Two grahas can both be inside
    // the orb, and the war is with the nearer of them.
    let war = AT_WAR.contains(&graha).then(|| {
        AT_WAR
            .iter()
            .filter(|&&other| other != graha)
            .map(|&other| War {
                with: other,
                separation: separation(longitude, longitude_of(other)),
            })
            .filter(|war| war.separation < WAR_ORB)
            .min_by(|a, b| a.separation.total_cmp(&b.separation))
    });

    Ok(Standing {
        dignity: dignity_of(graha, rashi),
        nakshatra_lord: Nakshatra::from_longitude(longitude).lord(),
        aspects,
        aspected_by,
        war: war.flatten(),
    })
}

/// Shortest angular distance between two longitudes, in degrees.
fn separation(a: f64, b: f64) -> f64 {
    let raw = (a - b).rem_euclid(360.0);
    raw.min(360.0 - raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exaltation and debilitation are always opposite signs. A transcription
    /// slip in the table would almost certainly break this.
    #[test]
    fn every_exaltation_faces_its_debilitation() {
        for graha in Graha::ALL {
            let Some((exalted, debilitated, own)) = dignities(graha) else {
                assert!(
                    matches!(graha, Graha::Rahu | Graha::Ketu),
                    "{graha:?} has no dignity table and is not a node"
                );
                continue;
            };
            assert_eq!(
                houses_between(exalted, debilitated),
                7,
                "{graha:?} is exalted in {exalted:?} and debilitated in {debilitated:?}, which are not opposite"
            );
            assert!(!own.is_empty(), "{graha:?} rules no rashi");
            assert!(
                !own.contains(&debilitated),
                "{graha:?} cannot rule the sign it is debilitated in"
            );

            // Budha is exalted in Kanya and also rules it - the one graha whose
            // exaltation falls in its own sign. Asserted rather than skipped, so
            // the exception is recorded and any other graha acquiring one fails.
            if own.contains(&exalted) {
                assert_eq!(
                    graha,
                    Graha::Budha,
                    "{graha:?} is exalted in a sign it rules, which only Budha is"
                );
                assert_eq!(exalted, Rashi::Kanya);
            }
        }
    }

    /// Every rashi is ruled, and the five tara grahas rule two each while the
    /// two luminaries rule one. That is twelve.
    #[test]
    fn the_twelve_rashis_are_ruled_exactly_once_each() {
        let mut ruled: Vec<Rashi> = Vec::new();
        for graha in Graha::ALL {
            if let Some((_, _, own)) = dignities(graha) {
                ruled.extend_from_slice(own);
            }
        }
        assert_eq!(ruled.len(), 12, "twelve rulerships over twelve rashis");
        for rashi in Rashi::ALL {
            assert_eq!(
                ruled.iter().filter(|&&r| r == rashi).count(),
                1,
                "{rashi:?} is not ruled exactly once"
            );
        }
    }

    #[test]
    fn every_graha_aspects_the_seventh_and_never_itself() {
        for graha in Graha::ALL {
            assert!(
                drishti(graha).contains(&7),
                "{graha:?} does not aspect the seventh"
            );
            for rashi in Rashi::ALL {
                assert!(
                    !aspects_rashi(graha, rashi, rashi),
                    "{graha:?} aspects its own rashi"
                );
                let opposite = Rashi::ALL[(rashi.index() + 6) % 12];
                assert!(
                    aspects_rashi(graha, rashi, opposite),
                    "{graha:?} in {rashi:?} does not aspect {opposite:?}"
                );
            }
        }
    }

    /// The special drishtis, checked as houses rather than as a table read back
    /// to itself: Mangala from Mesha reaches Karka, Tula and Vrishchika.
    #[test]
    fn the_special_drishtis_reach_the_houses_they_are_defined_as() {
        assert!(aspects_rashi(Graha::Mangala, Rashi::Mesha, Rashi::Karka)); // 4th
        assert!(aspects_rashi(
            Graha::Mangala,
            Rashi::Mesha,
            Rashi::Vrishchika
        )); // 8th
        assert!(!aspects_rashi(Graha::Mangala, Rashi::Mesha, Rashi::Simha)); // 5th

        assert!(aspects_rashi(Graha::Guru, Rashi::Mesha, Rashi::Simha)); // 5th
        assert!(aspects_rashi(Graha::Guru, Rashi::Mesha, Rashi::Dhanu)); // 9th
        assert!(!aspects_rashi(Graha::Guru, Rashi::Mesha, Rashi::Karka)); // 4th

        assert!(aspects_rashi(Graha::Shani, Rashi::Mesha, Rashi::Mithuna)); // 3rd
        assert!(aspects_rashi(Graha::Shani, Rashi::Mesha, Rashi::Makara)); // 10th
        assert!(!aspects_rashi(Graha::Shani, Rashi::Mesha, Rashi::Karka)); // 4th

        // Surya has the plain drishti and nothing else.
        assert!(!aspects_rashi(Graha::Surya, Rashi::Mesha, Rashi::Simha));
        assert!(aspects_rashi(Graha::Surya, Rashi::Mesha, Rashi::Tula));
    }

    #[test]
    fn separation_is_the_short_way_round() {
        assert!((separation(1.0, 359.0) - 2.0).abs() < 1e-9);
        assert!((separation(359.0, 1.0) - 2.0).abs() < 1e-9);
        assert!((separation(10.0, 10.5) - 0.5).abs() < 1e-9);
        assert!((separation(0.0, 180.0) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn only_the_five_tara_grahas_go_to_war() {
        assert!(!AT_WAR.contains(&Graha::Surya));
        assert!(!AT_WAR.contains(&Graha::Chandra));
        assert!(!AT_WAR.contains(&Graha::Rahu));
        assert!(!AT_WAR.contains(&Graha::Ketu));
        assert_eq!(AT_WAR.len(), 5);
    }

    #[test]
    fn dignity_reads_the_sign_not_the_degree() {
        assert_eq!(
            dignity_of(Graha::Surya, Rashi::Mesha),
            Some(Dignity::Exalted)
        );
        assert_eq!(
            dignity_of(Graha::Surya, Rashi::Tula),
            Some(Dignity::Debilitated)
        );
        assert_eq!(
            dignity_of(Graha::Surya, Rashi::Simha),
            Some(Dignity::OwnSign)
        );
        assert_eq!(dignity_of(Graha::Surya, Rashi::Karka), None);
        assert_eq!(dignity_of(Graha::Rahu, Rashi::Vrishabha), None);

        // Kanya is both Budha's exaltation and its own sign. Exaltation is the
        // stronger statement, so it is the one reported.
        assert_eq!(
            dignity_of(Graha::Budha, Rashi::Kanya),
            Some(Dignity::Exalted)
        );
        assert_eq!(
            dignity_of(Graha::Budha, Rashi::Mithuna),
            Some(Dignity::OwnSign)
        );
    }
}
