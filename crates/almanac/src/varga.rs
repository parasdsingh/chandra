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
/// The first pass carries five. They are the ones whose rule is agreed by every
/// classical source consulted, and between them they span the speeds the chart
/// changes at: D1 moves the lagna one sign every two hours, D12 every ten
/// minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Varga {
    /// D1. The rashi chart itself: the sign a body stands in.
    #[default]
    D1,
    /// D3, the drekkana. Read for coborn.
    D3,
    /// D7, the saptamsa. Read for progeny.
    D7,
    /// D9, the navamsa. Read for the spouse, and the one varga every
    /// practitioner reads beside D1.
    D9,
    /// D12, the dwadasamsa. Read for parents.
    D12,
}

impl Varga {
    pub const ALL: [Varga; 5] = [Varga::D1, Varga::D3, Varga::D7, Varga::D9, Varga::D12];

    /// Stable key for settings and IPC.
    pub const fn key(self) -> &'static str {
        match self {
            Varga::D1 => "d1",
            Varga::D3 => "d3",
            Varga::D7 => "d7",
            Varga::D9 => "d9",
            Varga::D12 => "d12",
        }
    }

    /// How the chart names itself: the division and what it is called.
    pub const fn label(self) -> &'static str {
        match self {
            Varga::D1 => "D1 · Rashi",
            Varga::D3 => "D3 · Drekkana",
            Varga::D7 => "D7 · Saptamsa",
            Varga::D9 => "D9 · Navamsa",
            Varga::D12 => "D12 · Dwadasamsa",
        }
    }

    /// Parts a rashi is divided into.
    pub const fn parts(self) -> u32 {
        match self {
            Varga::D1 => 1,
            Varga::D3 => 3,
            Varga::D7 => 7,
            Varga::D9 => 9,
            Varga::D12 => 12,
        }
    }

    /// How long the chart holds still, for a body moving at the lagna's rate.
    ///
    /// The ascendant crosses a rashi in about two hours, so it crosses one part
    /// of an `n`-part division in about `120/n` minutes. This is what makes the
    /// higher vargas worth animating and the lower ones not.
    pub const fn lagna_minutes(self) -> u32 {
        120 / self.parts()
    }
}

/// Which rashi a sidereal longitude occupies in a varga.
///
/// The rules, each cited in `docs/design/vargas.md` §7:
///
/// | Varga | Rule | Reads as |
/// |---|---|---|
/// | D1 | `s` | the sign itself |
/// | D3 | `(s + 4p) mod 12` | the 1st, 5th and 9th from the sign |
/// | D7 | `(7s + p) mod 12` | odd signs from themselves, even from the 7th |
/// | D9 | `(9s + p) mod 12` | movable from itself, fixed from the 9th, dual from the 5th |
/// | D12 | `(s + p) mod 12` | always from the sign itself |
///
/// The formulas are the same rules the sources give in words. D9's is the one
/// worth checking by hand: `9s mod 12` is 0 for movable signs, 9 for fixed and 6
/// for dual, which is exactly "from itself, from the 9th, from the 5th" counted
/// inclusively.
pub fn varga_rashi(varga: Varga, longitude: f64) -> Rashi {
    let longitude = longitude.rem_euclid(360.0);
    let sign = (longitude / RASHI_ARC) as usize % 12;
    let within = longitude - sign as f64 * RASHI_ARC;

    // Clamped to the last part. `within` is under 30 by construction, so
    // `part` is under `parts` - unless a rounding error at a rashi boundary
    // makes the division land exactly on it, which would index past the end.
    let part =
        ((within / RASHI_ARC * f64::from(varga.parts())) as usize).min(varga.parts() as usize - 1);

    let index = match varga {
        Varga::D1 => sign,
        Varga::D3 => sign + 4 * part,
        Varga::D7 => 7 * sign + part,
        Varga::D9 => 9 * sign + part,
        Varga::D12 => sign + part,
    };

    Rashi::ALL[index % 12]
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
            // §7.8 D12
            (Varga::D12, 5.0, Rashi::Mithuna),
            (Varga::D12, 56.0, Rashi::Meena),
            (Varga::D12, 229.0, Rashi::Mithuna),
            (Varga::D12, 2.49972, Rashi::Mesha),
            (Varga::D12, 2.5, Rashi::Vrishabha),
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

    /// Ketu is opposite Rahu in D1, D3, D7, D9 and D12.
    ///
    /// A varga keyed on a sign's *parity* or its movable/fixed/dual *class* puts
    /// the nodes in the same rashi, because a sign and the sign opposite it share
    /// both. One keyed on position or element separates them. All five here are
    /// the second kind, so the nodes stay opposite - which is the property to
    /// notice if a later varga breaks it, since D2, D16, D20, D24, D30, D40 and
    /// D45 all put them together (`docs/design/vargas.md` §4.3).
    #[test]
    fn the_nodes_stay_opposite_in_all_five() {
        for varga in Varga::ALL {
            for step in 0..360 {
                let rahu = f64::from(step) + 0.37;
                let ketu = rahu + 180.0;
                let separation =
                    (varga_rashi(varga, ketu).index() + 12 - varga_rashi(varga, rahu).index()) % 12;
                assert_eq!(
                    separation,
                    6,
                    "{} at {rahu}: the nodes must stay opposite",
                    varga.label()
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
