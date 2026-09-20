//! What `month_index` costs at the offsets the clamp allows.
//!
//!     cargo run -q --release -p chandra --example index_cost
//!
//! `MONTH_OFFSET_LIMIT` is 2,400 and bounds a hostile or corrupted value rather
//! than a real one - the front end re-anchors at six. It was sized against
//! `moon_month`, which walks the offset once; `month_index` asked for up to
//! twenty-nine absolute offsets and re-walked from the anchor for each.

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

    for offset in [0, 6, 60, 600, -2400, 2400] {
        let cursor = MonthCursor {
            anchor_unix_ms: anchor,
            offset,
            system: MonthSystem::Amanta,
            first_weekday: 0,
        };

        let start = Instant::now();
        let month = almanac.moon_month(cursor).expect("month");
        let month_cost = start.elapsed();

        let start = Instant::now();
        let index = almanac.month_index(cursor).expect("index");
        let index_cost = start.elapsed();

        println!(
            "offset {offset:>6}  moon_month {:>7.2?}  month_index {:>7.2?}  {} months, {}, cell 0 {}",
            month_cost,
            index_cost,
            index.months.len(),
            index.year,
            month.label,
        );
        for entry in &index.months {
            print!("{} @{} ", entry.name, entry.offset);
        }
        println!();
    }
}
