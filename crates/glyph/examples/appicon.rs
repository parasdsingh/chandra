//! Generates the application icon.
//!
//!     cargo run -p chandra-glyph --example appicon -- src-tauri/icons
//!
//! The icon is the same moon geometry the menu bar draws, so the app's icon and
//! its menu bar item cannot drift apart. A waxing gibbous is used rather than a
//! full disc: a full moon renders as a plain circle, which says nothing.

use std::path::PathBuf;

use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// macOS icons sit inside a rounded square with a margin the system expects.
const CANVAS: f32 = 1024.0;
const SQUIRCLE_INSET: f32 = 100.0;
const SQUIRCLE_RADIUS: f32 = 185.0;
const DISC_RADIUS: f32 = 300.0;
const ILLUMINATION: f64 = 0.72;

fn main() {
    let out_dir: PathBuf = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "src-tauri/icons".into())
        .into();
    std::fs::create_dir_all(&out_dir).expect("icon directory");

    let size = CANVAS as u32;
    let mut pixmap = Pixmap::new(size, size).expect("pixmap");

    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };

    // Ground: the panel's own background, so icon and panel share a palette.
    paint.set_color_rgba8(0x0A, 0x0A, 0x0B, 255);
    let squircle = rounded_rect(
        SQUIRCLE_INSET,
        SQUIRCLE_INSET,
        CANVAS - SQUIRCLE_INSET,
        CANVAS - SQUIRCLE_INSET,
        SQUIRCLE_RADIUS,
    );
    pixmap.fill_path(
        &squircle,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    let centre = CANVAS / 2.0;

    // Ring, at the same relative weight as the menu bar disc.
    paint.set_color_rgba8(0xED, 0xED, 0xEF, 90);
    if let Some(ring) = chandra_glyph::moon::disc(centre, centre, DISC_RADIUS) {
        pixmap.stroke_path(
            &ring,
            &paint,
            &Stroke {
                width: DISC_RADIUS / 8.0,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Stroke::default()
            },
            Transform::identity(),
            None,
        );
    }

    paint.set_color_rgba8(0xED, 0xED, 0xEF, 255);
    if let Some(lit) =
        chandra_glyph::moon::lit_region(centre, centre, DISC_RADIUS, ILLUMINATION, true, false)
    {
        pixmap.fill_path(&lit, &paint, FillRule::Winding, Transform::identity(), None);
    }

    let png = pixmap.encode_png().expect("encode");
    let path = out_dir.join("icon.png");
    std::fs::write(&path, png).expect("write icon");
    println!("wrote {}", path.display());
}

fn rounded_rect(left: f32, top: f32, right: f32, bottom: f32, radius: f32) -> tiny_skia::Path {
    const KAPPA: f32 = 0.5523;
    let handle = radius * (1.0 - KAPPA);

    let mut builder = PathBuilder::new();
    builder.move_to(left + radius, top);
    builder.line_to(right - radius, top);
    builder.cubic_to(right - handle, top, right, top + handle, right, top + radius);
    builder.line_to(right, bottom - radius);
    builder.cubic_to(
        right,
        bottom - handle,
        right - handle,
        bottom,
        right - radius,
        bottom,
    );
    builder.line_to(left + radius, bottom);
    builder.cubic_to(
        left + handle,
        bottom,
        left,
        bottom - handle,
        left,
        bottom - radius,
    );
    builder.line_to(left, top + radius);
    builder.cubic_to(left, top + handle, left + handle, top, left + radius, top);
    builder.close();
    builder.finish().expect("rounded rect")
}
