//! What a run of Previous-year clicks costs in the month-jump overlay.
//!
//!     cargo run -q --release -p chandra --example year_clicks
//!
//! The overlay keeps the anchor it opened with and moves a `probe` offset by
//! roughly twelve a click, so the nth click asks for an offset of about 12n -
//! and `lunar::shift` walks one syzygy at a time to get there.

use std::path::PathBuf;
use std::time::Instant;

use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::{Almanac, Location, MonthCursor};
use chandra_ephemeris::{Ayanamsa, NodeType, Observer, SiderealConfig};

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/ephe");
    let almanac = Almanac::new(
        &path,
        Location {
            observer: Observer::new(12.97, 77.59, 920.0),
            zone_name: "Asia/Kolkata".into(),
        },
        SiderealConfig {
            ayanamsa: Ayanamsa::Lahiri,
            node_type: NodeType::Mean,
        },
    )
    .expect("almanac");

    let anchor = 1_787_000_000_000i64;
    let cursor = |offset: i32| MonthCursor {
        anchor_unix_ms: anchor,
        offset,
        system: MonthSystem::Amanta,
        first_weekday: 0,
    };

    // The overlay opens on the current month, then each click takes the
    // `previous_year` offset the index just reported.
    let mut probe = 0i32;
    let mut total = std::time::Duration::ZERO;
    println!("click   offset   month_index   cumulative");
    for click in 0..=12 {
        let start = Instant::now();
        let index = almanac.month_index(cursor(probe)).expect("index");
        let cost = start.elapsed();
        total += cost;
        println!(
            "{click:>5}   {probe:>6}   {:>11.0?}   {:>10.1?}   {}",
            cost, total, index.year
        );
        probe = index.previous_year;
    }
}
