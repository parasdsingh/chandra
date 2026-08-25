//! Sidereal zodiac arithmetic.
//!
//! Both divisions start from the same origin, 0 degrees sidereal, so a longitude
//! maps to a rashi and a nakshatra by division alone. Every boundary is exact by
//! construction rather than tabulated, which is what keeps entry and exit times
//! consistent with the positions they were derived from.

use serde::{Deserialize, Serialize};

/// Degrees per rashi.
pub const RASHI_ARC: f64 = 30.0;
/// Degrees per nakshatra: 360 / 27 = 13 degrees 20 arcminutes exactly.
pub const NAKSHATRA_ARC: f64 = 360.0 / 27.0;
/// Degrees per pada: a quarter of a nakshatra, 3 degrees 20 arcminutes.
pub const PADA_ARC: f64 = NAKSHATRA_ARC / 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rashi {
    Mesha,
    Vrishabha,
    Mithuna,
    Karka,
    Simha,
    Kanya,
    Tula,
    Vrishchika,
    Dhanu,
    Makara,
    Kumbha,
    Meena,
}

impl Rashi {
    pub const ALL: [Rashi; 12] = [
        Rashi::Mesha,
        Rashi::Vrishabha,
        Rashi::Mithuna,
        Rashi::Karka,
        Rashi::Simha,
        Rashi::Kanya,
        Rashi::Tula,
        Rashi::Vrishchika,
        Rashi::Dhanu,
        Rashi::Makara,
        Rashi::Kumbha,
        Rashi::Meena,
    ];

