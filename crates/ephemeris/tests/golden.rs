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
        // A failing test panics while holding this, which poisons the mutex and
        // makes every later test in the file fail with a message about the
        // lock rather than about itself - one real failure reported as five,
        // and the first one buried. The engine's C state is not damaged by a
        // failed assertion, so the guard is taken anyway and the first failure
        // stays the only failure.
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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
            (got.degrees - f(case, "degrees")).abs() < POSITION_TOLERANCE_DEG,
            "{} ayanamsa at JD {jd}: got {}, want {}",
            case["ayanamsa"],
            got.degrees,
            f(case, "degrees")
        );
        // Every value carries the theory that produced it (D-006). The vectors
        // are all inside 1800-2399, so every one of them is data-file backed.
        assert_eq!(got.source, Source::Swieph, "ayanamsa provenance at JD {jd}");
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

        // The reference vectors are for the 24 hour UT day beginning at `jd`.
        let got = engine
            .rise_set(jd, jd + 1.0, graha, observer)
            .expect("rise_set");

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

    // Every entry point, not just `position`. ARCHITECTURE says the flags asked
    // for are compared against the flags returned on every call; two of the four
    // did not, so a user who scrolled outside 1800-2399 got a moonrise with no
    // precision note beside it - the exact failure D-006 exists to prevent, on
    // the one figure the calendar can be scrolled into.
    let observer = Observer::new(12.9716, 77.5946, 920.0);

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

        assert_eq!(
            engine
                .position(jd, Graha::Chandra)
                .expect("position")
                .source,
            expected,
            "position provenance for year {year}"
        );
        assert_eq!(
            engine.illumination(jd).expect("illumination").source,
            expected,
            "illumination provenance for year {year}"
        );
        assert_eq!(
            engine
                .rise_set(jd, jd + 1.0, Graha::Chandra, observer)
                .expect("rise_set")
                .source,
            Some(expected),
            "rise/set provenance for year {year}"
        );
        // The nodes never rise, so there is no window to attribute. An absent
        // provenance is the honest answer, and asking the ephemeris to produce
        // one is work done for a claim about a reading nobody took.
        assert_eq!(
            engine
                .rise_set(jd, jd + 1.0, Graha::Rahu, observer)
                .expect("rise_set")
                .source,
            None,
            "a node has no rise or set to attribute"
        );
        assert_eq!(
            engine.ayanamsa(jd).expect("ayanamsa").source,
            expected,
            "ayanamsa provenance for year {year}"
        );
    }
}

// Construction is tested in `tests/construction.rs`, which is a separate
// process. Every test here holds the process's one engine, so `Engine::new`
// refuses before it reaches any of the checks that were meant to be exercised.

