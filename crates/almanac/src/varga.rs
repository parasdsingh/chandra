//! Divisional charts.
//!
//! A varga divides each rashi into `n` parts and maps each part to a rashi. What
//! differs between them is only *which* rashi a part maps to, so one function
//! shape serves all of them: find the part, apply the varga's own rule.
//!
//! The specification is `docs/design/vargas.md`, which cites every rule to a
//! source and carries the worked examples the tests below are taken from. That
//! document exists because this is exactly the kind of table that is wrong when
//! it is written from memory - the Durmuhurtam table was, and two of its values
//! were wrong.
//!
//! **The Parashari scheme only** (D-032). Every varga here has published
//! alternatives, and three of the four kinds are a flag on this same arithmetic
//! rather than a different rule. None is offered: a chart that silently used one
//! reading of a disputed matter would be the app asserting an interpretation,
//! which is the same reason it declares no winner in a planetary war.
//!
//! **Rahu and Ketu take the same rule as any other graha.** Searched for
//! specifically: no classical text addresses the nodes in vargas at all, and
//! four independent implementations have no node branch. A reversal for the
//! nodes is the kind of rule that sounds plausible enough to write without a
//! source, so it is written here that there is none.

use serde::{Deserialize, Serialize};

use crate::zodiac::{Rashi, RASHI_ARC};

/// A divisional chart.
///
/// The sixteen of the Shodasavarga. Between them they span every speed the chart
/// changes at: D1 moves the lagna one sign every two hours, D60 every two
/// minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Varga {
    /// D1. The rashi chart itself: the sign a body stands in.
    #[default]
    D1,
    /// D2, the hora. Occupies **two signs only** - see [`varga_rashi`].
    D2,
    /// D3, the drekkana.
    D3,
    /// D4, the chaturthamsa.
    D4,
    /// D7, the saptamsa.
    D7,
    /// D9, the navamsa. The one varga every practitioner reads beside D1.
    D9,
    /// D10, the dasamsa.
    D10,
    /// D12, the dwadasamsa.
    D12,
    /// D16, the shodasamsa.
    D16,
    /// D20, the vimsamsa.
    D20,
    /// D24, the chaturvimsamsa.
    D24,
    /// D27, the bhamsa.
    D27,
    /// D30, the trimsamsa. The one **unequal** division - see [`varga_rashi`].
    D30,
    /// D40, the khavedamsa.
    D40,
    /// D45, the akshavedamsa.
    D45,
    /// D60, the shashtiamsa.
    D60,
}

impl Varga {
    pub const ALL: [Varga; 16] = [
        Varga::D1,
        Varga::D2,
        Varga::D3,
        Varga::D4,
        Varga::D7,
        Varga::D9,
        Varga::D10,
        Varga::D12,
        Varga::D16,
        Varga::D20,
        Varga::D24,
        Varga::D27,
        Varga::D30,
        Varga::D40,
        Varga::D45,
        Varga::D60,
    ];

