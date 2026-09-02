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
use chandra_almanac::varga::Varga;
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

/// An instant as the milliseconds the front end is served.
///
/// `day_fraction` is a fraction of a *UT* day, not of a local one: 0.72 is 17:16
/// UT, which is 22:46 in Kolkata.
fn instant(year: i32, month: u32, day: u32, day_fraction: f64) -> i64 {
    (chandra_ephemeris::jd_to_unix_seconds(chandra_ephemeris::julian_day(
        year,
        month,
        day,
        day_fraction,
    )) * 1000.0) as i64
}

/// The most crowded compartment the ephemeris can produce.
///
/// Found rather than hardcoded, for the same reason the moonless day is: a date
/// picked once by hand stops being the densest conjunction as soon as anything
/// about the ephemeris changes, and then the harness quietly draws an easy case
/// while claiming to draw the hard one.
///
/// Scanned over two centuries rather than one year. A year's worst case is not
/// the drawing's worst case, and the drawing is what this is for. The ceiling is
/// eight: the seven grahas can all share a sign, and exactly one node can join
/// them because Rahu and Ketu are opposite by construction and so can never be
/// in the same sign as each other.
///
/// Occupancy does not depend on where the observer stands - only the lagna does
/// - so the place this is cast for does not affect what is found.
fn crowded(almanac: &Almanac) -> chandra_almanac::chakra::Chakra {
    (0..73_000)
        .map(|day| {
            almanac
                .chakra(
                    instant(1950, 1, 1, 0.5) + day * 86_400_000,
                    PLACE,
                    Varga::D1,
                )
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

/// The same conjunction, at the hour that puts it in a given house.
///
/// Which *compartment* a crowd lands in is the whole of the layout problem, and
/// the house is what decides it: in the North Indian chart house 1 is the top
/// kite, houses 2, 6, 8 and 12 are corner triangles, and 3, 5, 9 and 11 are
/// triangles against a wall. A kite is the roomiest shape on the chart and a
/// corner triangle the tightest.
///
/// The harness drew its crowd in house 7 - a kite - and called it the worst
/// case. It is the best one. Everything that later turned out to be broken about
/// crowded triangles was invisible for exactly that reason, so the hard shapes
/// are now asked for by name.
///
/// Houses rotate with the lagna, so the same instant's grahas visit every
/// compartment over a day: this scans the minutes of the conjunction's own day
/// for the one that puts the crowd where it is wanted.
fn crowded_in_house(almanac: &Almanac, wanted: u8) -> chandra_almanac::chakra::Chakra {
    let day = crowded(almanac).unix_ms;
    let midnight = day - day.rem_euclid(86_400_000);

    (0..1440)
        .map(|minute| {
            almanac
                .chakra(midnight + minute * 60_000, PLACE, Varga::D1)
                .expect("house scan")
        })
        .find(|chart| {
            chart
                .rashis
                .iter()
                .max_by_key(|rashi| rashi.grahas.len())
                .is_some_and(|rashi| rashi.house == wanted)
        })
        .expect("every house is reached over a day")
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

    // Polar night at Longyearbyen, where the Sun does not rise at all and a day
    // cannot be named after the tithi at sunrise.
    //
    // The observer is moved and moved back rather than a second `Almanac` being
    // built: Swiss Ephemeris is a process-global, so there is only ever one, and
    // asking for another returns `AlreadyConstructed`. Everything else in this
    // file is cast for Bengaluru and is computed after the move is undone.
    let polar_day = {
        let home = almanac.location().expect("home location");
        almanac
            .set_location(Location {
                observer: Observer::new(78.0, 16.0, 0.0),
                zone_name: "Arctic/Longyearbyen".into(),
            })
            .expect("move to the arctic");

        let day = almanac
            .day_detail(
                Graha::Chandra,
                chandra_almanac::time::DateKey::new(2026, 12, 21).expect("date"),
                limbs(),
            )
            .expect("polar day");

        almanac.set_location(home).expect("move home");
        day
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
        // Polar night at Longyearbyen, where the Sun does not rise at all and the
        // day cannot be named after the tithi at sunrise. Its own almanac
        // because the observer is what decides this, and everything else in the
        // harness is cast for Bengaluru.
        "polarDay": polar_day,
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
            .chakra(instant(2026, 8, 20, 0.72), PLACE, Varga::D1)
            .expect("chakra"),
        // The layout's worst case over two centuries, searched for rather than
        // chosen: `cluster`
        // has a branch for a row that will not fit its compartment at any width,
        // and a chart with the grahas spread evenly never reaches it. Whatever
        // the densest conjunction of the year is, the harness draws it.
        "chakraCrowded": crowded(&almanac),
        // The same eight bodies in a corner triangle and in a wall triangle,
        // which are the two shapes that actually constrain the layout.
        "chakraCorner": crowded_in_house(&almanac, 2),
        "chakraWall": crowded_in_house(&almanac, 3),
        // The same instant as `chakra`, in the navamsa. Drawn beside the rashi
        // chart in the harness, because the thing worth checking about a varga
        // is that it is a *different* chart from D1 and not that it renders.
        "chakraNavamsa": almanac
            .chakra(instant(2026, 8, 20, 0.72), PLACE, Varga::D9)
            .expect("navamsa"),
        // D2, which occupies two compartments out of twelve and can therefore
        // put all nine bodies in one. It is the hardest crowding case the app
        // can produce, and it is not rare - every chart is like this in D2.
        "chakraHora": almanac
            .chakra(instant(2026, 8, 20, 0.72), PLACE, Varga::D2)
            .expect("hora"),
        // D30, the unequal division, which reaches ten signs and never Karka or
        // Simha.
        "chakraTrimsamsa": almanac
            .chakra(instant(2026, 8, 20, 0.72), PLACE, Varga::D30)
            .expect("trimsamsa"),
        // Before 1800, where the ephemeris falls back to the analytic model and
        // the pane has to say so.
        "chakraMoshier": almanac
            .chakra(instant(1650, 8, 20, 0.72), PLACE, Varga::D1)
            .expect("moshier chakra"),
    });

    println!("{}", serde_json::to_string_pretty(&document).expect("json"));
}
