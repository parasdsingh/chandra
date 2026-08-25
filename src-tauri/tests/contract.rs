//! Guards the IPC contract against drift.
//!
//! `src/ipc/types.ts` is hand-written, so nothing stops the Rust side changing
//! shape underneath it. This test serialises a real value of every payload type,
//! reduces it to its key structure, and compares that against a committed
//! fixture. A field added, removed or renamed in Rust fails here, and the diff
//! names exactly what to change in TypeScript.
//!
//! Regenerate deliberately, never to make a red test green:
//!     UPDATE_CONTRACT=1 cargo test -p chandra --test contract

use std::collections::BTreeMap;
use std::path::PathBuf;

use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::{Almanac, Location, MonthCursor};
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, Observer, SiderealConfig};
use serde_json::{json, Map, Value};

const FIXTURE: &str = "tests/contract.json";

/// Reduces a value to its structure: field names and leaf types, without the
/// data. Comparing whole values would fail whenever the sky moved.
fn shape(value: &Value) -> Value {
    match value {
        Value::Object(fields) => {
            let mut shaped = Map::new();
            for (key, field) in fields {
                shaped.insert(key.clone(), shape(field));
            }
            Value::Object(shaped)
        }
        Value::Array(items) => match items.first() {
            // The element type is what matters; length varies with the month.
            Some(first) => json!([shape(first)]),
            None => json!([]),
        },
        Value::String(_) => json!("string"),
        Value::Number(number) => {
            if number.is_f64() {
                json!("number")
            } else {
                json!("integer")
            }
        }
        Value::Bool(_) => json!("boolean"),
        // A null in a sample says the field is optional but not what it holds,
        // which is exactly what the TypeScript side declares as `| null`.
        Value::Null => json!("null"),
    }
}

/// Combines shapes so an optional field that is absent in one sample takes the
/// structure it had in another.
///
/// Without this the fixture would record `"moonset": "null"` for a date where
/// the Moon happens not to set, hiding the field's real shape and making the
/// contract depend on which day was sampled.
fn merge(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::String(left), Value::String(right)) if left == "null" && right != "null" => {
            Value::String(right)
        }
        (Value::String(left), right) if left == "null" => right,
        (left, Value::String(right)) if right == "null" => left,
        (Value::Object(mut left), Value::Object(right)) => {
            for (key, value) in right {
                let merged = match left.remove(&key) {
                    Some(existing) => merge(existing, value),
                    None => value,
                };
                left.insert(key, merged);
            }
            Value::Object(left)
        }
        (Value::Array(left), Value::Array(right)) => {
            match (left.into_iter().next(), right.into_iter().next()) {
                (Some(l), Some(r)) => json!([merge(l, r)]),
                (Some(l), None) => json!([l]),
                (None, Some(r)) => json!([r]),
                (None, None) => json!([]),
            }
        }
        (left, _) => left,
    }
}

/// Every optional limb on.
///
/// The fixture records what the front end may receive, so it has to be built
/// from days that carry everything. A fixture built with the shipped defaults
/// would record yoga, karana and the muhurtas as empty lists and let the two
/// sides drift apart on their shape unnoticed.
fn limbs() -> chandra_almanac::day::DayOptions {
    chandra_almanac::day::DayOptions {
        yogas: true,
        karanas: true,
        muhurtas: true,
    }
}

/// Shape of a day detail merged across a month, so every optional field is
/// represented by a day on which it is present.
///
/// A whole month rather than one day: kshaya and vriddhi occur about once a
/// month each, and a day that carries neither records their fields as absent.
///
/// The month system is no longer a parameter. A day carries its panchanga in
/// both calendars now, so there is nothing about a day's shape that varies with
/// it.
///
/// January 2024 is sampled alongside August 2026 because Mangala and Budha are
/// inside one degree of each other on the 26th to the 28th. A planetary war is
/// rare enough that a single month almost never holds one, and a fixture built
/// without one records `war` as null - which would let `War`'s own fields change
/// without this test noticing.
fn merged_day_shape(almanac: &Almanac, graha: Graha) -> Value {
    [(2026, 8), (2024, 1)]
        .into_iter()
        .flat_map(|(year, month)| {
            (1..=28)
                .filter_map(move |day| chandra_almanac::time::DateKey::new(year, month, day).ok())
        })
        .filter_map(|date| almanac.day_detail(graha, date, limbs()).ok())
        .map(|detail| shape(&serde_json::to_value(detail).unwrap()))
        .reduce(merge)
        .expect("a month yields at least one day")
}

