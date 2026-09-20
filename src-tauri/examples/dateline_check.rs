//! What the calendars do in a month a zone has no complete set of dates for.
//!
//!     cargo run -q --release -p chandra --example dateline_check
//!
//! Three zones crossed the international date line and skipped a calendar date
//! outright: Kiritimati has no 1994-12-31, Apia no 2011-12-30, Kwajalein no
//! 1993-08-21. `grid_days` draws that date anyway, as a day of zero length,
//! because leaving it out moves every later cell one column off its weekday.
//! This is the layer above: whether a month containing a zero-length day still
//! assembles, and what the day itself reports.

use std::path::PathBuf;

use chandra_almanac::day::DayOptions;
use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::time::DateKey;
use chandra_almanac::{Almanac, Location, MonthCursor};
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, Observer, SiderealConfig};

struct Skip {
    zone: &'static str,
    latitude: f64,
    longitude: f64,
    /// The date that never happened there.
    missing: (i16, i8, i8),
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/ephe");

    let skips = [
        Skip {
            zone: "Pacific/Kiritimati",
            latitude: 1.87,
            longitude: -157.43,
            missing: (1994, 12, 31),
        },
        Skip {
            zone: "Pacific/Apia",
            latitude: -13.83,
            longitude: -171.77,
            missing: (2011, 12, 30),
        },
        Skip {
            zone: "Pacific/Kwajalein",
            latitude: 8.72,
            longitude: 167.73,
            missing: (1993, 8, 21),
        },
    ];

    for skip in skips {
        let almanac = Almanac::new(
            &path,
            Location {
                observer: Observer::new(skip.latitude, skip.longitude, 0.0),
                zone_name: skip.zone.into(),
            },
            SiderealConfig {
                ayanamsa: Ayanamsa::Lahiri,
                node_type: NodeType::Mean,
            },
        )
        .expect("almanac");

        let (year, month, day) = skip.missing;
        println!("\n=== {} (no {year}-{month:02}-{day:02}) ===", skip.zone);

        // Local noon on the first of that month, as the anchor the panel would
        // send when it is showing it.
        let anchor = (chandra_ephemeris::jd_to_unix_seconds(chandra_ephemeris::julian_day(
            i32::from(year),
            u32::try_from(month).expect("month"),
            1,
            0.5,
        )) * 1000.0) as i64;

        for system in [
            MonthSystem::Solar,
            MonthSystem::Amanta,
            MonthSystem::Purnimanta,
        ] {
            let cursor = MonthCursor {
                anchor_unix_ms: anchor,
                offset: 0,
                system,
                first_weekday: 0,
            };
            match almanac.moon_month(cursor) {
                Ok(month) => println!(
                    "  {system:?} moon:  {} cells, {}",
                    month.days.len(),
                    month.label
                ),
                Err(e) => println!("  {system:?} moon:  ERROR {e}"),
            }
            match almanac.graha_month(Graha::Shani, cursor) {
                Ok(month) => println!(
                    "  {system:?} shani: {} cells, {}",
                    month.days.len(),
                    month.label
                ),
                Err(e) => println!("  {system:?} shani: ERROR {e}"),
            }
        }

        let date = DateKey::new(year, month, day).expect("date");
        let limbs = DayOptions {
            yogas: true,
            karanas: true,
            muhurtas: true,
        };
        match almanac.day_detail(Graha::Chandra, date, limbs) {
            Ok(detail) => println!("  the missing day opens: {detail:?}"),
            Err(e) => println!("  the missing day opens: ERROR {e}"),
        }
    }
}
