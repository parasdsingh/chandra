//! Lunar phase naming.
//!
//! Phase is a geometric relationship between Sun, Moon and Earth, so none of
//! this depends on the ayanamsa. Elongation here always means the Moon's
//! longitude minus the Sun's, normalised to `[0, 360)`, which increases
//! monotonically at roughly 12.2 degrees a day and is zero at new moon.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseName {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl PhaseName {
    pub const fn label(self) -> &'static str {
        match self {
            PhaseName::NewMoon => "New Moon",
            PhaseName::WaxingCrescent => "Waxing Crescent",
            PhaseName::FirstQuarter => "First Quarter",
            PhaseName::WaxingGibbous => "Waxing Gibbous",
            PhaseName::FullMoon => "Full Moon",
            PhaseName::WaningGibbous => "Waning Gibbous",
            PhaseName::LastQuarter => "Last Quarter",
            PhaseName::WaningCrescent => "Waning Crescent",
        }
    }

    /// True for the four phases that occur at a single instant rather than over
    /// a stretch of days.
    pub const fn is_principal(self) -> bool {
        matches!(
            self,
            PhaseName::NewMoon
                | PhaseName::FirstQuarter
                | PhaseName::FullMoon
                | PhaseName::LastQuarter
        )
    }
}

/// The four principal phases and the elongation at which each occurs.
pub const PRINCIPAL_PHASES: [(PhaseName, f64); 4] = [
    (PhaseName::NewMoon, 0.0),
    (PhaseName::FirstQuarter, 90.0),
    (PhaseName::FullMoon, 180.0),
    (PhaseName::LastQuarter, 270.0),
];

/// Name for a day on which no principal phase occurs.
///
/// Deliberately never returns a principal name. Splitting the 360 degrees into
/// eight equal sectors, as many phase widgets do, would label three consecutive
/// days "Full Moon" and none of them would be the day it was actually full.
/// A principal name is only ever used by [`principal_phase_in`], which requires
/// the exact instant to fall inside the day.
pub fn intermediate_phase(elongation: f64) -> PhaseName {
    match elongation.rem_euclid(360.0) {
        d if d < 90.0 => PhaseName::WaxingCrescent,
        d if d < 180.0 => PhaseName::WaxingGibbous,
        d if d < 270.0 => PhaseName::WaningGibbous,
        _ => PhaseName::WaningCrescent,
    }
}

/// The principal phase reached between two elongations, if any.
///
/// `start` and `end` are elongations at the beginning and end of an interval
/// short enough that the Moon cannot pass two principal phases within it - any
/// interval under seven days.
///
/// Only which phase, never when. The instant of a syzygy is found by bracketing
/// and Brent refinement (D-016); interpolating it linearly across a whole civil
/// day from these two endpoints disagreed with that by up to 3.6 minutes, which
/// is visible at the resolution the app prints.
pub fn principal_phase_in(start: f64, end: f64) -> Option<PhaseName> {
    let start = start.rem_euclid(360.0);
    let travelled = (end - start).rem_euclid(360.0);

    if travelled == 0.0 {
        return None;
    }

    PRINCIPAL_PHASES
        .iter()
        .filter_map(|&(name, target)| {
            let to_target = (target - start).rem_euclid(360.0);
            // `to_target == 0` means the interval begins exactly on the phase,
            // which belongs to this interval; the far endpoint belongs to the
            // next one, so the comparison is exclusive at the top.
            (to_target < travelled).then_some((name, to_target))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(name, _)| name)
}

/// True while the illuminated fraction is growing.
pub fn is_waxing(elongation: f64) -> bool {
    elongation.rem_euclid(360.0) < 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intermediate_never_names_a_principal_phase() {
        let mut d = 0.0;
        while d < 360.0 {
            assert!(
                !intermediate_phase(d).is_principal(),
                "elongation {d} named a principal phase"
            );
            d += 0.25;
        }
    }

    #[test]
    fn intermediate_covers_each_quadrant() {
        assert_eq!(intermediate_phase(1.0), PhaseName::WaxingCrescent);
        assert_eq!(intermediate_phase(89.9), PhaseName::WaxingCrescent);
        assert_eq!(intermediate_phase(90.1), PhaseName::WaxingGibbous);
        assert_eq!(intermediate_phase(179.9), PhaseName::WaxingGibbous);
        assert_eq!(intermediate_phase(180.1), PhaseName::WaningGibbous);
        assert_eq!(intermediate_phase(269.9), PhaseName::WaningGibbous);
        assert_eq!(intermediate_phase(270.1), PhaseName::WaningCrescent);
        assert_eq!(intermediate_phase(359.9), PhaseName::WaningCrescent);
    }

    #[test]
    fn principal_phase_detected_inside_a_day() {
        // A day spanning full moon: 178 -> 190 degrees.
        assert_eq!(
            principal_phase_in(178.0, 190.0),
            Some(PhaseName::FullMoon),
            "full moon inside"
        );

        // A day spanning new moon across the wraparound: 354 -> 6 degrees.
        assert_eq!(
            principal_phase_in(354.0, 6.0),
            Some(PhaseName::NewMoon),
            "new moon inside"
        );

        // The nearest phase wins when an interval could reach two: an interval
        // long enough to hold both must still name the one it reaches first.
        assert_eq!(
            principal_phase_in(88.0, 182.0),
            Some(PhaseName::FirstQuarter)
        );
    }

    #[test]
    fn no_principal_phase_when_the_day_falls_between_them() {
        assert!(principal_phase_in(100.0, 112.0).is_none());
        assert!(principal_phase_in(200.0, 212.0).is_none());
    }

    #[test]
    fn a_phase_belongs_to_exactly_one_day() {
        // Consecutive days must not both claim the same principal phase.
        let boundaries = [166.0, 178.0, 190.0, 202.0];
        let claims: Vec<_> = boundaries
            .windows(2)
            .filter_map(|w| principal_phase_in(w[0], w[1]))
            .collect();
        assert_eq!(
            claims.len(),
            1,
            "full moon claimed by {} days",
            claims.len()
        );
    }

    #[test]
    fn interval_starting_exactly_on_a_phase_owns_it() {
        assert_eq!(
            principal_phase_in(180.0, 192.0),
            Some(PhaseName::FullMoon),
            "owns the boundary"
        );
        // The preceding interval must not also claim it.
        assert!(principal_phase_in(168.0, 180.0).is_none());
    }

    #[test]
    fn waxing_matches_the_first_half_of_the_cycle() {
        assert!(is_waxing(0.0));
        assert!(is_waxing(179.9));
        assert!(!is_waxing(180.0));
        assert!(!is_waxing(359.9));
    }
}