    /// Rashi containing a sidereal longitude.
    pub fn from_longitude(longitude: f64) -> Self {
        Self::ALL[index_of(longitude, RASHI_ARC, Self::ALL.len())]
    }

    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&r| r == self)
            .expect("ALL is total")
    }

    /// Sidereal longitude at which this rashi begins.
    pub fn start_longitude(self) -> f64 {
        self.index() as f64 * RASHI_ARC
    }

    pub const fn name(self) -> &'static str {
        match self {
            Rashi::Mesha => "Mesha",
            Rashi::Vrishabha => "Vrishabha",
            Rashi::Mithuna => "Mithuna",
            Rashi::Karka => "Karka",
            Rashi::Simha => "Simha",
            Rashi::Kanya => "Kanya",
            Rashi::Tula => "Tula",
            Rashi::Vrishchika => "Vrishchika",
            Rashi::Dhanu => "Dhanu",
            Rashi::Makara => "Makara",
            Rashi::Kumbha => "Kumbha",
            Rashi::Meena => "Meena",
        }
    }

    /// Western equivalent, shown as a secondary label for readers who know the
    /// signs but not the Sanskrit.
    /// Three letters, for a 40px calendar cell.
    ///
    /// Western forms, because the Sanskrit names cannot be abbreviated to three
    /// letters and stay distinct: `Vrishabha` and `Vrishchika` are identical for
    /// six. Both names are already carried on every rashi, so this is a choice
    /// of which to print rather than new data.
    pub const fn short(self) -> &'static str {
        match self {
            Rashi::Mesha => "Ari",
            Rashi::Vrishabha => "Tau",
            Rashi::Mithuna => "Gem",
            Rashi::Karka => "Can",
            Rashi::Simha => "Leo",
            Rashi::Kanya => "Vir",
            Rashi::Tula => "Lib",
            Rashi::Vrishchika => "Sco",
            Rashi::Dhanu => "Sag",
            Rashi::Makara => "Cap",
            Rashi::Kumbha => "Aqu",
            Rashi::Meena => "Pis",
        }
    }

    pub const fn western(self) -> &'static str {
        match self {
            Rashi::Mesha => "Aries",
            Rashi::Vrishabha => "Taurus",
            Rashi::Mithuna => "Gemini",
            Rashi::Karka => "Cancer",
            Rashi::Simha => "Leo",
            Rashi::Kanya => "Virgo",
            Rashi::Tula => "Libra",
            Rashi::Vrishchika => "Scorpio",
            Rashi::Dhanu => "Sagittarius",
            Rashi::Makara => "Capricorn",
            Rashi::Kumbha => "Aquarius",
            Rashi::Meena => "Pisces",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nakshatra {
    Ashwini,
    Bharani,
    Krittika,
    Rohini,
    Mrigashira,
    Ardra,
    Punarvasu,
    Pushya,
    Ashlesha,
    Magha,
    PurvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swati,
    Vishakha,
    Anuradha,
    Jyeshtha,
    Mula,
    PurvaAshadha,
    UttaraAshadha,
    Shravana,
    Dhanishta,
    Shatabhisha,
    PurvaBhadrapada,
    UttaraBhadrapada,
    Revati,
}

impl Nakshatra {
    pub const ALL: [Nakshatra; 27] = [
        Nakshatra::Ashwini,
        Nakshatra::Bharani,
        Nakshatra::Krittika,
        Nakshatra::Rohini,
        Nakshatra::Mrigashira,
        Nakshatra::Ardra,
        Nakshatra::Punarvasu,
        Nakshatra::Pushya,
        Nakshatra::Ashlesha,
        Nakshatra::Magha,
        Nakshatra::PurvaPhalguni,
        Nakshatra::UttaraPhalguni,
        Nakshatra::Hasta,
        Nakshatra::Chitra,
        Nakshatra::Swati,
        Nakshatra::Vishakha,
        Nakshatra::Anuradha,
        Nakshatra::Jyeshtha,
        Nakshatra::Mula,
        Nakshatra::PurvaAshadha,
        Nakshatra::UttaraAshadha,
        Nakshatra::Shravana,
        Nakshatra::Dhanishta,
        Nakshatra::Shatabhisha,
        Nakshatra::PurvaBhadrapada,
        Nakshatra::UttaraBhadrapada,
        Nakshatra::Revati,
    ];

    pub fn from_longitude(longitude: f64) -> Self {
        Self::ALL[index_of(longitude, NAKSHATRA_ARC, Self::ALL.len())]
    }

    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&n| n == self)
            .expect("ALL is total")
    }

    pub fn start_longitude(self) -> f64 {
        self.index() as f64 * NAKSHATRA_ARC
    }

    pub const fn name(self) -> &'static str {
        match self {
            Nakshatra::Ashwini => "Ashwini",
            Nakshatra::Bharani => "Bharani",
            Nakshatra::Krittika => "Krittika",
            Nakshatra::Rohini => "Rohini",
            Nakshatra::Mrigashira => "Mrigashira",
            Nakshatra::Ardra => "Ardra",
            Nakshatra::Punarvasu => "Punarvasu",
            Nakshatra::Pushya => "Pushya",
            Nakshatra::Ashlesha => "Ashlesha",
            Nakshatra::Magha => "Magha",
            Nakshatra::PurvaPhalguni => "Purva Phalguni",
            Nakshatra::UttaraPhalguni => "Uttara Phalguni",
            Nakshatra::Hasta => "Hasta",
            Nakshatra::Chitra => "Chitra",
            Nakshatra::Swati => "Swati",
            Nakshatra::Vishakha => "Vishakha",
            Nakshatra::Anuradha => "Anuradha",
            Nakshatra::Jyeshtha => "Jyeshtha",
            Nakshatra::Mula => "Mula",
            Nakshatra::PurvaAshadha => "Purva Ashadha",
            Nakshatra::UttaraAshadha => "Uttara Ashadha",
            Nakshatra::Shravana => "Shravana",
            Nakshatra::Dhanishta => "Dhanishta",
            Nakshatra::Shatabhisha => "Shatabhisha",
            Nakshatra::PurvaBhadrapada => "Purva Bhadrapada",
            Nakshatra::UttaraBhadrapada => "Uttara Bhadrapada",
            Nakshatra::Revati => "Revati",
        }
    }

    /// A short form for a 40px calendar cell.
    ///
    /// Two parts where the name has two, because six of the twenty-seven begin
    /// `Purva` or `Uttara` and three of each share what follows: `P.Ash` and
    /// `U.Ash` are Purva and Uttara Ashadha, and truncating either to four
    /// letters would give `Asha` for both. The rest are the first four letters
    /// of the name, which are distinct across all twenty-one.
    pub const fn short(self) -> &'static str {
        match self {
            Nakshatra::Ashwini => "Ashw",
            Nakshatra::Bharani => "Bhar",
            Nakshatra::Krittika => "Krit",
            Nakshatra::Rohini => "Rohi",
            Nakshatra::Mrigashira => "Mrig",
            Nakshatra::Ardra => "Ardr",
            Nakshatra::Punarvasu => "Puna",
            Nakshatra::Pushya => "Push",
            Nakshatra::Ashlesha => "Ashl",
            Nakshatra::Magha => "Magh",
            Nakshatra::PurvaPhalguni => "P.Pha",
            Nakshatra::UttaraPhalguni => "U.Pha",
            Nakshatra::Hasta => "Hast",
            Nakshatra::Chitra => "Chit",
            Nakshatra::Swati => "Swat",
            Nakshatra::Vishakha => "Vish",
            Nakshatra::Anuradha => "Anur",
            Nakshatra::Jyeshtha => "Jyes",
            Nakshatra::Mula => "Mula",
            Nakshatra::PurvaAshadha => "P.Ash",
            Nakshatra::UttaraAshadha => "U.Ash",
            Nakshatra::Shravana => "Shra",
            Nakshatra::Dhanishta => "Dhan",
            Nakshatra::Shatabhisha => "Shat",
            Nakshatra::PurvaBhadrapada => "P.Bha",
            Nakshatra::UttaraBhadrapada => "U.Bha",
            Nakshatra::Revati => "Reva",
        }
    }

    /// Ruling graha in the Vimshottari sequence, which repeats every nine
    /// nakshatras starting from Ashwini.
    pub fn lord(self) -> chandra_ephemeris::Graha {
        use chandra_ephemeris::Graha::*;
        const SEQUENCE: [chandra_ephemeris::Graha; 9] = [
            Ketu, Shukra, Surya, Chandra, Mangala, Rahu, Guru, Shani, Budha,
        ];
        SEQUENCE[self.index() % 9]
    }
}