    /// Stable key for settings and IPC.
    pub const fn key(self) -> &'static str {
        match self {
            Varga::D1 => "d1",
            Varga::D2 => "d2",
            Varga::D3 => "d3",
            Varga::D4 => "d4",
            Varga::D7 => "d7",
            Varga::D9 => "d9",
            Varga::D10 => "d10",
            Varga::D12 => "d12",
            Varga::D16 => "d16",
            Varga::D20 => "d20",
            Varga::D24 => "d24",
            Varga::D27 => "d27",
            Varga::D30 => "d30",
            Varga::D40 => "d40",
            Varga::D45 => "d45",
            Varga::D60 => "d60",
        }
    }

    /// How the chart names itself: the division and what it is called.
    pub const fn label(self) -> &'static str {
        match self {
            Varga::D1 => "D1 · Rashi",
            Varga::D2 => "D2 · Hora",
            Varga::D3 => "D3 · Drekkana",
            Varga::D4 => "D4 · Chaturthamsa",
            Varga::D7 => "D7 · Saptamsa",
            Varga::D9 => "D9 · Navamsa",
            Varga::D10 => "D10 · Dasamsa",
            Varga::D12 => "D12 · Dwadasamsa",
            Varga::D16 => "D16 · Shodasamsa",
            Varga::D20 => "D20 · Vimsamsa",
            Varga::D24 => "D24 · Chaturvimsamsa",
            Varga::D27 => "D27 · Bhamsa",
            Varga::D30 => "D30 · Trimsamsa",
            Varga::D40 => "D40 · Khavedamsa",
            Varga::D45 => "D45 · Akshavedamsa",
            Varga::D60 => "D60 · Shashtiamsa",
        }
    }

    /// Parts a rashi is divided into.
    ///
    /// D30 is **five**, not thirty. Its name is the odd one out: a trimsamsa is a
    /// thirtieth of a rashi, but the Parashari scheme groups those thirty degrees
    /// into five unequal blocks of 5, 5, 8, 7 and 5.
    pub const fn parts(self) -> u32 {
        match self {
            Varga::D1 => 1,
            Varga::D2 => 2,
            Varga::D3 => 3,
            Varga::D4 => 4,
            Varga::D7 => 7,
            Varga::D9 => 9,
            Varga::D10 => 10,
            Varga::D12 => 12,
            Varga::D16 => 16,
            Varga::D20 => 20,
            Varga::D24 => 24,
            Varga::D27 => 27,
            Varga::D30 => 5,
            Varga::D40 => 40,
            Varga::D45 => 45,
            Varga::D60 => 60,
        }
    }

    /// The number in the name: D9's is 9, D30's is 30.
    ///
    /// Not [`parts`](Self::parts), which they agree with in fifteen of sixteen
    /// cases and not in D30's: a trimsamsa is a *thirtieth* of a rashi, and the
    /// Parashari scheme groups those thirty degrees into five unequal blocks. The
    /// chart is called D30 and divides into 5.
    pub const fn division(self) -> u32 {
        match self {
            Varga::D30 => 30,
            other => other.parts(),
        }
    }

    /// Whether every part of a rashi is the same width.
    ///
    /// True for fifteen of the sixteen. D30 is the exception in the Parashari
    /// scheme, and the exception is the whole of its difficulty.
    pub const fn equal(self) -> bool {
        !matches!(self, Varga::D30)
    }

    /// How many rashis the division can ever occupy.
    ///
    /// Twelve for most, and the two that are not are a drawing problem before
    /// they are anything else. **D2 reaches two**: every body in the chart lands
    /// in Karka or Simha, so ten compartments are always empty and one of the two
    /// may hold all nine. **D30 reaches ten**: its parts go to the five
    /// non-luminaries' own signs, so Karka and Simha - the Moon's and the Sun's -
    /// can never appear.
    pub const fn occupiable(self) -> usize {
        match self {
            Varga::D2 => 2,
            Varga::D30 => 10,
            _ => 12,
        }
    }

    /// Roughly how long the chart holds still, for a body at the lagna's rate.
    ///
    /// The ascendant crosses a rashi in about two hours, so it crosses one part
    /// of an `n`-part division in about `120/n` minutes. An average for D30,
    /// whose parts run from 5° to 8° and so from 20 to 32 minutes.
    pub const fn lagna_minutes(self) -> u32 {
        120 / self.parts()
    }
}

