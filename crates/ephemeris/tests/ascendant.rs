//! The lagna, checked against a derivation rather than against a second reading.
//!
//! `swe_houses_ex` takes latitude before longitude, which is the reverse of the
//! rest of this crate, and a transposed call does not fail - it returns a real
//! ascendant for a real place. Nothing at runtime would look wrong. So the guard
//! cannot be "call it again and compare"; it has to arrive at the number from
//! the inputs by a different route.
//!
//! The geographic latitude appears in the closed form below, so a transposed
//! argument cannot agree. That is the whole point of choosing this check over
//! the cheaper one - see `docs/design/kundali.md` §2.2.1.

use std::f64::consts::PI;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use chandra_ephemeris::{Engine, Observer, SiderealConfig};

const ARCSEC: f64 = 1.0 / 3600.0;

/// One engine for this binary, shared.
///
/// Swiss Ephemeris keeps its state in C globals and `Engine` enforces one
/// instance per process. A test binary is its own process, but the tests inside
/// it are threads - so the engine is built once and held for each test's body.
fn engine() -> MutexGuard<'static, Engine> {
    static ENGINE: OnceLock<Mutex<Engine>> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/resources/ephe");
            Mutex::new(
                Engine::new(&path, SiderealConfig::default())
                    .expect("bundled ephemeris data must be present"),
            )
        })
        .lock()
        .expect("engine lock poisoned by an earlier test failure")
}

fn radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

fn degrees(radians: f64) -> f64 {
    (radians * 180.0 / PI).rem_euclid(360.0)
}

/// Where the ecliptic point at `longitude` sits in equatorial coordinates.
fn to_equatorial(longitude_deg: f64, obliquity_deg: f64) -> (f64, f64) {
    let (lambda, eps) = (radians(longitude_deg), radians(obliquity_deg));
    let right_ascension = (lambda.sin() * eps.cos()).atan2(lambda.cos());
    let declination = (lambda.sin() * eps.sin()).asin();
    (degrees(right_ascension), declination.to_degrees())
}

