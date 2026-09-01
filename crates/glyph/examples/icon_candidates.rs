//! The menu bar icons, magnified, as they ship and as a 1x display gets them.
//!
//!     cargo run -q -p chandra-glyph --example icon_candidates -- <out.png>
//!
//! Renders at the scale the app uses and then magnifies with nearest neighbour,
//! so what is inspected is the pixels the menu bar gets rather than a smooth
//! re-render at a size nobody sees. The halved column is what macOS produces on
//! a 1x display, where a design that only works at 2x falls apart.

use chandra_glyph::{chart_icon, graha_icon, moon_icon, Tint};
use tiny_skia::{Paint, Pixmap, Rect, Transform};

const SLOT: u32 = 22;
const SCALE: u32 = 2;
const ZOOM: u32 = 6;

fn from_icon(icon: chandra_glyph::Icon) -> Pixmap {
    Pixmap::from_vec(
        icon.rgba,
        tiny_skia::IntSize::from_wh(icon.width, icon.height).expect("size"),
    )
    .expect("pixmap")
}

/// Half size, box filtered: what a 1x display is handed.
fn halved(source: &Pixmap) -> Pixmap {
    let mut out = Pixmap::new(source.width() / 2, source.height() / 2).expect("half");
    for y in 0..out.height() {
        for x in 0..out.width() {
            let mut total = [0u32; 4];
            for dy in 0..2 {
                for dx in 0..2 {
                    let pixel = source
                        .pixel(x * 2 + dx, y * 2 + dy)
                        .expect("pixel")
                        .demultiply();
                    total[0] += u32::from(pixel.red());
                    total[1] += u32::from(pixel.green());
                    total[2] += u32::from(pixel.blue());
                    total[3] += u32::from(pixel.alpha());
                }
            }
            let mut block = Paint::default();
            block.set_color_rgba8(
                (total[0] / 4) as u8,
                (total[1] / 4) as u8,
                (total[2] / 4) as u8,
                (total[3] / 4) as u8,
            );
            out.fill_rect(
                Rect::from_xywh(x as f32, y as f32, 1.0, 1.0).expect("cell"),
                &block,
                Transform::identity(),
                None,
            );
        }
    }
    out
}

fn main() {
    let out = std::env::args().nth(1).expect("an output path");
    let tint = Tint::Colour {
        r: 237,
        g: 237,
        b: 239,
    };

    let chart = from_icon(chart_icon(SCALE, tint).expect("chart"));
    let moon = from_icon(moon_icon(0.35, true, false, SCALE, tint).expect("moon"));
    let guru =
        from_icon(graha_icon(chandra_ephemeris::Graha::Guru, SCALE, tint, false).expect("guru"));

    let plates: Vec<(&str, Pixmap)> = vec![
        ("chart", chart.clone()),
        ("chart halved", halved(&chart)),
        ("moon", moon.clone()),
        ("moon halved", halved(&moon)),
        ("guru", guru.clone()),
        ("guru halved", halved(&guru)),
    ];

    let cell = SLOT * SCALE * ZOOM;
    let gap = 8;
    let mut sheet = Pixmap::new(
        cell * plates.len() as u32 + gap * (plates.len() as u32 + 1),
        cell + gap * 2,
    )
    .expect("sheet");

    for (index, (name, pixmap)) in plates.iter().enumerate() {
        println!("{index}: {name}");
        let origin_x = gap + (cell + gap) * index as u32;
        let zoom = cell / pixmap.width();
        for y in 0..pixmap.height() {
            for x in 0..pixmap.width() {
                let source = pixmap.pixel(x, y).expect("pixel");
                if source.alpha() == 0 {
                    continue;
                }
                let source = source.demultiply();
                let mut block = Paint::default();
                block.set_color_rgba8(source.red(), source.green(), source.blue(), source.alpha());
                sheet.fill_rect(
                    Rect::from_xywh(
                        (origin_x + x * zoom) as f32,
                        (gap + y * zoom) as f32,
                        zoom as f32,
                        zoom as f32,
                    )
                    .expect("block"),
                    &block,
                    Transform::identity(),
                    None,
                );
            }
        }
    }

    sheet.save_png(&out).expect("write png");
    println!("wrote {out}");
}