/// Which rashi a sidereal longitude occupies in a varga.
///
/// The rules, each cited in `docs/design/vargas.md` §7. `s` is the rashi index
/// and `p` the part index, both from zero:
///
/// | Varga | Rule |
/// |---|---|
/// | D1 | `s` |
/// | D2 | odd sign: Simha then Karka; even sign: reversed |
/// | D3 | `(s + 4p) mod 12` |
/// | D4 | `(s + 3p) mod 12` |
/// | D7 | `(7s + p) mod 12` |
/// | D9 | `(9s + p) mod 12` |
/// | D10 | `(s + 8·(s mod 2) + p) mod 12` |
/// | D12 | `(s + p) mod 12` |
/// | D16 | `(4s + p) mod 12` |
/// | D20 | `(8s + p) mod 12` |
/// | D24 | `(4 − (s mod 2) + p) mod 12` |
/// | D27 | `(3s + p) mod 12` |
/// | D30 | an unequal table, below |
/// | D40 | `(6·(s mod 2) + p) mod 12` |
/// | D45 | `(4·(s mod 3) + p) mod 12` |
/// | D60 | `(s + p) mod 12` |
///
/// Most sources state these as a starting sign per class of rashi rather than as
/// arithmetic - "movable from itself, fixed from the 9th, dual from the 5th" for
/// D9. The formulas reproduce that; the tests check they do, at all twelve signs,
/// which is what earns the right to write a multiplier instead of a table.
///
/// Two do not reduce to a multiplier.
///
/// **D2** maps to two signs only, and to no others. It is the only varga whose
/// targets are not reached by counting.
///
/// **D30** is not an equal division. The five parts run 5°, 5°, 8°, 7° and 5° in
/// an odd sign and are reversed in an even one, and their targets are the five
/// non-luminary planets' own signs rather than a count from anywhere.
pub fn varga_rashi(varga: Varga, longitude: f64) -> Rashi {
    let longitude = longitude.rem_euclid(360.0);
    let sign = (longitude / RASHI_ARC) as usize % 12;
    let within = longitude - sign as f64 * RASHI_ARC;

    if varga == Varga::D30 {
        return trimsamsa(sign, within);
    }

    // Clamped to the last part. `within` is under 30 by construction, so `part`
    // is under `parts` - unless a rounding error at a rashi boundary makes the
    // division land exactly on it, which would index past the end.
    let parts = varga.parts() as usize;
    let part = ((within / RASHI_ARC * parts as f64) as usize).min(parts - 1);

    if varga == Varga::D2 {
        return hora(sign, part);
    }

    let index = match varga {
        Varga::D1 => sign,
        Varga::D3 => sign + 4 * part,
        Varga::D4 => sign + 3 * part,
        Varga::D7 => 7 * sign + part,
        Varga::D9 => 9 * sign + part,
        Varga::D10 => sign + 8 * (sign % 2) + part,
        Varga::D12 => sign + part,
        Varga::D16 => 4 * sign + part,
        Varga::D20 => 8 * sign + part,
        Varga::D24 => 4 - (sign % 2) + part,
        Varga::D27 => 3 * sign + part,
        Varga::D40 => 6 * (sign % 2) + part,
        Varga::D45 => 4 * (sign % 3) + part,
        Varga::D60 => sign + part,
        // Both are handled above, and neither is a multiplier.
        Varga::D2 | Varga::D30 => unreachable!("handled before the table"),
    };

    Rashi::ALL[index % 12]
}

/// D2, the hora.
///
/// The classical texts give the two halves to the Sun and the Moon **as lords**
/// and never name a sign; the step from "the Sun's hora" to Simha is a modern
/// completion, and this is the one every implementation makes (D-032).
///
/// The consequence is that a hora chart occupies two compartments out of twelve.
/// That is not a defect in the drawing.
fn hora(sign: usize, half: usize) -> Rashi {
    let odd_sign = sign % 2 == 0;
    match (odd_sign, half) {
        (true, 0) | (false, 1) => Rashi::Simha,
        _ => Rashi::Karka,
    }
}

