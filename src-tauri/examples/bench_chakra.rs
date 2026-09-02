//! How much a chart costs, so the animation's tick rate is chosen from a
//! measurement rather than from a guess.
//!
//!     cargo run -q --release -p chandra --example bench_chakra

use std::path::PathBuf;
use std::time::Instant;

use chandra_almanac::varga::Varga;
use chandra_almanac::{Almanac, Location};
use chandra_ephemeris::{Ayanamsa, NodeType, Observer, SiderealConfig};

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
            node_type: NodeType::Mean,
        },
    )
    .expect("almanac");

    let base = 1_787_000_000_000i64;
    // Warm the ephemeris file cache first; the first call pays for the read.
    for i in 0..50 {
        almanac
            .chakra(base + i * 1000, "Bengaluru", Varga::D1)
            .expect("warm");
    }

    let runs = 2000;
    let start = Instant::now();
    for i in 0..runs {
        almanac
            .chakra(base + i * 1000, "Bengaluru", Varga::D1)
            .expect("chakra");
    }
    let each = start.elapsed() / runs as u32;
    println!("chakra: {:?} per call", each);
    println!(
        "at 1 Hz that is {:.4}% of one core",
        each.as_secs_f64() * 100.0
    );
    println!(
        "at 10 Hz that is {:.3}% of one core",
        each.as_secs_f64() * 1000.0
    );
}