/// A configuration change is one critical section, not two (D-005).
///
/// The node type lives in this crate's own state and the ayanamsa in a Swiss
/// Ephemeris global. Writing them under separate locks lets a reader in on a
/// pairing the user never selected, and the four pairings are more than a degree
/// apart: the reading stays plausible while being wrong, which is the failure
/// this crate exists to make impossible.
#[test]
fn a_configuration_change_is_never_half_applied() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    let guard = engine();
    let engine = &*guard;

    // 2023-02-24, where the true and mean nodes are far enough apart to tell
    // one reading from the other.
    const JD: f64 = 2_460_000.5;
    const READERS: usize = 4;
    const FLIPS: usize = 4000;

    let selected = [
        SiderealConfig {
            ayanamsa: Ayanamsa::Lahiri,
            node_type: NodeType::True,
        },
        SiderealConfig {
            ayanamsa: Ayanamsa::Suryasiddhanta,
            node_type: NodeType::Mean,
        },
    ];

    // Every pairing of the two ayanamsas with the two node types, so a reading
    // that takes one setting from each side of a flip is recognisable rather
    // than merely unexpected.
    let mut pairings = Vec::new();
    for ayanamsa in [Ayanamsa::Lahiri, Ayanamsa::Suryasiddhanta] {
        for node_type in [NodeType::True, NodeType::Mean] {
            let config = SiderealConfig {
                ayanamsa,
                node_type,
            };
            engine.reconfigure(config).expect("reconfigure");
            let longitude = engine.position(JD, Graha::Rahu).expect("rahu").longitude;
            pairings.push((config, longitude));
        }
    }
    for (index, (left, a)) in pairings.iter().enumerate() {
        for (right, b) in &pairings[index + 1..] {
            assert!(
                (a - b).abs() > 0.1,
                "{left:?} and {right:?} are indistinguishable at JD {JD}, so this test \
                 could not see a mixed state"
            );
        }
    }

    let readings = AtomicUsize::new(0);
    let mixed = AtomicUsize::new(0);
    let flipping = AtomicBool::new(true);

    std::thread::scope(|scope| {
        scope.spawn(|| {
            for step in 0..FLIPS {
                engine.reconfigure(selected[step % 2]).expect("reconfigure");
            }
            engine.reconfigure(selected[0]).expect("reconfigure");
            flipping.store(false, Ordering::SeqCst);
        });

        for _ in 0..READERS {
            scope.spawn(|| {
                while flipping.load(Ordering::SeqCst) {
                    let longitude = engine.position(JD, Graha::Rahu).expect("rahu").longitude;
                    readings.fetch_add(1, Ordering::Relaxed);

                    let matches_a_selection = pairings
                        .iter()
                        .filter(|(config, _)| selected.contains(config))
                        .any(|(_, expected)| (expected - longitude).abs() < 1e-9);
                    if !matches_a_selection {
                        mixed.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
    });

    let total = readings.load(Ordering::Relaxed);
    // A sanity check that the race actually ran, not a performance assertion.
    //
    // This used to require more readings than flips, which is a statement about
    // how fast this machine is: under load - another build running, which is
    // exactly when a test suite runs - the readers get fewer turns and the test
    // failed while the invariant it exists to protect was perfectly intact. It
    // then panicked while holding the engine lock, poisoning it, so one flaky
    // failure became five.
    //
    // The bar is now that the race happened at all. `mixed == 0` below is the
    // assertion that matters and is not timing-dependent.
    assert!(
        total > 0,
        "no readings overlapped {FLIPS} changes; the race was never exercised"
    );
    assert_eq!(
        mixed.load(Ordering::Relaxed),
        0,
        "of {total} readings, some were computed under a pairing that was never selected"
    );
}

/// Rise and set are searched over the window the caller asked for.
///
/// A civil day is 23 or 25 hours across a daylight saving change, and this crate
/// has no way to know which - only the zone does. Assuming 1.0 loses a real
/// moonrise in the twenty-fifth hour of a long day and reports one in the
/// twenty-fourth hour of a short day, which by then belongs to tomorrow.
#[test]
fn rise_and_set_are_searched_over_the_window_the_caller_asked_for() {
    let engine = engine();
    engine
        .reconfigure(SiderealConfig::default())
        .expect("reconfigure");

    // New York, the zone the two transitions are checked in upstream.
    let observer = Observer::new(40.7128, -74.0060, 10.0);
    let base = chandra_ephemeris::julian_day(2026, 1, 1, 5.0 / 24.0);

    let mut only_in_the_long_day = 0;
    let mut lost_from_the_short_day = 0;

    for offset in 0..60 {
        let start = base + offset as f64;
        let ordinary = engine
            .rise_set(start, start + 1.0, Graha::Chandra, observer)
            .expect("rise_set");
        let long = engine
            .rise_set(start, start + 25.0 / 24.0, Graha::Chandra, observer)
            .expect("rise_set");
        let short = engine
            .rise_set(start, start + 23.0 / 24.0, Graha::Chandra, observer)
            .expect("rise_set");

        // A wider window never loses an event a narrower one found, and every
        // event reported falls inside the window it was asked for.
        for (narrow, wide) in [(short, ordinary), (ordinary, long)] {
            if let Some(rise) = narrow.rise {
                assert_eq!(wide.rise, Some(rise), "a wider window lost a rise");
            }
            if let Some(set) = narrow.set {
                assert_eq!(wide.set, Some(set), "a wider window lost a set");
            }
        }
        for (window, result) in [
            (start + 23.0 / 24.0, short),
            (start + 1.0, ordinary),
            (start + 25.0 / 24.0, long),
        ] {
            assert!(result.rise.is_none_or(|jd| jd < window));
            assert!(result.set.is_none_or(|jd| jd < window));
        }

        only_in_the_long_day += usize::from(long.rise.is_some() && ordinary.rise.is_none());
        lost_from_the_short_day += usize::from(ordinary.rise.is_some() && short.rise.is_none());
    }

    assert!(
        only_in_the_long_day > 0,
        "no moonrise in these two months fell in a twenty-fifth hour, so the long day is untested"
    );
    assert!(
        lost_from_the_short_day > 0,
        "no moonrise fell in a twenty-fourth hour, so the short day is untested"
    );
}