/// D30, the trimsamsa.
///
/// Five unequal parts, whose targets are the own signs of the five planets that
/// rule them - Mangala, Shani, Guru, Budha, Shukra in an odd rashi, and the
/// reverse in an even one. Each of those five owns one odd sign and one even
/// one, and the part takes the sign matching the rashi's own parity, which is
/// why the even column is the odd column reversed in both order and width.
///
/// The Sun and the Moon rule no trimsamsa, so **Karka and Simha never appear**.
///
/// The target does not depend on the rashi beyond its parity: every odd rashi's
/// first five degrees go to Mesha, and every even rashi's to Vrishabha.
fn trimsamsa(sign: usize, within: f64) -> Rashi {
    if sign % 2 == 0 {
        match within {
            d if d < 5.0 => Rashi::Mesha,
            d if d < 10.0 => Rashi::Kumbha,
            d if d < 18.0 => Rashi::Dhanu,
            d if d < 25.0 => Rashi::Mithuna,
            _ => Rashi::Tula,
        }
    } else {
        match within {
            d if d < 5.0 => Rashi::Vrishabha,
            d if d < 12.0 => Rashi::Kanya,
            d if d < 20.0 => Rashi::Meena,
            d if d < 25.0 => Rashi::Makara,
            _ => Rashi::Vrishchika,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every worked example in `docs/design/vargas.md` §7, for the five vargas
    /// built here.
    ///
    /// Transcribed from the document rather than computed from this code, which
    /// is the point of them: a table generated from the implementation it is
    /// meant to check agrees with itself and with nothing else. Each varga's set
    /// includes an odd sign, an even sign, and the pair of longitudes either
    /// side of a part boundary.
    #[test]
    fn the_specification_worked_examples() {
        let cases: &[(Varga, f64, Rashi)] = &[
            // §7.3 D3
            (Varga::D3, 5.0, Rashi::Mesha),
            (Varga::D3, 15.0, Rashi::Simha),
            (Varga::D3, 25.0, Rashi::Dhanu),
            (Varga::D3, 55.0, Rashi::Makara),
            (Varga::D3, 9.99972, Rashi::Mesha),
            (Varga::D3, 10.0, Rashi::Simha),
            // §7.5 D7
            (Varga::D7, 5.0, Rashi::Vrishabha),
            (Varga::D7, 35.0, Rashi::Dhanu),
            (Varga::D7, 70.0, Rashi::Simha),
            (Varga::D7, 169.0, Rashi::Karka),
            (Varga::D7, 4.28556, Rashi::Mesha),
            (Varga::D7, 4.28583, Rashi::Vrishabha),
            // §7.6 D9
            (Varga::D9, 5.0, Rashi::Vrishabha),
            (Varga::D9, 35.0, Rashi::Kumbha),
            (Varga::D9, 65.0, Rashi::Vrishchika),
            (Varga::D9, 95.0, Rashi::Simha),
            (Varga::D9, 229.0, Rashi::Dhanu),
            (Varga::D9, 3.33306, Rashi::Mesha),
            // The document prints this boundary as `3.33333`, which is five
            // decimal places of 3°20′00″ and therefore *below* it: 3°20′ is
            // 10/3 = 3.33333… recurring. At the printed value the answer really
            // is Mesha, and only at the exact boundary does it become
            // Vrishabha. Both are asserted, because the difference between them
            // is the boundary this example exists to pin.
            (Varga::D9, 3.33333, Rashi::Mesha),
            (Varga::D9, 10.0 / 3.0, Rashi::Vrishabha),
            // §7.2 D2
            (Varga::D2, 7.0, Rashi::Simha),
            (Varga::D2, 22.0, Rashi::Karka),
            (Varga::D2, 37.0, Rashi::Karka),
            (Varga::D2, 52.0, Rashi::Simha),
            (Varga::D2, 14.99972, Rashi::Simha),
            (Varga::D2, 15.0, Rashi::Karka),
            // §7.4 D4
            (Varga::D4, 3.0, Rashi::Mesha),
            (Varga::D4, 20.0, Rashi::Tula),
            (Varga::D4, 44.0, Rashi::Simha),
            (Varga::D4, 53.0, Rashi::Kumbha),
            (Varga::D4, 7.49972, Rashi::Mesha),
            (Varga::D4, 7.5, Rashi::Karka),
            // §7.7 D10
            (Varga::D10, 5.0, Rashi::Vrishabha),
            (Varga::D10, 35.0, Rashi::Kumbha),
            (Varga::D10, 70.0, Rashi::Kanya),
            (Varga::D10, 229.0, Rashi::Makara),
            (Varga::D10, 2.99972, Rashi::Mesha),
            (Varga::D10, 3.0, Rashi::Vrishabha),
            // §7.8 D12
            (Varga::D12, 5.0, Rashi::Mithuna),
            (Varga::D12, 56.0, Rashi::Meena),
            (Varga::D12, 229.0, Rashi::Mithuna),
            (Varga::D12, 2.49972, Rashi::Mesha),
            (Varga::D12, 2.5, Rashi::Vrishabha),
            // §7.9 D16
            (Varga::D16, 5.0, Rashi::Mithuna),
            (Varga::D16, 35.0, Rashi::Tula),
            (Varga::D16, 71.0, Rashi::Vrishabha),
            (Varga::D16, 229.0, Rashi::Mithuna),
            (Varga::D16, 1.87472, Rashi::Mesha),
            (Varga::D16, 1.875, Rashi::Vrishabha),
            // §7.10 D20
            (Varga::D20, 5.0, Rashi::Karka),
            (Varga::D20, 35.0, Rashi::Meena),
            (Varga::D20, 71.0, Rashi::Meena),
            (Varga::D20, 229.0, Rashi::Dhanu),
            (Varga::D20, 1.49972, Rashi::Mesha),
            (Varga::D20, 1.5, Rashi::Vrishabha),
            // §7.11 D24
            (Varga::D24, 5.0, Rashi::Dhanu),
            (Varga::D24, 35.0, Rashi::Vrishchika),
            (Varga::D24, 71.0, Rashi::Mesha),
            (Varga::D24, 229.0, Rashi::Tula),
            (Varga::D24, 1.24972, Rashi::Simha),
            (Varga::D24, 1.25, Rashi::Kanya),
            // §7.12 D27
            (Varga::D27, 5.0, Rashi::Simha),
            (Varga::D27, 35.0, Rashi::Vrishchika),
            (Varga::D27, 71.0, Rashi::Karka),
            (Varga::D27, 95.0, Rashi::Vrishabha),
            (Varga::D27, 229.0, Rashi::Mithuna),
            (Varga::D27, 1.11083, Rashi::Mesha),
            (Varga::D27, 10.0 / 9.0, Rashi::Vrishabha),
            (Varga::D27, 1.11139, Rashi::Vrishabha),
            // §7.13 D30, the unequal one. Every boundary of both parities.
            (Varga::D30, 3.0, Rashi::Mesha),
            (Varga::D30, 7.0, Rashi::Kumbha),
            (Varga::D30, 14.0, Rashi::Dhanu),
            (Varga::D30, 20.0, Rashi::Mithuna),
            (Varga::D30, 27.0, Rashi::Tula),
            (Varga::D30, 33.0, Rashi::Vrishabha),
            (Varga::D30, 37.0, Rashi::Kanya),
            (Varga::D30, 44.0, Rashi::Meena),
            (Varga::D30, 52.0, Rashi::Makara),
            (Varga::D30, 57.0, Rashi::Vrishchika),
            // §7.14 D40
            (Varga::D40, 5.0, Rashi::Tula),
            (Varga::D40, 35.0, Rashi::Mesha),
            (Varga::D40, 71.0, Rashi::Mithuna),
            (Varga::D40, 229.0, Rashi::Vrishchika),
            (Varga::D40, 0.74972, Rashi::Mesha),
            (Varga::D40, 0.75, Rashi::Vrishabha),
            // §7.15 D45
            (Varga::D45, 5.0, Rashi::Vrishchika),
            (Varga::D45, 35.0, Rashi::Meena),
            (Varga::D45, 65.0, Rashi::Karka),
            (Varga::D45, 229.0, Rashi::Dhanu),
            (Varga::D45, 0.66639, Rashi::Mesha),
            (Varga::D45, 0.66667, Rashi::Vrishabha),
            // §7.16 D60
            (Varga::D60, 5.0, Rashi::Kumbha),
            (Varga::D60, 35.0, Rashi::Meena),
            (Varga::D60, 222.9667, Rashi::Dhanu),
            (Varga::D60, 283.4167, Rashi::Meena),
            (Varga::D60, 0.49972, Rashi::Mesha),
            (Varga::D60, 0.5, Rashi::Vrishabha),
        ];

        for &(varga, longitude, expected) in cases {
            assert_eq!(
                varga_rashi(varga, longitude),
                expected,
                "{} at {longitude}",
                varga.label()
            );
        }
    }

    /// D1 is the identity, at every sign.
    #[test]
    fn d1_is_the_sign_itself() {
        for (index, rashi) in Rashi::ALL.into_iter().enumerate() {
            let middle = index as f64 * RASHI_ARC + 15.0;
            assert_eq!(varga_rashi(Varga::D1, middle), rashi);
        }
    }

    /// The classical statements of D9, checked against the formula.
    ///
    /// Sources give the rule as a starting sign per class rather than as
    /// arithmetic: a movable sign's navamsas begin at itself, a fixed sign's at
    /// the 9th from it, a dual sign's at the 5th. This asserts `(9s + p) mod 12`
    /// reproduces that at every one of the twelve, which is what earns the right
    /// to write the formula instead of the table.
    #[test]
    fn d9_reproduces_the_classical_start_signs() {
        for (sign, rashi) in Rashi::ALL.into_iter().enumerate() {
            let start = match sign % 3 {
                0 => sign,     // movable: from itself
                1 => sign + 8, // fixed: the 9th from it, counted inclusively
                _ => sign + 4, // dual: the 5th from it
            } % 12;

            let first = varga_rashi(Varga::D9, rashi.start_longitude() + 0.5);
            assert_eq!(first, Rashi::ALL[start], "{}'s first navamsa", rashi.name());
        }
    }

    /// D3 puts a body in the 1st, 5th or 9th from its own sign, and nothing else.
    #[test]
    fn d3_reaches_only_the_trines() {
        for (sign, rashi) in Rashi::ALL.into_iter().enumerate() {
            for part in 0..3 {
                let longitude = rashi.start_longitude() + f64::from(part) * 10.0 + 1.0;
                let got = varga_rashi(Varga::D3, longitude).index();
                let offset = (got + 12 - sign) % 12;
                assert!(
                    offset == 0 || offset == 4 || offset == 8,
                    "{} part {part} landed {offset} signs away",
                    rashi.name()
                );
            }
        }
    }

    /// Where the nodes fall together, and where they stay apart.
    ///
    /// Ketu is Rahu plus 180° (D-003), so the two always sit in signs six apart.
    /// A sign and the sign opposite it share their **parity** and their
    /// **movable/fixed/dual class**, and differ in their **element** and of
    /// course their position. So a varga keyed on the first two puts the nodes in
    /// the *same* rashi, and one keyed on the last two keeps them opposite.
    ///
    /// Seven do the first: D2, D16, D20, D24, D30, D40, D45. Nine do the second.
    /// Derived from the rules rather than observed, then asserted here at every
    /// degree of the circle - it is a strong check on all sixteen at once, and a
    /// drawing constraint besides, because it says which charts can put nine
    /// bodies where eight would otherwise be the ceiling.
    #[test]
    fn the_nodes_fall_together_in_exactly_seven_divisions() {
        const TOGETHER: [Varga; 7] = [
            Varga::D2,
            Varga::D16,
            Varga::D20,
            Varga::D24,
            Varga::D30,
            Varga::D40,
            Varga::D45,
        ];

        for varga in Varga::ALL {
            let expected = if TOGETHER.contains(&varga) { 0 } else { 6 };
            for step in 0..360 {
                let rahu = f64::from(step) + 0.37;
                let ketu = rahu + 180.0;
                let separation =
                    (varga_rashi(varga, ketu).index() + 12 - varga_rashi(varga, rahu).index()) % 12;
                assert_eq!(separation, expected, "{} at {rahu}", varga.label());
            }
        }
    }

    /// Each division reaches exactly the rashis it can reach.
    ///
    /// Two of the sixteen do not reach all twelve, and both are a drawing problem
    /// before they are anything else. D2 lands every body in Karka or Simha, so
    /// ten compartments are always empty and one of the two may hold all nine.
    /// D30's parts go to the five non-luminaries' own signs, so the Moon's and
    /// the Sun's - Karka and Simha - never appear.
    #[test]
    fn each_division_occupies_the_signs_it_should() {
        for varga in Varga::ALL {
            // Indices rather than the rashis themselves: `Rashi` is not ordered,
            // and giving it an ordering to satisfy a set here would assert
            // something about the zodiac that this test does not need.
            let mut seen = std::collections::BTreeSet::new();
            // Every tenth of a degree, which lands inside every part of every
            // division: the finest is D60's half a degree.
            for step in 0..3600 {
                seen.insert(varga_rashi(varga, f64::from(step) / 10.0).index());
            }

            assert_eq!(
                seen.len(),
                varga.occupiable(),
                "{} reached {:?}",
                varga.label(),
                seen.iter()
                    .map(|index| Rashi::ALL[*index].name())
                    .collect::<Vec<_>>()
            );

            if varga == Varga::D2 {
                assert!(
                    seen.contains(&Rashi::Karka.index()) && seen.contains(&Rashi::Simha.index())
                );
            }
            if varga == Varga::D30 {
                assert!(
                    !seen.contains(&Rashi::Karka.index()) && !seen.contains(&Rashi::Simha.index()),
                    "the Sun and the Moon rule no trimsamsa"
                );
            }
        }
    }

    /// Every part of every sign lands somewhere, for every varga.
    ///
    /// The indexing is arithmetic on a float division, so this walks each part's
    /// two edges and its middle looking for a panic or a wrapped index.
    #[test]
    fn no_longitude_falls_outside_a_rashi() {
        for varga in Varga::ALL {
            let arc = RASHI_ARC / f64::from(varga.parts());
            for sign in 0..12 {
                for part in 0..varga.parts() {
                    let start = f64::from(sign) * RASHI_ARC + f64::from(part) * arc;
                    for probe in [start, start + arc / 2.0, start + arc - 1e-9] {
                        let _ = varga_rashi(varga, probe);
                    }
                }
            }
        }
        // The wrap points either side of zero.
        for probe in [-1e-9, 0.0, 360.0, 359.999_999_999, -0.5, 720.5] {
            for varga in Varga::ALL {
                let _ = varga_rashi(varga, probe);
            }
        }
    }
}
