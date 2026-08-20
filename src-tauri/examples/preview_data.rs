//! Dumps real almanac output for the front-end visual harness.
//!
//!     cargo run -p chandra --example preview_data > src/dev/fixture.json
//!
//! The harness renders the panel components against this, so what is inspected
//! is the real data the app produces rather than invented sample values. A
//! layout that only works for tidy fixtures is not verified at all.

use std::path::PathBuf;

use chandra_almanac::{Almanac, Location};
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, Observer, SiderealConfig};
use serde_json::json;

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/ephe");
    let almanac = Almanac::new(
        &path,
        Location {
            observer: Observer::new(12.9716, 77.5946, 920.0),
            zone_name: "Asia/Kolkata".into(),
        },
        SiderealConfig {
            ayanamsa: Ayanamsa::Lahiri,
            node_type: NodeType::True,
        },
    )
    .expect("almanac");

    let date = |day| chandra_almanac::time::DateKey::new(2026, 8, day).expect("date");

    let grahas: Vec<_> = Graha::ALL
        .into_iter()
        .map(|graha| {
            let (path, ink) = chandra_glyph::glyphs::glyph(graha);
            json!({
                "key": graha.key(),
                "name": graha.name(),
                "english": graha.english(),
                "path": path,
                "filled": ink == chandra_glyph::glyphs::Ink::Fill,
                "stroke_width": chandra_glyph::glyphs::STROKE_WIDTH,
            })
        })
        .collect();

    let document = json!({
        "timeZone": "Asia/Kolkata",
        "grahas": grahas,
        "moonMonth": almanac.moon_month(2026, 8).expect("moon month"),
        "moonDay": almanac.day_detail(Graha::Chandra, date(20)).expect("moon day"),
        // The Moon rises about 50 minutes later each day, so roughly one civil
        // day a month contains no moonrise at all. Found rather than hardcoded,
        // so the harness always has a real example of the degraded state.
        "moonDayNoRise": (1..=31)
            .filter_map(|day| almanac.day_detail(Graha::Chandra, date(day)).ok())
            .find(|detail| match detail {
                chandra_almanac::month::DayDetail::Moon(moon) => moon.moonrise.is_none(),
                _ => false,
            })
            .expect("a month contains a day with no moonrise"),
        // February 2025: Mangala is retrograde and turns direct mid-month, so the
        // grid shows a span rule, station markers and ingresses together.
        "grahaMonth": almanac
            .graha_month(Graha::Mangala, 2025, 2)
            .expect("graha month"),
        "grahaDay": almanac
            .day_detail(
                Graha::Mangala,
                chandra_almanac::time::DateKey::new(2025, 2, 24).expect("date"),
            )
            .expect("graha day"),
        // Before 1800, to exercise the reduced-precision note.
        "moshierDay": almanac
            .day_detail(
                Graha::Chandra,
                chandra_almanac::time::DateKey::new(1650, 8, 20).expect("date"),
            )
            .expect("moshier day"),
        "snapshot": almanac
            .now(1_755_000_000_000, &[Graha::Mangala])
            .expect("snapshot"),
    });

    println!("{}", serde_json::to_string_pretty(&document).expect("json"));
}
