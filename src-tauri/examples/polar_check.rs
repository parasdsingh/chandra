//! What the app does at the latitudes where the sun does not rise.
//!
//!     cargo run -q --release -p chandra --example polar_check
//!
//! The ascendant is guarded to +/-89 by `crates/ephemeris/tests/ascendant.rs`.
//! What is untested is the layer above: whether a chart and a day can be read at
//! a polar place, and what the day does in a month with no sunrise in it.

use std::path::PathBuf;

use chandra_almanac::day::DayOptions;
use chandra_almanac::time::DateKey;
use chandra_almanac::{Almanac, Location};
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, Observer, SiderealConfig};

struct Place {
    label: &'static str,
    zone: &'static str,
    latitude: f64,
    longitude: f64,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/ephe");

    // Both are in the shipped zone.tab, so both are reachable from the settings
    // pane today. Longyearbyen has polar night around the winter solstice and
    // midnight sun around the summer one; Vostok has the same in the opposite
    // months, and is the southernmost place the table carries.
    let places = [
        Place {
            label: "Longyearbyen",
            zone: "Arctic/Longyearbyen",
            latitude: 78.0,
            longitude: 16.0,
        },
        Place {
            label: "Vostok",
            zone: "Antarctica/Vostok",
            latitude: -78.4,
            longitude: 106.9,
        },
    ];

    // Midwinter, midsummer and an equinox at each: the two solstices are where
    // the sun neither rises nor sets, and the equinox is the control.
    let dates: [(i16, i8, i8); 3] = [(2026, 6, 21), (2026, 12, 21), (2026, 3, 20)];

    for place in places {
        let almanac = Almanac::new(
            &path,
            Location {
                observer: Observer::new(place.latitude, place.longitude, 0.0),
                zone_name: place.zone.into(),
            },
            SiderealConfig {
                ayanamsa: Ayanamsa::Lahiri,
                node_type: NodeType::Mean,
            },
        )
        .expect("almanac");

        println!("\n=== {} ({:.1} lat) ===", place.label, place.latitude);

        for (year, month, day) in dates {
            let date = DateKey::new(year, month, day).expect("date");
            let noon = chandra_ephemeris::jd_to_unix_seconds(chandra_ephemeris::julian_day(
                i32::from(year),
                u32::try_from(month).expect("month"),
                u32::try_from(day).expect("day"),
                0.5,
            )) * 1000.0;

            match almanac.chakra(noon as i64, place.label) {
                Ok(chart) => println!(
                    "  {year}-{month:02}-{day:02} chart: lagna {} {}deg",
                    chart.lagna.name, chart.lagna.degrees_in_rashi.0
                ),
                Err(e) => println!("  {year}-{month:02}-{day:02} chart: ERROR {e}"),
            }

            let limbs = DayOptions {
                yogas: true,
                karanas: true,
                muhurtas: true,
            };
            match almanac.day_detail(Graha::Chandra, date, limbs) {
                Ok(chandra_almanac::month::DayDetail::Moon(moon)) => println!(
                    "  {year}-{month:02}-{day:02} day:   moonrise {} moonset {} tithi {} muhurtas {}",
                    moon.moonrise.is_some(),
                    moon.moonset.is_some(),
                    moon.panchanga.tithis.first().map(|t| t.name.as_str()).unwrap_or("none"),
                    moon.panchanga.muhurtas.len(),
                ),
                Ok(_) => println!("  {year}-{month:02}-{day:02} day:   not the moon"),
                Err(e) => println!("  {year}-{month:02}-{day:02} day:   ERROR {e}"),
            }
        }
    }
}
