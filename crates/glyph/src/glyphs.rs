//! The nine graha glyphs, authored as SVG path data on a 24 x 24 unit grid.
//!
//! Path data and the constants below come from `docs/DESIGN.md` section 7.2 and
//! are reproduced here verbatim. They are drawings, not arithmetic, so they live
//! as data and are validated by test rather than being rebuilt in code.

use chandra_ephemeris::Graha;

/// The grid the glyphs are drawn on.
pub const DESIGN_GRID: f32 = 24.0;

/// Stroke width in design units.
pub const STROKE_WIDTH: f32 = 1.8;

/// How a glyph's path is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// Outlined at [`STROKE_WIDTH`] with round caps and joins.
    Stroke,
    /// Filled with the nonzero rule. Only Chandra.
    Fill,
}

/// Path data and ink mode for a graha.
///
/// Chandra's glyph is used in the settings list only: in the menu bar the live
/// phase disc always takes its place.
pub fn glyph(graha: Graha) -> (&'static str, Ink) {
    match graha {
        Graha::Surya => (SURYA, Ink::Stroke),
        Graha::Chandra => (CHANDRA, Ink::Fill),
        Graha::Mangala => (MANGALA, Ink::Stroke),
        Graha::Budha => (BUDHA, Ink::Stroke),
        Graha::Guru => (GURU, Ink::Stroke),
        Graha::Shukra => (SHUKRA, Ink::Stroke),
        Graha::Shani => (SHANI, Ink::Stroke),
        Graha::Rahu => (RAHU, Ink::Stroke),
        Graha::Ketu => (KETU, Ink::Stroke),
    }
}

/// Ring plus a centre dot. The dot is a radius-0.9 circle whose interior
/// collapses to nothing once stroked at 1.8, giving a solid disc of diameter
/// 3.6 from the same stroke pass. It must not be filled separately.
const SURYA: &str = "M 3.6,12 C 3.6,7.36 7.36,3.6 12,3.6 C 16.64,3.6 20.4,7.36 20.4,12 \
C 20.4,16.64 16.64,20.4 12,20.4 C 7.36,20.4 3.6,16.64 3.6,12 Z \
M 11.1,12 C 11.1,11.5 11.5,11.1 12,11.1 C 12.5,11.1 12.9,11.5 12.9,12 \
C 12.9,12.5 12.5,12.9 12,12.9 C 11.5,12.9 11.1,12.5 11.1,12 Z";

const CHANDRA: &str = "M 14.74,3.43 C 11.03,2.24 6.99,3.57 4.71,6.72 C 2.43,9.87 2.43,14.13 4.71,17.28 \
C 6.99,20.43 11.03,21.76 14.74,20.57 C 10.26,20.23 6.8,16.49 6.8,12 C 6.8,7.51 10.26,3.77 14.74,3.43 Z";

const MANGALA: &str = "M 3.8,14.6 C 3.8,11.51 6.31,9 9.4,9 C 12.49,9 15,11.51 15,14.6 \
C 15,17.69 12.49,20.2 9.4,20.2 C 6.31,20.2 3.8,17.69 3.8,14.6 Z \
M 13.36,10.64 L 19.4,4.6 M 14,4.6 L 19.4,4.6 L 19.4,10";

const BUDHA: &str = "M 8.7,4.8 C 8.7,6.62 10.18,8.1 12,8.1 C 13.82,8.1 15.3,6.62 15.3,4.8 \
M 7.8,12.3 C 7.8,9.98 9.68,8.1 12,8.1 C 14.32,8.1 16.2,9.98 16.2,12.3 \
C 16.2,14.62 14.32,16.5 12,16.5 C 9.68,16.5 7.8,14.62 7.8,12.3 Z \
M 12,16.5 L 12,20.4 M 7.8,18.3 L 16.2,18.3";

const GURU: &str = "M 7.2,9 C 7.2,5.4 10.5,3.9 13.2,5.1 C 16.2,6.45 16.05,10.2 14.1,12.6 \
C 12.15,15 9.6,17.4 8.4,19.2 L 19.2,19.2 M 13.5,12.6 L 13.5,20.4";

const SHUKRA: &str = "M 7.2,9.6 C 7.2,6.95 9.35,4.8 12,4.8 C 14.65,4.8 16.8,6.95 16.8,9.6 \
C 16.8,12.25 14.65,14.4 12,14.4 C 9.35,14.4 7.2,12.25 7.2,9.6 Z \
M 12,14.4 L 12,20.4 M 8.4,17.7 L 15.6,17.7";

const SHANI: &str = "M 5.1,7.2 L 11.1,7.2 M 8.1,3.6 L 8.1,20.4 \
M 8.1,14.7 C 8.7,11.4 12,10.2 14.4,11.7 C 16.8,13.2 17.1,16.8 15.6,20.4";

const RAHU: &str = "M 6,10.8 C 6,7.49 8.69,4.8 12,4.8 C 15.31,4.8 18,7.49 18,10.8 \
M 6,10.8 L 6,16.2 C 6,18.6 4.8,19.8 3.3,19.5 \
M 18,10.8 L 18,16.2 C 18,18.6 19.2,19.8 20.7,19.5";

