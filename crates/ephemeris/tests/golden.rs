//! Golden vector tests for the Swiss Ephemeris boundary.
//!
//! Expected values in `golden.json` are produced by `tools/gen_golden.py`, which
//! drives Swiss Ephemeris' own `swetest` command line program. Because that is a
//! different program reaching the library through a different code path, a
//! mistake anywhere in this crate - flag words, body identifiers, the Ketu
//! reflection, the geopos argument order, sidereal setup - shows up as a
//! mismatch rather than being silently encoded into the expectation.
//!
//! Regenerate with:
//!     python3 tools/gen_golden.py > crates/ephemeris/tests/golden.json

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use chandra_ephemeris::{Ayanamsa, Engine, Graha, NodeType, Observer, SiderealConfig, Source};
use serde_json::Value;

/// Positions agree with the reference to well under a milliarcsecond; the
/// tolerance is set at 1 milliarcsecond purely to absorb the decimal rounding in
/// swetest's own text output.
const ARCSEC: f64 = 1.0 / 3600.0;
const POSITION_TOLERANCE_DEG: f64 = ARCSEC / 1000.0;
/// swetest prints rise and set to a tenth of a second.
const RISE_TOLERANCE_DAYS: f64 = 0.2 / 86_400.0;

/// Swiss Ephemeris state is process-global and [`Engine`] enforces a single
/// instance, so every test shares one engine and holds this lock for its whole
/// body. Reconfiguring the ayanamsa is a global mutation and cannot overlap.
fn engine() -> MutexGuard<'static, Engine> {
    static ENGINE: OnceLock<Mutex<Engine>> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/resources/ephe");
            let engine = Engine::new(&path, SiderealConfig::default())
                .expect("bundled ephemeris data must be present");
            Mutex::new(engine)
        })
        .lock()
        .expect("engine lock poisoned by an earlier test failure")
}

fn golden() -> &'static Value {
    static DOC: OnceLock<Value> = OnceLock::new();
    DOC.get_or_init(|| {
        let raw = include_str!("golden.json");
        serde_json::from_str(raw).expect("golden.json must be valid JSON")
    })
}

fn ayanamsa_by_key(key: &str) -> Ayanamsa {
    Ayanamsa::ALL
        .into_iter()
        .find(|a| a.key() == key)
        .unwrap_or_else(|| panic!("golden.json names an unknown ayanamsa: {key}"))
}

fn graha_by_key(key: &str) -> Graha {
    Graha::ALL
        .into_iter()
        .find(|g| g.key() == key)
        .unwrap_or_else(|| panic!("golden.json names an unknown graha: {key}"))
}

fn f(value: &Value, field: &str) -> f64 {
    value[field]
        .as_f64()
        .unwrap_or_else(|| panic!("golden.json field {field} is not a number"))
}

#[test]
fn positions_match_swetest() {
    let engine = engine();
    let cases = golden()["positions"].as_array().expect("positions array");
    assert!(!cases.is_empty(), "golden.json has no position cases");

    let mut checked = 0usize;
    for case in cases {
        let node_type = match case["node_type"].as_str().unwrap() {
            "true" => NodeType::True,
            "mean" => NodeType::Mean,
            other => panic!("unknown node type {other}"),
        };
        engine
            .reconfigure(SiderealConfig {
                ayanamsa: ayanamsa_by_key(case["ayanamsa"].as_str().unwrap()),
                node_type,
            })
            .expect("reconfigure");

        let jd = f(case, "jd_ut");
        let bodies = case["bodies"].as_object().expect("bodies object");

        for (key, expected) in bodies {
            let graha = graha_by_key(key);
            let got = engine.position(jd, graha).expect("position");

            let d_lon = (got.longitude - f(expected, "longitude")).abs();
            assert!(
                d_lon < POSITION_TOLERANCE_DEG,
                "{key} longitude at JD {jd} ({:?}): got {}, want {}, delta {:.6} arcsec",
                case["ayanamsa"],
                got.longitude,
                f(expected, "longitude"),
                d_lon * 3600.0
            );
            assert!(
                (got.latitude - f(expected, "latitude")).abs() < POSITION_TOLERANCE_DEG,
                "{key} latitude at JD {jd}"
            );
            assert!(
                (got.speed - f(expected, "speed")).abs() < POSITION_TOLERANCE_DEG,
                "{key} speed at JD {jd}: got {}, want {}",
                got.speed,
                f(expected, "speed")
            );
            checked += 1;
        }
    }
    assert!(
        checked > 100,
        "expected a broad sweep, only checked {checked}"
    );
}

#[test]
fn ketu_is_rahu_reflected() {
    let engine = engine();
    for node_type in [NodeType::True, NodeType::Mean] {
        engine
            .reconfigure(SiderealConfig {
                ayanamsa: Ayanamsa::Lahiri,
                node_type,
            })
            .expect("reconfigure");

        for jd in [2_415_020.5, 2_451_545.0, 2_461_272.5, 2_524_593.5] {
            let rahu = engine.position(jd, Graha::Rahu).expect("rahu");
            let ketu = engine.position(jd, Graha::Ketu).expect("ketu");

            let separation = (ketu.longitude - rahu.longitude).rem_euclid(360.0);
            assert!(
                (separation - 180.0).abs() < 1e-9,
                "Ketu must be exactly opposite Rahu at JD {jd}, got {separation}"
            );
            assert_eq!(ketu.speed, rahu.speed, "the node pair moves as one");
        }
    }
}

