//! The live moon disc.
//!
//! Geometry follows `docs/DESIGN.md` section 7.1. The lit region is a single
//! closed path: the limb semicircle on the lit side plus the terminator
//! half-ellipse, each built from two cubic quarter-arcs.

use tiny_skia::{Path, PathBuilder};

/// Handle length as a fraction of the radius for a cubic quarter-circle. The
/// classic value; maximum radial error is about 0.02%, well under a pixel at
/// any menu bar size.
const KAPPA: f32 = 0.5523;

/// Below this illuminated fraction only the ring is drawn.
pub const NEW_MOON_THRESHOLD: f64 = 0.02;
/// Above this the disc is solid and the ring is omitted.
pub const FULL_MOON_THRESHOLD: f64 = 0.98;

/// Ring alpha away from new moon.
///
/// Near-opaque rather than the 0.55 first drawn. macOS renders a template image
/// by masking with its own foreground colour, so a half-transparent ring comes
/// out as grey next to fully opaque system icons and the moon reads as a
/// different, dimmer class of thing.
pub const RING_ALPHA: f32 = 0.9;
/// Ring alpha at new moon, where the ring is the only thing drawn and carries
/// the whole click target.
pub const NEW_MOON_RING_ALPHA: f32 = 1.0;

/// What to draw for a given illumination.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rendering {
    /// New moon: ring only, no lit region.
    RingOnly { alpha: f32 },
    /// Full moon: solid disc, no ring.
    SolidDisc,
    /// Everything between: lit region plus ring.
    LitAndRing { alpha: f32 },
}

pub fn rendering_for(illumination: f64) -> Rendering {
    if illumination < NEW_MOON_THRESHOLD {
        Rendering::RingOnly {
            alpha: NEW_MOON_RING_ALPHA,
        }
    } else if illumination > FULL_MOON_THRESHOLD {
        Rendering::SolidDisc
    } else {
        Rendering::LitAndRing { alpha: RING_ALPHA }
    }
}

/// The lit region of the disc.
///
/// `illumination` is the illuminated fraction. `waxing` puts the lit side on the
/// right, as seen from the northern hemisphere. `southern` mirrors the whole
/// form, which is what an observer south of the equator actually sees.
pub fn lit_region(
    centre_x: f32,
    centre_y: f32,
    radius: f32,
    illumination: f64,
    waxing: bool,
    southern: bool,
) -> Option<Path> {
    // Signed semi-minor axis of the terminator ellipse. Positive for a crescent,
    // where the terminator bows into the lit side; negative for a gibbous, where
    // it bows across into the dark side. Zero at half, giving a straight edge.
    let terminator = radius * (1.0 - 2.0 * illumination as f32);

    // One sign flip covers both waning and the southern hemisphere, and applying
    // both leaves the orientation unchanged, which is correct: a waning moon
    // seen from the south looks like a waxing moon seen from the north.
    let side = if waxing != southern { 1.0 } else { -1.0 };

    let mut builder = PathBuilder::new();
    builder.move_to(centre_x, centre_y - radius);

    // Limb: the true circular edge, on the lit side.
    quarter_arc(
        &mut builder,
        centre_x,
        centre_y,
        (0.0, -radius),
        (side * radius, 0.0),
    );
    quarter_arc(
        &mut builder,
        centre_x,
        centre_y,
        (side * radius, 0.0),
        (0.0, radius),
    );

    // Terminator: back to the top along the ellipse.
    quarter_arc(
        &mut builder,
        centre_x,
        centre_y,
        (0.0, radius),
        (side * terminator, 0.0),
    );
    quarter_arc(
        &mut builder,
        centre_x,
        centre_y,
        (side * terminator, 0.0),
        (0.0, -radius),
    );

    builder.close();
    builder.finish()
}

/// The full disc outline, used for the hairline ring.
pub fn disc(centre_x: f32, centre_y: f32, radius: f32) -> Option<Path> {
    let mut builder = PathBuilder::new();
    builder.push_circle(centre_x, centre_y, radius);
    builder.finish()
}

