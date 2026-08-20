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

use chandra_almanac::{Almanac, Location};
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

/// Shape of a day detail merged across a month, so every optional field is
/// represented by a day on which it is present.
fn merged_day_shape(almanac: &Almanac, graha: Graha) -> Value {
    (1..=28)
        .filter_map(|day| chandra_almanac::time::DateKey::new(2026, 8, day).ok())
        .filter_map(|date| almanac.day_detail(graha, date).ok())
        .map(|detail| shape(&serde_json::to_value(detail).unwrap()))
        .reduce(merge)
        .expect("a month yields at least one day")
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

    shapes.insert(
        "MoonMonth",
        shape(&serde_json::to_value(almanac.moon_month(2026, 8).expect("moon month")).unwrap()),
    );
    shapes.insert(
        "GrahaMonth",
        shape(
            &serde_json::to_value(
                almanac
                    .graha_month(Graha::Mangala, 2025, 2)
                    .expect("graha month"),
            )
            .unwrap(),
        ),
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