#[test]
fn illumination_matches_swetest() {
    let engine = engine();
    for case in golden()["illumination"]
        .as_array()
        .expect("illumination array")
    {
        let jd = f(case, "jd_ut");
        let got = engine.illumination(jd).expect("illumination");

        assert!(
            (got.fraction - f(case, "fraction")).abs() < 1e-9,
            "illuminated fraction at JD {jd}: got {}, want {}",
            got.fraction,
            f(case, "fraction")
        );
        // swetest prints these two sexagesimally, so they round at 0.1 arcsec.
        assert!(
            (got.phase_angle - f(case, "phase_angle")).abs() < ARCSEC,
            "phase angle at JD {jd}"
        );
        assert!(
            (got.elongation - f(case, "elongation")).abs() < ARCSEC,
            "elongation at JD {jd}"
        );
        assert!(
            (0.0..=1.0).contains(&got.fraction),
            "illuminated fraction out of range at JD {jd}"
        );
    }
}

#[test]
fn ayanamsa_matches_the_value_applied_to_positions() {
    let engine = engine();
    for case in golden()["ayanamsa"].as_array().expect("ayanamsa array") {
        engine
            .reconfigure(SiderealConfig {
                ayanamsa: ayanamsa_by_key(case["ayanamsa"].as_str().unwrap()),
                node_type: NodeType::True,
            })
            .expect("reconfigure");

        let jd = f(case, "jd_ut");
        let got = engine.ayanamsa(jd).expect("ayanamsa");
        assert!(
            (got - f(case, "degrees")).abs() < POSITION_TOLERANCE_DEG,
            "{} ayanamsa at JD {jd}: got {got}, want {}",
            case["ayanamsa"],
            f(case, "degrees")
        );
    }
}

#[test]
fn rise_and_set_match_swetest() {
    let engine = engine();
    engine
        .reconfigure(SiderealConfig::default())
        .expect("reconfigure");

    let mut absent = 0usize;
    for case in golden()["rise_set"].as_array().expect("rise_set array") {
        let graha = graha_by_key(case["graha"].as_str().unwrap());
        let obs = &case["observer"];
        let observer = Observer::new(f(obs, "latitude"), f(obs, "longitude"), f(obs, "elevation"));
        let jd = chandra_ephemeris::julian_day(
            case["date"][0].as_i64().unwrap() as i32,
            case["date"][1].as_u64().unwrap() as u32,
            case["date"][2].as_u64().unwrap() as u32,
            0.0,
        );

        let got = engine.rise_set(jd, graha, observer).expect("rise_set");

        for (label, actual, expected) in [
            ("rise", got.rise, case["rise_jd"].as_f64()),
            ("set", got.set, case["set_jd"].as_f64()),
        ] {
            match (actual, expected) {
                (Some(a), Some(e)) => assert!(
                    (a - e).abs() < RISE_TOLERANCE_DAYS,
                    "{} {label} at {} on {:?}: delta {:.3} s",
                    graha.name(),
                    case["place"],
                    case["date"],
                    (a - e) * 86_400.0
                ),
                (None, None) => absent += 1,
                (a, e) => panic!(
                    "{} {label} at {} on {:?}: got {a:?}, want {e:?}",
                    graha.name(),
                    case["place"],
                    case["date"]
                ),
            }
        }
    }
    assert!(
        absent > 0,
        "the fixture must include a day with no rise or set, or the None path is untested"
    );
}

#[test]
fn bundled_data_serves_the_navigable_range_at_full_precision() {
    let engine = engine();
    engine
        .reconfigure(SiderealConfig::default())
        .expect("reconfigure");

    for (year, expected) in [
        (1800, Source::Swieph),
        (2026, Source::Swieph),
        (2399, Source::Swieph),
        // Outside the bundled files Swiss Ephemeris degrades silently. The point
        // of Source is that the degradation is visible; see DECISIONS.md D-006.
        (1700, Source::Moshier),
        (2500, Source::Moshier),
    ] {
        let jd = chandra_ephemeris::julian_day(year, 6, 15, 0.0);
        let got = engine.position(jd, Graha::Chandra).expect("position");
        assert_eq!(got.source, expected, "provenance for year {year}");
    }
}

#[test]
fn a_second_engine_is_refused() {
    let _held = engine();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/resources/ephe");
    assert!(
        Engine::new(&path, SiderealConfig::default()).is_err(),
        "a second engine would silently reconfigure the first"
    );
}

#[test]
fn missing_data_directory_is_an_error_not_a_silent_downgrade() {
    let _held = engine();
    let missing = PathBuf::from("/nonexistent/chandra/ephe");
    assert!(Engine::new(&missing, SiderealConfig::default()).is_err());
}