/// Quarter of a nakshatra, numbered 1 to 4.
pub fn pada(longitude: f64) -> u8 {
    let within = longitude.rem_euclid(360.0) % NAKSHATRA_ARC;
    // Clamped because a longitude of exactly 13.333... would otherwise divide to
    // 4 and index a fifth pada that does not exist.
    ((within / PADA_ARC) as u8 + 1).min(4)
}

/// Degrees, arcminutes and arcseconds within the containing rashi, which is how
/// a position is conventionally written and read.
pub fn degrees_in_rashi(longitude: f64) -> (u32, u32, f64) {
    let within = longitude.rem_euclid(360.0) % RASHI_ARC;
    let degrees = within.trunc();
    let minutes_total = (within - degrees) * 60.0;
    let minutes = minutes_total.trunc();
    (
        degrees as u32,
        minutes as u32,
        (minutes_total - minutes) * 60.0,
    )
}

/// Index of the division containing `longitude`.
///
/// Clamped at the top because floating point division of a longitude a hair
/// under 360 can round up to the division count and index past the end.
fn index_of(longitude: f64, arc: f64, count: usize) -> usize {
    let normalised = longitude.rem_euclid(360.0);
    ((normalised / arc) as usize).min(count - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rashi_boundaries_are_exact() {
        assert_eq!(Rashi::from_longitude(0.0), Rashi::Mesha);
        assert_eq!(Rashi::from_longitude(29.999_999), Rashi::Mesha);
        assert_eq!(Rashi::from_longitude(30.0), Rashi::Vrishabha);
        assert_eq!(Rashi::from_longitude(330.0), Rashi::Meena);
        assert_eq!(Rashi::from_longitude(359.999_999_9), Rashi::Meena);
        // Wraparound must land back at the start, not index out of bounds.
        assert_eq!(Rashi::from_longitude(360.0), Rashi::Mesha);
        assert_eq!(Rashi::from_longitude(-0.000_001), Rashi::Meena);
        // -30 normalises to 330, the start of Meena, not the start of Kumbha.
        assert_eq!(Rashi::from_longitude(-30.0), Rashi::Meena);
        assert_eq!(Rashi::from_longitude(-31.0), Rashi::Kumbha);
    }

    #[test]
    fn nakshatra_boundaries_are_exact() {
        assert_eq!(Nakshatra::from_longitude(0.0), Nakshatra::Ashwini);
        assert_eq!(
            Nakshatra::from_longitude(NAKSHATRA_ARC - 1e-9),
            Nakshatra::Ashwini
        );
        assert_eq!(Nakshatra::from_longitude(NAKSHATRA_ARC), Nakshatra::Bharani);
        assert_eq!(Nakshatra::from_longitude(359.999_999_9), Nakshatra::Revati);
        assert_eq!(Nakshatra::from_longitude(360.0), Nakshatra::Ashwini);
    }

    #[test]
    fn every_division_start_maps_back_to_itself() {
        for rashi in Rashi::ALL {
            assert_eq!(Rashi::from_longitude(rashi.start_longitude()), rashi);
        }
        for nakshatra in Nakshatra::ALL {
            assert_eq!(
                Nakshatra::from_longitude(nakshatra.start_longitude()),
                nakshatra
            );
        }
    }

    #[test]
    fn pada_is_one_to_four_everywhere() {
        let mut seen = [false; 4];
        let mut longitude = 0.0;
        while longitude < 360.0 {
            let p = pada(longitude);
            assert!((1..=4).contains(&p), "pada {p} at {longitude}");
            seen[p as usize - 1] = true;
            longitude += 0.37;
        }
        assert!(seen.iter().all(|&s| s), "all four padas must occur");
        assert_eq!(pada(0.0), 1);
        assert_eq!(pada(PADA_ARC - 1e-9), 1);
        assert_eq!(pada(PADA_ARC), 2);
        assert_eq!(pada(NAKSHATRA_ARC - 1e-9), 4);
        assert_eq!(pada(NAKSHATRA_ARC), 1);
    }

    #[test]
    fn known_position_decomposes_correctly() {
        // Moon at 211.5053 sidereal on 2026-08-20, verified in docs/RESEARCH.md.
        let longitude = 211.505_313_5;
        assert_eq!(Rashi::from_longitude(longitude), Rashi::Vrishchika);
        assert_eq!(Nakshatra::from_longitude(longitude), Nakshatra::Vishakha);
        assert_eq!(pada(longitude), 4);
        let (d, m, _) = degrees_in_rashi(longitude);
        assert_eq!((d, m), (1, 30));
    }

    #[test]
    fn vimshottari_lords_cycle_every_nine() {
        for nakshatra in Nakshatra::ALL {
            let ninth_later = Nakshatra::ALL[(nakshatra.index() + 9) % 27];
            assert_eq!(nakshatra.lord(), ninth_later.lord());
        }
        assert_eq!(Nakshatra::Ashwini.lord(), chandra_ephemeris::Graha::Ketu);
        assert_eq!(Nakshatra::Rohini.lord(), chandra_ephemeris::Graha::Chandra);
    }
}

#[cfg(test)]
mod short_form_tests {
    use super::*;

    /// A label that names two things names neither. Every short form has to be
    /// unique inside its own division, which is the whole reason the two-part
    /// nakshatra forms exist.
    #[test]
    fn short_forms_are_distinct_within_their_division() {
        let rashis: std::collections::BTreeSet<&str> =
            Rashi::ALL.iter().map(|rashi| rashi.short()).collect();
        assert_eq!(rashis.len(), 12, "two rashis share a short form");

        let nakshatras: std::collections::BTreeSet<&str> = Nakshatra::ALL
            .iter()
            .map(|nakshatra| nakshatra.short())
            .collect();
        assert_eq!(nakshatras.len(), 27, "two nakshatras share a short form");
    }

    /// The cell is 40px wide and the label replaces a 16px glyph in it. Five
    /// characters is what the two-part nakshatra forms need and is the ceiling.
    #[test]
    fn short_forms_fit_a_cell() {
        for rashi in Rashi::ALL {
            assert_eq!(rashi.short().chars().count(), 3, "{rashi:?}");
        }
        for nakshatra in Nakshatra::ALL {
            let length = nakshatra.short().chars().count();
            assert!(
                (4..=5).contains(&length),
                "{nakshatra:?} is {length} characters"
            );
        }
    }

    /// The six that begin Purva or Uttara are the reason for the two-part form,
    /// and three pairs of them would collide without it.
    #[test]
    fn the_purva_and_uttara_pairs_stay_apart() {
        for (purva, uttara) in [
            (Nakshatra::PurvaPhalguni, Nakshatra::UttaraPhalguni),
            (Nakshatra::PurvaAshadha, Nakshatra::UttaraAshadha),
            (Nakshatra::PurvaBhadrapada, Nakshatra::UttaraBhadrapada),
        ] {
            assert_ne!(purva.short(), uttara.short());
            assert!(purva.short().starts_with("P."));
            assert!(uttara.short().starts_with("U."));
        }
    }

    /// Vrishabha and Vrishchika are identical for six letters, which is why the
    /// rashi forms are western rather than transliterated.
    #[test]
    fn the_rashi_pair_that_forced_western_forms_stays_apart() {
        assert_eq!(Rashi::Vrishabha.short(), "Tau");
        assert_eq!(Rashi::Vrishchika.short(), "Sco");
        // Three letters of either transliteration is "Vri". That is the whole
        // argument for western forms, so it is asserted rather than asserted
        // about: if these ever differ, transliterated forms become possible.
        assert_eq!(Rashi::Vrishabha.name()[..3], Rashi::Vrishchika.name()[..3]);
        assert_eq!(&Rashi::Vrishabha.name()[..5], "Vrish");
        assert_eq!(&Rashi::Vrishchika.name()[..5], "Vrish");
    }
}
