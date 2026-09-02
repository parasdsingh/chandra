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

/// The menu bar slot, from the crate that draws into it rather than copied.
const SLOT: u32 = chandra_glyph::SLOT_POINTS as u32;
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
    // The first argument that is not a flag. Taking `nth(1)` blindly meant
    // running this with `--halved` and no path wrote a PNG named `--halved` into
    // the working directory.
    let out = std::env::args()
        .skip(1)
        .find(|arg| !arg.starts_with("--"))
        .expect("an output path");
    let tint = Tint::Colour {
        r: 237,
        g: 237,
        b: 239,
    };

    let mut plates: Vec<(String, Pixmap)> = Vec::new();

    plates.push((
        "chart".into(),
        from_icon(chart_icon(SCALE, tint).expect("chart")),
    ));
    if std::env::args().any(|a| a == "--charts") {
        render(&out, plates);
        return;
    }

    plates.push((
        "moon".into(),
        from_icon(moon_icon(0.35, true, false, SCALE, tint).expect("moon")),
    ));
    for graha in chandra_ephemeris::Graha::ALL {
        plates.push((
            graha.name().to_string(),
            from_icon(graha_icon(graha, SCALE, tint, false).expect("graha")),
        ));
    }
    // Guru marked, which is the one state the menu bar carries: the glyph is
    // drawn smaller and pinned to the top left to make room for the mark.
    plates.push((
        "Guru retrograde".into(),
        from_icon(graha_icon(chandra_ephemeris::Graha::Guru, SCALE, tint, true).expect("marked")),
    ));

    if std::env::args().any(|arg| arg == "--halved") {
        plates = plates
            .into_iter()
            .map(|(name, pixmap)| (format!("{name} halved"), halved(&pixmap)))
            .collect();
    }

    // Optical weight, measured. A row of icons looks even when they carry
    // similar ink in similar space, not when their nominal boxes match - which
    // is why these are read off the pixels rather than off the design.
    if std::env::args().any(|arg| arg == "--measure") {
        println!(
            "{:<18} {:>9} {:>9} {:>9} {:>12}",
            "icon", "ink w x h", "coverage", "of slot", "off centre"
        );
        for (name, pixmap) in &plates {
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
            let mut ink = 0f64;
            for y in 0..pixmap.height() {
                for x in 0..pixmap.width() {
                    let alpha = pixmap.pixel(x, y).expect("pixel").alpha();
                    if alpha == 0 {
                        continue;
                    }
                    ink += f64::from(alpha) / 255.0;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
            let (w, h) = (x1 + 1 - x0, y1 + 1 - y0);
            let slot = f64::from(pixmap.width() * pixmap.height());
            // Where the ink actually sits against the middle of the slot. A
            // glyph centred on its design grid rather than on its own ink reads
            // as hanging off to one side in a row of them.
            let middle = f64::from(pixmap.width()) / 2.0;
            let dx = (f64::from(x0) + f64::from(x1) + 1.0) / 2.0 - middle;
            let dy = (f64::from(y0) + f64::from(y1) + 1.0) / 2.0 - middle;
            println!(
                "{name:<18} {:>4} x{:>4} {:>8.1}% {:>8.1}% {:>+5.1},{:>+5.1}",
                w,
                h,
                100.0 * ink / f64::from(w * h),
                100.0 * ink / slot,
                dx,
                dy
            );
        }
        return;
    }

    render(&out, plates);
}

fn render(out: &str, plates: Vec<(String, Pixmap)>) {
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

    sheet.save_png(out).expect("write png");
    println!("wrote {out}");
}