/// Altitude of an ecliptic point above the horizon, and its hour angle.
///
/// The hour angle is what decides *which* of the two horizon crossings this is:
/// negative means east of the meridian, which is the half of the sky where
/// things are rising.
fn horizon(longitude_deg: f64, obliquity_deg: f64, ramc_deg: f64, latitude_deg: f64) -> (f64, f64) {
    let (right_ascension, declination) = to_equatorial(longitude_deg, obliquity_deg);
    let hour_angle = {
        let raw = (ramc_deg - right_ascension).rem_euclid(360.0);
        if raw > 180.0 {
            raw - 360.0
        } else {
            raw
        }
    };
    let (phi, dec, h) = (
        radians(latitude_deg),
        radians(declination),
        radians(hour_angle),
    );
    let altitude = (phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();
    (altitude, hour_angle)
}

/// The tropical ascendant, derived from the sidereal time, the obliquity and the
/// latitude - and from nothing the library was asked for.
///
/// The closed form gives the ecliptic longitude where the ecliptic meets the
/// horizon, but it cannot say which of the two crossings that is: the other lies
/// 180 degrees away and is the descendant. The quadrant is settled physically
/// rather than by a remembered rule - the ascendant is the crossing east of the
/// meridian, which is the one with a negative hour angle - and the caller checks
/// that whichever was chosen really is on the horizon.
fn derived_ascendant(ramc_deg: f64, obliquity_deg: f64, latitude_deg: f64) -> f64 {
    let (ramc, eps, phi) = (
        radians(ramc_deg),
        radians(obliquity_deg),
        radians(latitude_deg),
    );

    let candidate = degrees(
        ramc.cos()
            .atan2(-(ramc.sin() * eps.cos() + phi.tan() * eps.sin())),
    );
    let opposite = (candidate + 180.0).rem_euclid(360.0);

    let (_, candidate_hour_angle) = horizon(candidate, obliquity_deg, ramc_deg, latitude_deg);
    if candidate_hour_angle < 0.0 {
        candidate
    } else {
        opposite
    }
}

/// Instants spread so that every rashi takes a turn rising.
///
/// Hourly, not two-hourly. The ascendant does not advance evenly - a sign of
/// short ascension can rise and set again inside two hours at a middling
/// latitude - so a two-hour step reached only ten of the twelve, and the
/// coverage assertion below said so. Four dates across the year move the
/// obliquity and the equation of time as well.
fn instants() -> Vec<f64> {
    let mut out = Vec::new();
    for (year, month, day) in [(2026, 1, 15), (2026, 4, 15), (2026, 8, 30), (2026, 11, 15)] {
        for hour in 0..24 {
            // A day *fraction*, not hours. Passing the hour directly stepped a
            // whole day each time, which at a fixed UT advances the ascendant by
            // about a degree - so 24 samples covered 23 degrees of it rather
            // than 360, and the coverage assertion below caught that the sample
            // proved almost nothing.
            out.push(chandra_ephemeris::julian_day(
                year,
                month,
                day,
                hour as f64 / 24.0,
            ));
        }
    }
    out
}

/// Latitudes from the equator to the edge of the temperate zone.
///
/// Polar latitudes are excluded deliberately: inside the polar circle the
/// ecliptic can lie wholly above or below the horizon and there is no rising
/// point to agree about. That case is its own decision, not this test's.
const LATITUDES: [f64; 7] = [0.0, 12.9716, -12.9716, 23.4, -23.4, 51.5, -35.3];

#[test]
fn the_ascendant_agrees_with_an_independent_derivation() {
    let engine = engine();
    let mut checked = 0;
    let mut rising_signs = std::collections::BTreeSet::new();

    for latitude in LATITUDES {
        // Longitude deliberately unlike the latitude, so a transposed argument
        // lands somewhere else entirely rather than somewhere nearby.
        let observer = Observer::new(latitude, 77.5946, 0.0);

        for jd in instants() {
            let obliquity = engine.obliquity(jd).expect("obliquity");
            let greenwich_sidereal_hours = engine.sidereal_time(jd).expect("sidereal time");
            let ramc =
                ((greenwich_sidereal_hours + observer.longitude / 15.0) * 15.0).rem_euclid(360.0);

            let derived = derived_ascendant(ramc, obliquity, latitude);

            // The derivation checks itself: whatever it chose has to actually be
            // on the horizon. If the closed form were wrong this would fail here
            // rather than downstream, and it would fail for a reason.
            let (altitude, hour_angle) = horizon(derived, obliquity, ramc, latitude);
            assert!(
                altitude.abs() < 1e-6,
                "the derived ascendant is not on the horizon: altitude {altitude} at \
                 latitude {latitude}, JD {jd}"
            );
            assert!(
                hour_angle < 0.0,
                "the derived ascendant is west of the meridian, so it is setting, \
                 not rising: hour angle {hour_angle} at latitude {latitude}, JD {jd}"
            );

            let library = engine
                .ascendant_tropical(jd, observer)
                .expect("tropical ascendant");

            let apart = {
                let raw = (derived - library).rem_euclid(360.0);
                raw.min(360.0 - raw)
            };
            assert!(
                apart < ARCSEC,
                "derived {derived} and swe_houses_ex {library} disagree by {apart} degrees \
                 at latitude {latitude}, JD {jd}"
            );

            rising_signs.insert((library / 30.0).floor() as u8);
            checked += 1;
        }
    }

    assert!(checked >= 600, "only {checked} cases checked");
    assert_eq!(
        rising_signs.len(),
        12,
        "only {} of the twelve rashis were ever rising; the sample proves less \
         than it looks",
        rising_signs.len()
    );
}

/// The sidereal reading is the tropical one less the ayanamsa the rest of the
/// app applies.
///
/// This is the assertion that would have been its own test. It is here instead,
/// because on its own it is blind to the bug the test above exists for: the
/// ayanamsa does not depend on location, so a transposed call transposes both
/// readings identically and their difference is still exactly right.
///
/// It still earns its place. It catches `SEFLG_SIDEREAL` being dropped, and it
/// catches `swehouse.c` substituting Fagan-Bradley when `swe_set_sid_mode` was
/// never called - which it does silently.
#[test]
fn the_sidereal_lagna_sits_on_the_same_frame_as_every_other_figure() {
    let engine = engine();
    let observer = Observer::new(12.9716, 77.5946, 920.0);

    for jd in instants() {
        let tropical = engine
            .ascendant_tropical(jd, observer)
            .expect("tropical ascendant");
        let sidereal = engine.ascendant(jd, observer).expect("lagna").degrees;
        let ayanamsa = engine.ayanamsa(jd).expect("ayanamsa").degrees;

        let applied = (tropical - sidereal).rem_euclid(360.0);
        let difference = (applied - ayanamsa).abs();

        // Arcsecond, not "close enough". `Engine::ayanamsa` is the
        // tropical-minus-sidereal Sun difference rather than
        // `swe_get_ayanamsa_ex_ut`, because that is the quantity applied to
        // every position the app displays. If `swe_houses_ex` resolves the
        // equinox differently the two part company by the nutation in longitude
        // - about 17 arcseconds - and that would mean the lagna sits on a
        // slightly different sidereal frame from every rashi beside it. The
        // answer to that is to move the lagna onto the app's frame, not to widen
        // this number.
        assert!(
            difference < ARCSEC,
            "the lagna carries an ayanamsa of {applied} where the app applies \
             {ayanamsa}, a difference of {} arcseconds at JD {jd}",
            difference * 3600.0
        );
    }
}
