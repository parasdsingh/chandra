//! Rasterisation to the RGBA buffer a tray icon is set from.

use chandra_ephemeris::Graha;
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, Pixmap, Stroke, Transform};

use crate::glyphs::{self, Ink, DESIGN_GRID, RETROGRADE_STROKE, STROKE_WIDTH};
use crate::moon::{self, Rendering};
use crate::path;

/// Menu bar slot height in points. macOS gives a status item a 22pt square.
pub const SLOT_POINTS: f32 = 22.0;

/// Moon disc radius in points.
///
/// 7.2 rather than 8.0: a 14.4pt disc fills 65% of the 22pt slot, matching the
/// optical size of the system status icons it sits beside. At 8.0 the moon read
/// visibly larger and heavier than the wifi and battery glyphs next to it.
const DISC_RADIUS_POINTS: f32 = 7.2;
/// Ring stroke in points, centred on the radius.
///
/// Matched to the ~1.3pt strokes macOS uses for its own status icons, so the
/// outline carries the same weight as its neighbours.
const RING_STROKE_POINTS: f32 = 1.3;

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("glyph path: {0}")]
    Path(#[from] path::PathError),

    #[error("cannot allocate a {width}x{height} pixmap")]
    Allocation { width: u32, height: u32 },

    /// A path builder produced nothing. Only reachable if a shape was described
    /// with no segments, which for the shapes in this crate would be a bug
    /// rather than an input the caller can fix.
    #[error("the {0} path is empty")]
    EmptyPath(&'static str),
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

    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };

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

/// Size the graha glyph shrinks to when a retrograde mark is drawn beside it.
///
/// Small enough to free the lower right corner, large enough that the symbol is
/// still the thing the eye lands on. The mark annotates the glyph; it does not
/// share the slot with it.
const GLYPH_WITH_MARK_POINTS: f32 = 16.5;

/// Size of the retrograde mark inside the slot.
const MARK_POINTS: f32 = 10.0;

/// Clearance between the mark's ink and the edge of the slot.
///
/// The mark is placed by its ink, not by its nominal box: `℞` fills a little
/// over half its own design grid, so an origin computed from the box left it
/// 1.8pt short of the corner it is meant to occupy - and hard against the glyph
/// beside it. Guru's baseline bar and Rahu's right tail both ran into the mark's
/// top bar, and the two rendered as a single shape rather than as a symbol with
/// an annotation.
const MARK_CLEARANCE: f32 = 0.8;

/// Renders a graha's template glyph, marked `℞` while it is retrograde.
///
/// Retrograde is the one state the menu bar carries. It is the rarest and the
/// most watched, and at 22 points one mark is all that stays legible; everything
/// else a graha can be doing is named in the calendar, where there is room for
/// words.
/// The Lagna Kundali's menu bar icon.
///
/// The North Indian chart's construction reduced to what survives at 22 points:
/// the outer square, the midpoint diamond filled solid, and the part of each
/// diagonal that crosses a corner triangle.
///
/// The fill is what makes it hold up. Every outline version of this mark reads
/// thin beside the moon, which is a filled disc - and worse, an outline that
/// carries the whole construction turns to a grey blur when macOS maps the 2x
/// buffer onto a 1x slot, because eight strokes in 22 points is more line than
/// there is room for. A solid middle halves cleanly. The diagonals are what stop
/// the result reading as a lozenge in a box; they are drawn only in the corner
/// triangles, since inside the diamond they would be the colour of the fill.
///
/// Rejected, and why:
///
/// | Candidate | Why not |
/// |---|---|
/// | Square and diamond, no diagonals | What shipped. Reads as a generic mark rather than a chart |
/// | The full twelve-compartment construction | Legible at 2x, a grey blob once halved |
/// | Square and diagonals, no diamond | Reads as a cross through a box, which is the shape of a cancel button |
/// | The South Indian frame | Its broken grid reads as noise at this size |
/// | Corner triangles filled, middle open | Reads as an octagon, and the chart is not one |
///
/// Static. The chart behind it changes constantly and none of that is legible at
/// this size; what changes is in the tooltip, which is rebuilt on hover.
pub fn chart_icon(scale: u32, tint: Tint) -> Result<Icon, RenderError> {
    let size = SLOT_POINTS as u32 * scale;
    let mut pixmap = Pixmap::new(size, size).ok_or(RenderError::Allocation {
        width: size,
        height: size,
    })?;

    let (r, g, b) = tint.components();
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color_rgba8(r, g, b, u8::MAX);

    // The weight a graha glyph ends up at, reproduced.
    //
    // This mark is built in pixels while a graha is built in design-grid units
    // and scaled, and tiny-skia scales the stroke with the transform. Passing
    // `STROKE_WIDTH` to a path already in pixels therefore drew this at
    // `STROKE_WIDTH` *pixels* - 1.8 against a graha's 3.3 at 2x, a little over
    // half the weight of every glyph beside it. That is why the mark looked
    // faint, and no amount of redrawing the shape would have fixed it.
    let weight = STROKE_WIDTH * (SLOT_POINTS / DESIGN_GRID) * scale as f32;

    let inset = SLOT_POINTS * CHART_INSET * scale as f32;
    let edge = size as f32 - inset * 2.0;
    let far = size as f32 - inset;
    let middle = size as f32 / 2.0;

    let mut square = tiny_skia::PathBuilder::new();
    square.push_rect(tiny_skia::Rect::from_xywh(inset, inset, edge, edge).ok_or(
        RenderError::Allocation {
            width: size,
            height: size,
        },
    )?);
    pixmap.stroke_path(
        &square
            .finish()
            .ok_or(RenderError::EmptyPath("chart square"))?,
        &paint,
        &stroke_of(weight),
        Transform::identity(),
        None,
    );

    // Held off the square by a stroke's width, so the two shapes stay two
    // shapes. Touching, they merged into one silhouette at 1x.
    let clearance = weight * 1.15;
    let mut diamond = tiny_skia::PathBuilder::new();
    diamond.move_to(middle, inset + clearance);
    diamond.line_to(far - clearance, middle);
    diamond.line_to(middle, far - clearance);
    diamond.line_to(inset + clearance, middle);
    diamond.close();
    pixmap.fill_path(
        &diamond
            .finish()
            .ok_or(RenderError::EmptyPath("chart diamond"))?,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    // Half way from each corner to the centre, which is where the diamond's edge
    // is: any further and the stub disappears into the fill.
    let mut stubs = tiny_skia::PathBuilder::new();
    for (x, y) in [(inset, inset), (far, inset), (far, far), (inset, far)] {
        stubs.move_to(x, y);
        stubs.line_to(x + (middle - x) * 0.5, y + (middle - y) * 0.5);
    }
    pixmap.stroke_path(
        &stubs
            .finish()
            .ok_or(RenderError::EmptyPath("chart diagonals"))?,
        &paint,
        &stroke_of(weight * 0.8),
        Transform::identity(),
        None,
    );

    Ok(Icon {
        rgba: pixmap.take(),
        width: size,
        height: size,
    })
}

/// Clearance between the chart mark and the edge of its slot.
///
/// Tighter than the 0.18 the outline version used. A filled shape reads smaller
/// than an outline of the same size, because the outline's ink is all at the
/// edge where it defines the silhouette.
const CHART_INSET: f32 = 0.12;

fn stroke_of(width: f32) -> Stroke {
    Stroke {
        width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    }
}

pub fn graha_icon(
    graha: Graha,
    scale: u32,
    tint: Tint,
    retrograde: bool,
) -> Result<Icon, RenderError> {
    let size = SLOT_POINTS as u32 * scale;
    let mut pixmap = Pixmap::new(size, size).ok_or(RenderError::Allocation {
        width: size,
        height: size,
    })?;

    let (data, ink) = glyphs::glyph(graha);
    let path = path::parse(data)?;

    // Centred on the glyph's own ink, not on the design grid it was drawn in.
    // Several glyphs sit off-centre in that grid, so pinning the grid to the
    // slot put the symbol itself off-centre in the menu bar.
    let (box_x, box_y, _, _) = glyphs::view_box(graha);

    // The 24 unit design grid maps onto the slot, then onto pixels. A marked
    // glyph is drawn smaller and pinned to the top left, which is where the
    // corner it gives up is.
    let glyph_points = if retrograde {
        GLYPH_WITH_MARK_POINTS
    } else {
        SLOT_POINTS
    };
    let unit_scale = glyph_points / DESIGN_GRID * scale as f32;
    let transform = Transform::from_scale(unit_scale, unit_scale).pre_translate(-box_x, -box_y);

    let (r, g, b) = tint.components();
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color_rgba8(r, g, b, u8::MAX);

    match ink {
        Ink::Fill => {
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
        Ink::Stroke => {
            pixmap.stroke_path(&path, &paint, &glyph_stroke(), transform, None);
        }
    }

    if retrograde {
        let mark = path::parse(glyphs::RETROGRADE)?;
        let mark_ink = mark
            .compute_tight_bounds()
            .ok_or(path::PathError::Empty)?
            .outset(RETROGRADE_STROKE / 2.0, RETROGRADE_STROKE / 2.0)
            .ok_or(path::PathError::Empty)?;

        let mark_scale = MARK_POINTS / DESIGN_GRID * scale as f32;
        let corner = (SLOT_POINTS - MARK_CLEARANCE) * scale as f32;
        let placement = Transform::from_scale(mark_scale, mark_scale).post_translate(
            corner - mark_ink.right() * mark_scale,
            corner - mark_ink.bottom() * mark_scale,
        );
        pixmap.stroke_path(&mark, &paint, &mark_stroke(), placement, None);
    }

    Ok(to_icon(pixmap))
}

fn glyph_stroke() -> Stroke {
    Stroke {
        width: STROKE_WIDTH,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    }
}

fn mark_stroke() -> Stroke {
    Stroke {
        width: RETROGRADE_STROKE,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    }
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

        // A 14.4pt disc in a 22pt square covers pi*7.2^2/22^2 = 33.6%.
        assert!((full - 0.336).abs() < 0.01, "full disc covered {full}");
        // Half the lit disc plus the ring around the dark half.
        assert!((0.22..0.30).contains(&half), "half moon covered {half}");
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
            let icon = graha_icon(graha, 2, Tint::Template, false).expect("render");
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

    /// Regions of ink connected to each other, at 8-connectivity.
    ///
    /// Ink measured inside a corner box cannot tell an annotation from a blob:
    /// two shapes that touch still put ink in the corner, and total coverage
    /// says less still, because a shrunk glyph loses more area than the mark
    /// adds. Whether the mark is a separate shape is the actual question, so it
    /// is the one asked.
    fn components(icon: &Icon) -> usize {
        // Any pixel the rasteriser touched at all, so a single bridging pixel
        // of anti-aliasing is a failure rather than a rounding detail.
        const INK: u8 = 0;

        let width = icon.width as i64;
        let height = icon.height as i64;
        let inked: Vec<bool> = icon.rgba.chunks_exact(4).map(|p| p[3] > INK).collect();
        let mut seen = vec![false; inked.len()];
        let mut found = 0;

        for start in 0..inked.len() {
            if !inked[start] || seen[start] {
                continue;
            }
            found += 1;
            seen[start] = true;

            let mut pending = vec![start];
            while let Some(index) = pending.pop() {
                let (x, y) = ((index as i64) % width, (index as i64) / width);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= width || ny >= height {
                            continue;
                        }
                        let neighbour = (ny * width + nx) as usize;
                        if inked[neighbour] && !seen[neighbour] {
                            seen[neighbour] = true;
                            pending.push(neighbour);
                        }
                    }
                }
            }
        }
        found
    }

    /// The retrograde mark must annotate the glyph, not merge with it.
    ///
    /// A menu bar icon has 22 points and one job: say which graha this is. The
    /// mark is an annotation on that, drawn in the corner the glyph gives up by
    /// shrinking, and it has to read as a second shape rather than as a growth
    /// on the first.
    #[test]
    fn the_retrograde_mark_is_a_separate_shape_beside_the_glyph() {
        for graha in Graha::ALL {
            let plain = graha_icon(graha, 2, Tint::Template, false).expect("render");
            let marked = graha_icon(graha, 2, Tint::Template, true).expect("render");

            assert_ne!(plain.rgba, marked.rgba, "{} is unmarked", graha.name());
            assert_eq!((marked.width, marked.height), (44, 44));

            assert_eq!(
                components(&marked),
                components(&plain) + 1,
                "{}: the mark and the glyph rasterise as one shape",
                graha.name()
            );
            assert!(
                (0.03..0.60).contains(&coverage(&marked)),
                "{}: marked icon covers {} of the slot",
                graha.name(),
                coverage(&marked)
            );

            // Still a template image: macOS inverts these by alpha, and any
            // colour left in the RGB channels would survive as a tint.
            for pixel in marked.rgba.chunks_exact(4) {
                assert_eq!([pixel[0], pixel[1], pixel[2]], [0, 0, 0]);
            }

            // Nothing of the mark falls outside the slot: the outermost row and
            // column must stay clear, or macOS clips it against its neighbour.
            for index in 0..44u32 {
                let edge = |x: u32, y: u32| marked.rgba[((y * 44 + x) * 4 + 3) as usize];
                assert_eq!(
                    edge(43, index),
                    0,
                    "{}: ink on the right edge",
                    graha.name()
                );
                assert_eq!(
                    edge(index, 43),
                    0,
                    "{}: ink on the bottom edge",
                    graha.name()
                );
            }
        }
    }

    #[test]
    fn template_icons_carry_shape_in_alpha_only() {
        // macOS inverts a template image using its alpha channel; any colour in
        // the RGB channels would survive inversion and show as a tint.
        let icon = graha_icon(Graha::Shani, 2, Tint::Template, false).expect("render");
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
            false,
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
            let icon = graha_icon(graha, 2, Tint::Template, false).expect("render");
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
