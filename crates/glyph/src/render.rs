//! Rasterisation to the RGBA buffer a tray icon is set from.

use chandra_ephemeris::Graha;
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, Pixmap, Stroke, Transform};

use crate::glyphs::{self, Ink, DESIGN_GRID, STROKE_WIDTH};
use crate::moon::{self, Rendering};
use crate::path;

/// Menu bar slot height in points. macOS gives a status item a 22pt square.
pub const SLOT_POINTS: f32 = 22.0;

/// Moon disc radius in points: a 16pt disc fills 73% of the slot, which reads as
/// a moon rather than as a dot or a plate.
const DISC_RADIUS_POINTS: f32 = 8.0;
/// Ring stroke in points, centred on the radius.
const RING_STROKE_POINTS: f32 = 1.0;

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("glyph path: {0}")]
    Path(#[from] path::PathError),

    #[error("cannot allocate a {width}x{height} pixmap")]
    Allocation { width: u32, height: u32 },
}

/// A rendered icon: straight (non-premultiplied) RGBA, ready for
/// `tauri::image::Image::new_owned`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Ink colour for an icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tint {
    /// Opaque black, varying only in alpha. macOS inverts a template image to
    /// match the menu bar, so this is the mode that works in both appearances.
    Template,
    /// A fixed colour, for the optional colour mode.
    Colour { r: u8, g: u8, b: u8 },
}

impl Tint {
    fn components(self) -> (u8, u8, u8) {
        match self {
            Tint::Template => (0, 0, 0),
            Tint::Colour { r, g, b } => (r, g, b),
        }
    }
}

/// Renders the moon phase disc.
///
/// `scale` is the backing scale factor: 2 on a Retina display, giving a 44x44
/// buffer for the 22pt slot.
pub fn moon_icon(
    illumination: f64,
    waxing: bool,
    southern: bool,
    scale: u32,
    tint: Tint,
) -> Result<Icon, RenderError> {
    let size = SLOT_POINTS as u32 * scale;
    let mut pixmap = Pixmap::new(size, size).ok_or(RenderError::Allocation {
        width: size,
        height: size,
    })?;

    let centre = SLOT_POINTS / 2.0;
    let transform = Transform::from_scale(scale as f32, scale as f32);
    let (r, g, b) = tint.components();

    let mut paint = Paint::default();
    paint.anti_alias = true;

    match moon::rendering_for(illumination) {
        Rendering::RingOnly { alpha } => {
            paint.set_color_rgba8(r, g, b, to_u8(alpha));
            if let Some(ring) = moon::disc(centre, centre, DISC_RADIUS_POINTS) {
                pixmap.stroke_path(&ring, &paint, &ring_stroke(), transform, None);
            }
        }
        Rendering::SolidDisc => {
            paint.set_color_rgba8(r, g, b, u8::MAX);
            if let Some(disc) = moon::disc(centre, centre, DISC_RADIUS_POINTS) {
                pixmap.fill_path(&disc, &paint, FillRule::Winding, transform, None);
            }
        }
        Rendering::LitAndRing { alpha } => {
            paint.set_color_rgba8(r, g, b, u8::MAX);
            if let Some(lit) = moon::lit_region(
                centre,
                centre,
                DISC_RADIUS_POINTS,
                illumination,
                waxing,
                southern,
            ) {
                pixmap.fill_path(&lit, &paint, FillRule::Winding, transform, None);
            }

            paint.set_color_rgba8(r, g, b, to_u8(alpha));
            if let Some(ring) = moon::disc(centre, centre, DISC_RADIUS_POINTS) {
                pixmap.stroke_path(&ring, &paint, &ring_stroke(), transform, None);
            }
        }
    }

    Ok(to_icon(pixmap))
}