/// Appends one cubic quarter-arc between two points that lie on the axes of an
/// ellipse centred at the origin.
///
/// Exactly one of the two points is on the x axis and the other on the y axis,
/// so the handle for each runs along the axis of the other, scaled by KAPPA.
fn quarter_arc(
    builder: &mut PathBuilder,
    centre_x: f32,
    centre_y: f32,
    from: (f32, f32),
    to: (f32, f32),
) {
    let control_from = (from.0 + to.0 * KAPPA, from.1 + to.1 * KAPPA);
    let control_to = (to.0 + from.0 * KAPPA, to.1 + from.1 * KAPPA);

    builder.cubic_to(
        centre_x + control_from.0,
        centre_y + control_from.1,
        centre_x + control_to.0,
        centre_y + control_to.1,
        centre_x + to.0,
        centre_y + to.1,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const CENTRE: f32 = 11.0;
    const RADIUS: f32 = 8.0;

    /// Tight bounds of the drawn curve. `Path::bounds` returns the control
    /// point hull, which for a Bezier is a strict overestimate and would let a
    /// real overflow pass unnoticed while failing on shapes that are fine.
    fn region(illumination: f64, waxing: bool) -> tiny_skia::Rect {
        lit_region(CENTRE, CENTRE, RADIUS, illumination, waxing, false)
            .expect("path")
            .compute_tight_bounds()
            .expect("has geometry")
    }

    #[test]
    fn thresholds_select_the_right_rendering() {
        assert!(matches!(rendering_for(0.0), Rendering::RingOnly { .. }));
        assert!(matches!(rendering_for(0.019), Rendering::RingOnly { .. }));
        assert!(matches!(rendering_for(0.021), Rendering::LitAndRing { .. }));
        assert!(matches!(rendering_for(0.5), Rendering::LitAndRing { .. }));
        assert!(matches!(rendering_for(0.979), Rendering::LitAndRing { .. }));
        assert!(matches!(rendering_for(1.0), Rendering::SolidDisc));
    }

    #[test]
    fn the_lit_region_never_leaves_the_disc() {
        for step in 0..=100 {
            let illumination = step as f64 / 100.0;
            for waxing in [true, false] {
                let bounds = region(illumination, waxing);
                assert!(
                    bounds.left() >= CENTRE - RADIUS - 0.01
                        && bounds.right() <= CENTRE + RADIUS + 0.01
                        && bounds.top() >= CENTRE - RADIUS - 0.01
                        && bounds.bottom() <= CENTRE + RADIUS + 0.01,
                    "illumination {illumination} waxing {waxing} escaped the disc: {bounds:?}"
                );
            }
        }
    }

    #[test]
    fn a_full_moon_covers_the_whole_disc() {
        let bounds = region(1.0, true);
        assert!((bounds.left() - (CENTRE - RADIUS)).abs() < 0.01);
        assert!((bounds.right() - (CENTRE + RADIUS)).abs() < 0.01);
        assert!((bounds.top() - (CENTRE - RADIUS)).abs() < 0.01);
        assert!((bounds.bottom() - (CENTRE + RADIUS)).abs() < 0.01);
    }

    #[test]
    fn a_half_moon_covers_exactly_one_side() {
        let waxing = region(0.5, true);
        assert!(
            (waxing.left() - CENTRE).abs() < 0.01,
            "a waxing half moon must start at the centre line, got {}",
            waxing.left()
        );
        assert!((waxing.right() - (CENTRE + RADIUS)).abs() < 0.01);

        let waning = region(0.5, false);
        assert!((waning.left() - (CENTRE - RADIUS)).abs() < 0.01);
        assert!(
            (waning.right() - CENTRE).abs() < 0.01,
            "a waning half moon must end at the centre line"
        );
    }

    #[test]
    fn waxing_lights_the_right_and_waning_the_left() {
        // A crescent spans from the centre line to the limb. Its cusps sit
        // exactly on the centre line at the poles, because both the limb and the
        // terminator pass through those two points, so the bound touches the
        // centre rather than clearing it.
        let waxing = region(0.15, true);
        assert!(
            (waxing.left() - CENTRE).abs() < 0.01,
            "a waxing crescent's cusps sit on the centre line, got {}",
            waxing.left()
        );
        assert!((waxing.right() - (CENTRE + RADIUS)).abs() < 0.01);

        let waning = region(0.15, false);
        assert!((waning.left() - (CENTRE - RADIUS)).abs() < 0.01);
        assert!(
            (waning.right() - CENTRE).abs() < 0.01,
            "a waning crescent's cusps sit on the centre line"
        );
    }

    #[test]
    fn the_terminator_sits_where_the_geometry_says_it_should() {
        // The strongest check available on the curve: the widest point of the
        // terminator must be at R(1-2k) from the centre, on the correct side.
        for illumination in [0.1, 0.25, 0.4, 0.6, 0.75, 0.9] {
            let expected = RADIUS * (1.0 - 2.0 * illumination as f32);
            let bounds = region(illumination, true);

            if illumination < 0.5 {
                // Crescent: the terminator bows into the lit side, so it forms
                // the left edge at CENTRE + a.
                assert!(
                    (bounds.left() - CENTRE).abs() < 0.01,
                    "crescent cusps at {illumination}"
                );
            } else {
                // Gibbous: the terminator bows across, so it forms the left edge
                // at CENTRE + a with a negative.
                assert!(
                    (bounds.left() - (CENTRE + expected)).abs() < 0.02,
                    "gibbous terminator at {illumination}: got {}, want {}",
                    bounds.left(),
                    CENTRE + expected
                );
            }
        }
    }

    #[test]
    fn a_gibbous_moon_crosses_the_centre_line() {
        let bounds = region(0.75, true);
        assert!(bounds.left() < CENTRE, "a gibbous moon passes the centre");
        assert!((bounds.right() - (CENTRE + RADIUS)).abs() < 0.01);
    }

    #[test]
    fn the_southern_hemisphere_sees_the_mirror_image() {
        let northern = region(0.15, true);
        let southern = lit_region(CENTRE, CENTRE, RADIUS, 0.15, true, true)
            .expect("path")
            .compute_tight_bounds()
            .expect("has geometry");

        // Mirrored about the centre line: the left inset of one equals the right
        // inset of the other.
        assert!(
            ((northern.left() - CENTRE) + (southern.right() - CENTRE)).abs() < 0.01,
            "northern {northern:?} and southern {southern:?} are not mirror images"
        );
    }

    #[test]
    fn a_waning_southern_moon_matches_a_waxing_northern_one() {
        // Two sign flips cancel, which is what an observer actually sees.
        let a = lit_region(CENTRE, CENTRE, RADIUS, 0.3, false, true).expect("path");
        let b = lit_region(CENTRE, CENTRE, RADIUS, 0.3, true, false).expect("path");
        assert_eq!(a.compute_tight_bounds(), b.compute_tight_bounds());
    }
}