const KETU: &str = "M 6,13.2 C 6,16.51 8.69,19.2 12,19.2 C 15.31,19.2 18,16.51 18,13.2 \
M 6,13.2 L 6,7.8 C 6,5.4 4.8,4.2 3.3,4.5 \
M 18,13.2 L 18,7.8 C 18,5.4 19.2,4.2 20.7,4.5";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path;

    #[test]
    fn every_graha_has_a_parseable_glyph() {
        for graha in Graha::ALL {
            let (data, _) = glyph(graha);
            path::parse(data).unwrap_or_else(|e| panic!("{} glyph: {e}", graha.name()));
        }
    }

    #[test]
    fn no_glyph_uses_an_arc_command() {
        // The renderer has no arc support by design; an arc would fail to parse
        // rather than render wrongly, but catching it here names the glyph.
        for graha in Graha::ALL {
            let (data, _) = glyph(graha);
            assert!(
                !data.contains(['A', 'a']),
                "{} glyph contains an arc command",
                graha.name()
            );
        }
    }

    #[test]
    fn only_chandra_is_filled() {
        for graha in Graha::ALL {
            let (_, ink) = glyph(graha);
            let expected = if graha == Graha::Chandra {
                Ink::Fill
            } else {
                Ink::Stroke
            };
            assert_eq!(ink, expected, "{} ink mode", graha.name());
        }
    }

    /// The inked bounds recorded in `docs/DESIGN.md` section 7.2, as
    /// `(x_min, y_min, x_max, y_max, cap_height)` in design units with the
    /// stroke included. Asserting against the published table means the drawing
    /// and its specification cannot drift apart unnoticed.
    const DOCUMENTED_BOUNDS: [(Graha, f32, f32, f32, f32, f32); 9] = [
        (Graha::Surya, 2.70, 2.70, 21.30, 21.30, 18.60),
        (Graha::Chandra, 3.00, 3.00, 14.74, 21.00, 18.00),
        (Graha::Mangala, 2.90, 3.70, 20.30, 21.10, 17.40),
        (Graha::Budha, 6.90, 3.90, 17.10, 21.30, 17.40),
        (Graha::Guru, 6.30, 3.80, 20.10, 21.30, 17.50),
        (Graha::Shukra, 6.30, 3.90, 17.70, 21.30, 17.40),
        (Graha::Shani, 4.20, 2.70, 17.42, 21.30, 18.60),
        (Graha::Rahu, 2.40, 3.90, 21.60, 20.44, 16.54),
        (Graha::Ketu, 2.40, 3.56, 21.60, 20.10, 16.54),
    ];

    /// Inked bounds: the drawn geometry grown by half the stroke width, which is
    /// what actually lands on screen.
    fn inked_bounds(graha: Graha) -> (f32, f32, f32, f32) {
        let (data, ink) = glyph(graha);
        let bounds = path::parse(data)
            .expect("parses")
            .compute_tight_bounds()
            .expect("has geometry");
        let margin = match ink {
            Ink::Stroke => STROKE_WIDTH / 2.0,
            Ink::Fill => 0.0,
        };
        (
            bounds.left() - margin,
            bounds.top() - margin,
            bounds.right() + margin,
            bounds.bottom() + margin,
        )
    }

    #[test]
    fn ink_matches_the_published_design_table() {
        for (graha, x_min, y_min, x_max, y_max, cap) in DOCUMENTED_BOUNDS {
            let (left, top, right, bottom) = inked_bounds(graha);
            let close = |actual: f32, documented: f32, label: &str| {
                assert!(
                    (actual - documented).abs() < 0.02,
                    "{} {label}: drawn {actual:.2}, documented {documented:.2}",
                    graha.name()
                );
            };
            close(left, x_min, "x-min");
            close(top, y_min, "y-min");
            close(right, x_max, "x-max");
            close(bottom, y_max, "y-max");
            close(bottom - top, cap, "cap-height");
        }
    }

    #[test]
    fn ink_stays_inside_the_design_grid() {
        for graha in Graha::ALL {
            let (left, top, right, bottom) = inked_bounds(graha);
            assert!(left >= -0.01, "{} overflows the left edge", graha.name());
            assert!(top >= -0.01, "{} overflows the top edge", graha.name());
            assert!(
                right <= DESIGN_GRID + 0.01,
                "{} overflows the right edge",
                graha.name()
            );
            assert!(
                bottom <= DESIGN_GRID + 0.01,
                "{} overflows the bottom edge",
                graha.name()
            );
        }
    }

    #[test]
    fn cap_heights_stay_within_the_documented_optical_spread() {
        // docs/DESIGN.md 7.2 records a deliberate spread of 16.54 to 18.60
        // units: flat-terminated forms read short and wide forms read large, so
        // holding every glyph to one height would look uneven. Outside that
        // range is drift rather than intent.
        for graha in Graha::ALL {
            let (_, top, _, bottom) = inked_bounds(graha);
            let cap = bottom - top;
            assert!(
                (16.5..=18.65).contains(&cap),
                "{} cap-height {cap:.2} is outside the documented spread",
                graha.name()
            );
        }
    }
}
