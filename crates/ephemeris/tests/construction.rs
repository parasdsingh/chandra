//! What the engine refuses before it exists.
//!
//! Its own process, deliberately. The engine is a process-wide singleton, so a
//! test that holds one gets `AlreadyConstructed` from every later `Engine::new`
//! and never reaches the checks it was written for. That is what the previous
//! version of this did: it asserted an error and got the wrong one.

use std::fs;
use std::path::PathBuf;

use chandra_ephemeris::{Engine, Graha, SiderealConfig, Source};

fn bundled() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/resources/ephe")
}

/// One test, in order, because the guard being exercised is process-wide: two of
/// these running at once would each see the other's engine.
#[test]
fn construction_refuses_everything_that_would_downgrade_silently() {
    // A path to nothing. The wrong directory is the ordinary mistake.
    let missing = PathBuf::from("/nonexistent/chandra/ephe");
    assert!(
        Engine::new(&missing, SiderealConfig::default()).is_err(),
        "a missing data directory must be an error, not a silent Moshier fallback"
    );

    // A directory that exists and holds nothing. `is_dir` passes it, Swiss
    // Ephemeris accepts it, and every result afterwards is Moshier with no
    // marker anywhere: the failure D-006 exists to prevent, reached through the
    // check meant to prevent it.
    let empty = std::env::temp_dir().join("chandra-empty-ephe");
    let _ = fs::remove_dir_all(&empty);
    fs::create_dir_all(&empty).expect("temp dir");
    assert!(
        Engine::new(&empty, SiderealConfig::default()).is_err(),
        "an empty data directory must be an error, not a silent Moshier fallback"
    );
    let _ = fs::remove_dir_all(&empty);

    // A refused construction must leave the singleton free, or one bad path at
    // startup would make every later attempt fail for the wrong reason.
    let engine = Engine::new(&bundled(), SiderealConfig::default())
        .expect("the bundled data must construct after two refusals");
    assert_eq!(
        engine
            .position(2_451_545.0, Graha::Chandra)
            .expect("position")
            .source,
        Source::Swieph
    );

    // And a second one is refused while the first is alive.
    assert!(
        Engine::new(&bundled(), SiderealConfig::default()).is_err(),
        "a second engine would silently reconfigure the first"
    );
}