fn cursor_in(year: i32, month: u32, system: MonthSystem) -> MonthCursor {
    MonthCursor {
        anchor_unix_ms: (chandra_ephemeris::jd_to_unix_seconds(chandra_ephemeris::julian_day(
            year, month, 15, 0.25,
        )) * 1000.0) as i64,
        offset: 0,
        system,
        first_weekday: 0,
    }
}

fn almanac() -> Almanac {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/ephe");
    Almanac::new(
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
    .expect("almanac")
}

#[test]
fn ipc_payload_shapes_match_the_committed_contract() {
    let almanac = almanac();
    let mut shapes: BTreeMap<&str, Value> = BTreeMap::new();

    // Merged across both systems for the same reason day shapes are: a cell's
    // tithi exists only in a lunar month.
    shapes.insert(
        "MoonMonth",
        [MonthSystem::Solar, MonthSystem::Amanta]
            .into_iter()
            .map(|system| {
                shape(
                    &serde_json::to_value(
                        almanac
                            .moon_month(cursor_in(2026, 8, system))
                            .expect("moon month"),
                    )
                    .unwrap(),
                )
            })
            .reduce(merge)
            .expect("both systems yield a month"),
    );
    shapes.insert(
        "GrahaMonth",
        [MonthSystem::Solar, MonthSystem::Amanta]
            .into_iter()
            .map(|system| {
                shape(
                    &serde_json::to_value(
                        almanac
                            .graha_month(Graha::Mangala, cursor_in(2025, 2, system))
                            .expect("graha month"),
                    )
                    .unwrap(),
                )
            })
            .reduce(merge)
            .expect("both systems yield a month"),
    );
    shapes.insert("MoonDay", merged_day_shape(&almanac, Graha::Chandra));
    shapes.insert("GrahaDay", merged_day_shape(&almanac, Graha::Mangala));
    shapes.insert(
        "Snapshot",
        shape(
            &serde_json::to_value(
                almanac
                    .now(1_755_000_000_000, &[Graha::Mangala, Graha::Shani])
                    .expect("snapshot"),
            )
            .unwrap(),
        ),
    );
    shapes.insert(
        "Settings",
        shape(&serde_json::to_value(sample_settings()).unwrap()),
    );
    shapes.insert(
        "City",
        shape(&serde_json::to_value(chandra_geo::search("kolkata", 1).first().unwrap()).unwrap()),
    );

    let produced = serde_json::to_value(&shapes).expect("shapes serialise");
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);

    if std::env::var("UPDATE_CONTRACT").is_ok() {
        std::fs::write(
            &fixture_path,
            format!("{}\n", serde_json::to_string_pretty(&produced).unwrap()),
        )
        .expect("write fixture");
        return;
    }

    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(&fixture_path).expect("contract fixture must exist"),
    )
    .expect("fixture is valid JSON");

    if produced != expected {
        let produced_text = serde_json::to_string_pretty(&produced).unwrap();
        let expected_text = serde_json::to_string_pretty(&expected).unwrap();
        panic!(
            "The IPC payload shape changed.\n\nsrc/ipc/types.ts must be updated to match, then \
             regenerate with UPDATE_CONTRACT=1.\n\n--- committed ---\n{expected_text}\n\n--- now \
             produced ---\n{produced_text}"
        );
    }
}

/// A settings value with every optional field populated, so the fixture records
/// the shape of `place` rather than a bare null.
fn sample_settings() -> chandra_lib::PublicSettings {
    chandra_lib::PublicSettings::sample()
}