/// Renders a graha's template glyph.
pub fn graha_icon(graha: Graha, scale: u32, tint: Tint) -> Result<Icon, RenderError> {
    let size = SLOT_POINTS as u32 * scale;
    let mut pixmap = Pixmap::new(size, size).ok_or(RenderError::Allocation {
        width: size,
        height: size,
    })?;

    let (data, ink) = glyphs::glyph(graha);
    let path = path::parse(data)?;

    // The 24 unit design grid maps onto the 22pt slot, then onto pixels.
    let unit_scale = SLOT_POINTS / DESIGN_GRID * scale as f32;
    let transform = Transform::from_scale(unit_scale, unit_scale);

    let (r, g, b) = tint.components();
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color_rgba8(r, g, b, u8::MAX);

    match ink {
        Ink::Fill => {
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
        Ink::Stroke => {
            let stroke = Stroke {
                width: STROKE_WIDTH,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Stroke::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
        }
    }

    Ok(to_icon(pixmap))
}

fn ring_stroke() -> Stroke {
    Stroke {
        width: RING_STROKE_POINTS,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    }
}

fn to_u8(alpha: f32) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// `tiny-skia` composites in premultiplied alpha; the tray API expects straight
/// RGBA. Converting here keeps that detail out of the application layer.
fn to_icon(pixmap: Pixmap) -> Icon {
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for pixel in pixmap.pixels() {
        let straight = pixel.demultiply();
        rgba.extend_from_slice(&[
            straight.red(),
            straight.green(),
            straight.blue(),
            straight.alpha(),
        ]);
    }

    Icon {
        width,
        height,
        rgba,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Alpha-weighted ink coverage.
    ///
    /// Counting pixels above a threshold instead would score every
    /// anti-aliased edge pixel as fully covered, inflating a circle's area by
    /// its whole perimeter - about 6% of the slot at this size.
    fn coverage(icon: &Icon) -> f64 {
        let ink: f64 = icon
            .rgba
            .chunks_exact(4)
            .map(|pixel| pixel[3] as f64 / 255.0)
            .sum();
        ink / (icon.width * icon.height) as f64
    }

    /// Fill ink either side of the vertical centre line.
    ///
    /// Only near-opaque pixels count, which excludes the ring: the ring
    /// surrounds the dark half too, so including it would report a crescent as
    /// nearly symmetric.
    fn halves(icon: &Icon) -> (usize, usize) {
        let mid = icon.width / 2;
        let mut left = 0;
        let mut right = 0;
        for (index, pixel) in icon.rgba.chunks_exact(4).enumerate() {
            if pixel[3] < 250 {
                continue;
            }
            let x = index as u32 % icon.width;
            if x < mid {
                left += 1;
            } else if x > mid {
                right += 1;
            }
        }
        (left, right)
    }

    #[test]
    fn icons_have_the_expected_buffer_size() {
        let icon = moon_icon(0.5, true, false, 2, Tint::Template).expect("render");
        assert_eq!((icon.width, icon.height), (44, 44));
        assert_eq!(icon.rgba.len(), 44 * 44 * 4);

        let icon = moon_icon(0.5, true, false, 1, Tint::Template).expect("render");
        assert_eq!((icon.width, icon.height), (22, 22));
    }

    #[test]
    fn a_new_moon_is_still_a_visible_click_target() {
        // The whole reason the ring exists: with no ring a new moon renders zero
        // pixels and the menu bar item cannot be hit.
        let icon = moon_icon(0.0, true, false, 2, Tint::Template).expect("render");
        let covered = coverage(&icon);
        assert!(
            covered > 0.05,
            "a new moon must still draw its ring, covered {covered}"
        );
        assert!(
            covered < 0.35,
            "a new moon must not look like a lit disc, covered {covered}"
        );
    }

    #[test]
    fn illumination_increases_coverage_monotonically_in_the_lit_range() {
        let mut previous = 0.0;
        for step in 2..=98 {
            let illumination = step as f64 / 100.0;
            let icon = moon_icon(illumination, true, false, 2, Tint::Template).expect("render");
            let covered = coverage(&icon);
            assert!(
                covered >= previous - 0.005,
                "coverage fell from {previous} to {covered} at illumination {illumination}"
            );
            previous = covered;
        }
    }

    #[test]
    fn a_full_moon_is_a_solid_disc_and_a_half_moon_is_half_of_one() {
        let full = coverage(&moon_icon(1.0, true, false, 2, Tint::Template).expect("render"));
        let half = coverage(&moon_icon(0.5, true, false, 2, Tint::Template).expect("render"));

        // A 16pt disc in a 22pt square covers pi*8^2/22^2 = 41.5%.
        assert!((full - 0.415).abs() < 0.01, "full disc covered {full}");
        // Half the lit disc plus the ring: about 21% plus the ring's own area.
        assert!((0.24..0.32).contains(&half), "half moon covered {half}");
        assert!(half < full, "a half moon must cover less than a full one");
    }

    #[test]
    fn the_lit_side_follows_the_phase() {
        let (left, right) =
            halves(&moon_icon(0.2, true, false, 2, Tint::Template).expect("render"));
        assert!(
            right > left * 2,
            "a waxing crescent must be mostly right of centre: {left} vs {right}"
        );

        let (left, right) =
            halves(&moon_icon(0.2, false, false, 2, Tint::Template).expect("render"));
        assert!(
            left > right * 2,
            "a waning crescent must be mostly left of centre: {left} vs {right}"
        );
    }

    #[test]
    fn the_southern_hemisphere_flips_the_lit_side() {
        let (north_left, north_right) =
            halves(&moon_icon(0.2, true, false, 2, Tint::Template).expect("render"));
        let (south_left, south_right) =
            halves(&moon_icon(0.2, true, true, 2, Tint::Template).expect("render"));

        assert!(north_right > north_left);
        assert!(south_left > south_right);
    }

    #[test]
    fn every_graha_renders_visible_distinguishable_ink() {
        let mut signatures = Vec::new();
        for graha in Graha::ALL {
            let icon = graha_icon(graha, 2, Tint::Template).expect("render");
            assert_eq!((icon.width, icon.height), (44, 44));

            let covered = coverage(&icon);
            assert!(
                (0.03..0.60).contains(&covered),
                "{} covers {covered} of the slot",
                graha.name()
            );
            signatures.push((graha, icon.rgba));
        }

        // No two glyphs may rasterise identically, or the menu bar would show
        // the same mark for two different grahas.
        for (index, (graha, rgba)) in signatures.iter().enumerate() {
            for (other, other_rgba) in &signatures[index + 1..] {
                assert_ne!(
                    rgba,
                    other_rgba,
                    "{} and {} render identically",
                    graha.name(),
                    other.name()
                );
            }
        }
    }

    #[test]
    fn template_icons_carry_shape_in_alpha_only() {
        // macOS inverts a template image using its alpha channel; any colour in
        // the RGB channels would survive inversion and show as a tint.
        let icon = graha_icon(Graha::Shani, 2, Tint::Template).expect("render");
        for pixel in icon.rgba.chunks_exact(4) {
            assert_eq!(
                [pixel[0], pixel[1], pixel[2]],
                [0, 0, 0],
                "a template pixel must be black"
            );
        }
    }

    #[test]
    fn colour_mode_preserves_the_requested_colour() {
        let icon = graha_icon(
            Graha::Shani,
            2,
            Tint::Colour {
                r: 237,
                g: 237,
                b: 239,
            },
        )
        .expect("render");

        let opaque = icon
            .rgba
            .chunks_exact(4)
            .find(|p| p[3] == 255)
            .expect("some pixel is fully opaque");
        assert_eq!([opaque[0], opaque[1], opaque[2]], [237, 237, 239]);
    }

    #[test]
    fn ink_stays_inside_the_slot() {
        for graha in Graha::ALL {
            let icon = graha_icon(graha, 2, Tint::Template).expect("render");
            let edge_ink = icon
                .rgba
                .chunks_exact(4)
                .enumerate()
                .filter(|(index, pixel)| {
                    let x = *index as u32 % icon.width;
                    let y = *index as u32 / icon.width;
                    pixel[3] > 8
                        && (x == 0 || y == 0 || x == icon.width - 1 || y == icon.height - 1)
                })
                .count();
            assert_eq!(
                edge_ink,
                0,
                "{} touches the edge of the slot and would be clipped",
                graha.name()
            );
        }
    }
}
