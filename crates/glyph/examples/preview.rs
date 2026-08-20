//! Renders every icon as ASCII art for eyeballing without a build of the app.
//!
//!     cargo run -p chandra-glyph --example preview

use chandra_ephemeris::Graha;
use chandra_glyph::{graha_icon, moon_icon, Icon, Tint};

const RAMP: [char; 5] = [' ', '.', ':', '#', '@'];

fn draw(icon: &Icon, columns: u32) {
    let step = icon.width / columns;
    for row in (0..icon.height).step_by(step as usize) {
        let mut line = String::new();
        for column in (0..icon.width).step_by(step as usize) {
            let index = ((row * icon.width + column) * 4 + 3) as usize;
            let alpha = icon.rgba[index] as usize;
            line.push(RAMP[alpha * (RAMP.len() - 1) / 255]);
        }
        println!("  {line}");
    }
}

fn main() {
    for (label, illumination, waxing) in [
        ("new", 0.00, true),
        ("waxing crescent", 0.25, true),
        ("first quarter", 0.50, true),
        ("waxing gibbous", 0.75, true),
        ("full", 1.00, true),
        ("waning gibbous", 0.75, false),
        ("last quarter", 0.50, false),
        ("waning crescent", 0.25, false),
    ] {
        println!("\n{label}");
        draw(
            &moon_icon(illumination, waxing, false, 2, Tint::Template).expect("render"),
            22,
        );
    }

    for graha in Graha::ALL {
        println!("\n{}", graha.name());
        draw(&graha_icon(graha, 2, Tint::Template).expect("render"), 22);
    }
}
