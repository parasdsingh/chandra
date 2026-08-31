//! Dumps real almanac output for the front-end visual harness.
//!
//!     cargo run -p chandra --example preview_data > src/dev/fixture.json
//!
//! The harness renders the panel components against this, so what is inspected
//! is the real data the app produces rather than invented sample values. A
//! layout that only works for tidy fixtures is not verified at all.

use std::path::PathBuf;

use chandra_almanac::day::DayOptions;
use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::{Almanac, Location, MonthCursor};
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, Observer, SiderealConfig};
use serde_json::json;

/// Every optional limb on.
///
/// The harness is what the panel's layout is inspected against, and a pane that
/// is only ever rendered with its optional rows absent is a pane whose full
/// height nobody has looked at.
fn limbs() -> DayOptions {
    DayOptions {
        yogas: true,
        karanas: true,
        muhurtas: true,
    }
}

/// The place every chart in the harness is cast for, matching the observer the
/// almanac above is configured with. A chart drawn for one place and captioned
/// with another would be the exact defect the caption exists to prevent.
const PLACE: &str = "Bengaluru";

/// A local-clock instant as the milliseconds the front end is served.
fn instant(year: i32, month: u32, day: u32, day_fraction: f64) -> i64 {
    (chandra_ephemeris::jd_to_unix_seconds(chandra_ephemeris::julian_day(
        year,
        month,
        day,
        day_fraction,
    )) * 1000.0) as i64
}

/// The most crowded compartment of 2026.
///
/// Found rather than hardcoded, for the same reason the moonless day is: a date
/// picked once by hand stops being the densest conjunction as soon as anything
/// about the ephemeris changes, and then the harness quietly draws an easy case
/// while claiming to draw the hard one.
fn crowded(almanac: &Almanac) -> chandra_almanac::chakra::Chakra {
    (1..=365)
        .map(|day| {
            almanac
                .chakra(instant(2026, 1, 1, 0.5) + day * 86_400_000, PLACE)
                .expect("crowded scan")
        })
        .max_by_key(|chart| {
            chart
                .rashis
                .iter()
                .map(|rashi| rashi.grahas.len())
                .max()
                .unwrap_or(0)
        })
        .expect("a chart in the scan")
}

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

    let cursor = |year: i32, month: u32, system: MonthSystem| MonthCursor {
        anchor_unix_ms: (chandra_ephemeris::jd_to_unix_seconds(chandra_ephemeris::julian_day(
            year, month, 15, 0.25,
        )) * 1000.0) as i64,
        offset: 0,
        system,
        first_weekday: 0,
    };

    let document = json!({
        "timeZone": "Asia/Kolkata",
        "grahas": chandra_lib::graha_info(),
        "moonMonth": almanac
            .moon_month(cursor(2026, 8, MonthSystem::Solar))
            .expect("moon month"),
        // The same stretch of sky as a lunar month, to check the label and the
        // way the grid handles a month that starts mid-week and mid-Gregorian-month.
        "lunarMonth": almanac
            .moon_month(cursor(2026, 8, MonthSystem::Amanta))
            .expect("lunar month"),
        "moonDay": almanac
            .day_detail(Graha::Chandra, date(20), limbs())
            .expect("moon day"),
        // The same day in a lunar month, which carries the panchanga block.
        "lunarDay": almanac
            .day_detail(Graha::Chandra, date(20), limbs())
            .expect("lunar day"),
        // The Moon rises about 50 minutes later each day, so roughly one civil
        // day a month contains no moonrise at all. Found rather than hardcoded,
        // so the harness always has a real example of the degraded state.
        "moonDayNoRise": (1..=31)
            .filter_map(|day| {
                almanac
                    .day_detail(Graha::Chandra, date(day), limbs())
                    .ok()
            })
            .find(|detail| match detail {
                chandra_almanac::month::DayDetail::Moon(moon) => moon.moonrise.is_none(),
                _ => false,
            })
            .expect("a month contains a day with no moonrise"),
        // February 2025: Mangala is retrograde and turns direct mid-month, so the
        // grid shows a span rule, station markers and ingresses together.
        "grahaMonth": almanac
            .graha_month(Graha::Mangala, cursor(2025, 2, MonthSystem::Solar))
            .expect("graha month"),
        "grahaDay": almanac
            .day_detail(
                Graha::Mangala,
                chandra_almanac::time::DateKey::new(2025, 2, 24).expect("date"),
                limbs(),
            )
            .expect("graha day"),
        // A graha in a lunar month: the one view shape the harness did not cover.
        // Shani is retrograde on this date, so it carries the tithi block, the
        // panchanga and the retrograde state at once.
        "lunarGrahaDay": almanac
            .day_detail(Graha::Shani, date(21), limbs())
            .expect("lunar graha day"),
        // Before 1800, to exercise the reduced-precision note.
        "moshierDay": almanac
            .day_detail(
                Graha::Chandra,
                chandra_almanac::time::DateKey::new(1650, 8, 20).expect("date"),
                limbs(),
            )
            .expect("moshier day"),
        "snapshot": almanac
            .now(1_755_000_000_000, &[Graha::Mangala])
            .expect("snapshot"),
        // An ordinary chart. Every format draws this same reading, because the
        // three formats differ in where a rashi is put on screen and not in
        // what is true.
        "chakra": almanac
            .chakra(instant(2026, 8, 20, 0.72), PLACE)
            .expect("chakra"),
        // The layout's worst case, searched for rather than chosen: `cluster`
        // has a branch for a row that will not fit its compartment at any width,
        // and a chart with the grahas spread evenly never reaches it. Whatever
        // the densest conjunction of the year is, the harness draws it.
        "chakraCrowded": crowded(&almanac),
        // Before 1800, where the ephemeris falls back to the analytic model and
        // the pane has to say so.
        "chakraMoshier": almanac
            .chakra(instant(1650, 8, 20, 0.72), PLACE)
            .expect("moshier chakra"),
    });

    println!("{}", serde_json::to_string_pretty(&document).expect("json"));
}
