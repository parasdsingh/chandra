//! Integration tests for calendar assembly.
//!
//! These run the real ephemeris. Expected values are either independently
//! verifiable astronomy (a full moon is 100% lit; a nakshatra boundary is at an
//! exact multiple of 13 degrees 20 arcminutes) or invariants that must hold for
//! every date rather than for one hand-picked one.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use chandra_almanac::month::DayDetail;
use chandra_almanac::time::DateKey;
use chandra_almanac::zodiac::{NAKSHATRA_ARC, RASHI_ARC};
use chandra_almanac::{Almanac, Location};
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, Observer, SiderealConfig, Source};

const BENGALURU: Observer = Observer::new(12.9716, 77.5946, 920.0);

/// One almanac per process: the underlying engine is a singleton because Swiss
/// Ephemeris keeps its configuration in C globals.
fn almanac() -> MutexGuard<'static, Almanac> {
    static INSTANCE: OnceLock<Mutex<Almanac>> = OnceLock::new();
    INSTANCE
        .get_or_init(|| {
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/resources/ephe");
            Mutex::new(
                Almanac::new(
                    &path,
                    Location {
                        observer: BENGALURU,
                        zone_name: "Asia/Kolkata".into(),
                    },
                    SiderealConfig::default(),
                )
                .expect("almanac must build"),
            )
        })
        .lock()
        .expect("almanac lock poisoned by an earlier failure")
}

fn reset(almanac: &Almanac) {
    almanac
        .set_location(Location {
            observer: BENGALURU,
            zone_name: "Asia/Kolkata".into(),
        })
        .expect("location");
    almanac
        .set_sidereal(SiderealConfig {
            ayanamsa: Ayanamsa::Lahiri,
            node_type: NodeType::True,
        })
        .expect("sidereal");
}

#[test]
fn a_month_has_one_cell_per_day() {
    let almanac = almanac();
    reset(&almanac);

    for (year, month, expected) in [(2026, 8, 31), (2026, 2, 28), (2024, 2, 29), (2026, 4, 30)] {
        let view = almanac.moon_month(year, month).expect("month");
        assert_eq!(view.days.len(), expected, "{year}-{month}");
        assert_eq!(view.days[0].day, 1);
        assert_eq!(view.days[expected - 1].day, expected as i8);
        assert_eq!(view.source, Source::Swieph);
    }
}

#[test]
fn illumination_is_a_fraction_and_tracks_the_phase() {
    let almanac = almanac();
    reset(&almanac);

    let view = almanac.moon_month(2026, 8).expect("month");
    for cell in &view.days {
        assert!(
            (0.0..=1.0).contains(&cell.illumination),
            "day {} illumination {}",
            cell.day,
            cell.illumination
        );
    }

    for cell in view.days.iter().filter(|c| c.principal) {
        use chandra_almanac::phase::PhaseName::*;
        match cell.phase {
            FullMoon => assert!(
                cell.illumination > 0.97,
                "full moon at {}",
                cell.illumination
            ),
            NewMoon => assert!(
                cell.illumination < 0.03,
                "new moon at {}",
                cell.illumination
            ),
            FirstQuarter | LastQuarter => assert!(
                (cell.illumination - 0.5).abs() < 0.06,
                "quarter at {}",
                cell.illumination
            ),
            other => panic!("{other:?} is not a principal phase"),
        }
    }
}

#[test]
fn every_principal_phase_falls_on_exactly_one_day() {
    let almanac = almanac();
    reset(&almanac);

    let view = almanac.moon_month(2026, 8).expect("month");
    let principal: Vec<_> = view.days.iter().filter(|c| c.principal).collect();

    assert!(
        (3..=5).contains(&principal.len()),
        "a 31 day month should contain 3 to 5 principal phases, found {}",
        principal.len()
    );
    for pair in principal.windows(2) {
        assert!(
            pair[1].day - pair[0].day >= 6,
            "principal phases {} and {} are too close to be real",
            pair[0].day,
            pair[1].day
        );
    }
}

#[test]
fn nakshatra_entry_and_exit_land_on_exact_boundaries() {
    let almanac = almanac();
    reset(&almanac);

    let detail = almanac
        .day_detail(Graha::Chandra, DateKey::new(2026, 8, 20).unwrap())
        .expect("detail");
    let DayDetail::Moon(moon) = detail else {
        panic!("the Moon must produce a moon detail");
    };

    assert!(!moon.nakshatras.is_empty());
    assert_eq!(
        moon.nakshatras.iter().filter(|n| n.prevailing).count(),
        1,
        "exactly one nakshatra prevails at the reference instant"
    );
    assert_eq!(moon.rashis.iter().filter(|r| r.prevailing).count(), 1);

    let prevailing = moon.nakshatras.iter().find(|n| n.prevailing).unwrap();
    assert_eq!(prevailing.name, "Vishakha");
    assert!((1..=4).contains(&prevailing.pada));
    assert_eq!(
        moon.rashis.iter().find(|r| r.prevailing).unwrap().name,
        "Vrishchika"
    );

    for nakshatra in &moon.nakshatras {
        assert!(
            nakshatra.entry.unix_ms < nakshatra.exit.unix_ms,
            "a span must not end before it starts"
        );
    }
}

#[test]
fn a_span_boundary_is_the_instant_the_longitude_crosses_it() {
    let almanac = almanac();
    reset(&almanac);

    let detail = almanac
        .day_detail(Graha::Chandra, DateKey::new(2026, 8, 20).unwrap())
        .expect("detail");
    let DayDetail::Moon(moon) = detail else {
        panic!("moon detail");
    };

    let longitude_at = |unix_ms: i64| {
        almanac
            .now(unix_ms, &[Graha::Chandra])
            .expect("snapshot")
            .grahas[0]
            .longitude
    };

    for span in &moon.nakshatras {
        let longitude = longitude_at(span.entry.unix_ms);
        let offset = (longitude % NAKSHATRA_ARC).min(NAKSHATRA_ARC - longitude % NAKSHATRA_ARC);
        assert!(
            offset < 1e-4,
            "nakshatra entry is {offset} degrees off the boundary"
        );
    }
    for span in &moon.rashis {
        let longitude = longitude_at(span.entry.unix_ms);
        let offset = (longitude % RASHI_ARC).min(RASHI_ARC - longitude % RASHI_ARC);
        assert!(
            offset < 1e-4,
            "rashi entry is {offset} degrees off the boundary"
        );
    }
}

#[test]
fn moonrise_and_moonset_stay_inside_the_day_they_are_reported_for() {
    let almanac = almanac();
    reset(&almanac);

    let mut days_with_no_rise = 0;
    for day in 1..=31 {
        let DayDetail::Moon(moon) = almanac
            .day_detail(Graha::Chandra, DateKey::new(2026, 8, day).unwrap())
            .expect("detail")
        else {
            panic!("moon detail");
        };

        if let Some(rise) = moon.moonrise {
            assert_eq!(rise.day_offset, 0, "rise must fall inside the day");
        } else {
            days_with_no_rise += 1;
        }
        if let Some(set) = moon.moonset {
            assert_eq!(set.day_offset, 0, "set must fall inside the day");
        }
    }
    assert!(
        days_with_no_rise >= 1,
        "expected at least one day without a moonrise in a full month"
    );
}

#[test]
fn graha_months_find_stations_and_ingresses() {
    use chandra_almanac::events::EventKind;

    let almanac = almanac();
    reset(&almanac);

    let view = almanac.graha_month(Graha::Mangala, 2025, 2).expect("month");
    let stations: Vec<_> = view
        .events
        .iter()
        .filter(|e| e.kind == EventKind::DirectStation)
        .collect();
    assert_eq!(
        stations.len(),
        1,
        "Mangala turns direct once in February 2025"
    );
    assert_eq!(stations[0].date.day, 24, "station date");

    let before = view.days.iter().find(|d| d.day == 20).unwrap();
    let after = view.days.iter().find(|d| d.day == 28).unwrap();
    assert!(before.retrograde, "Mangala retrograde before the station");
    assert!(!after.retrograde, "Mangala direct after the station");
}

#[test]
fn events_are_ordered_and_attributed_to_the_day_they_occur_on() {
    let almanac = almanac();
    reset(&almanac);

    let view = almanac.graha_month(Graha::Budha, 2026, 8).expect("month");
    assert!(
        !view.events.is_empty(),
        "Budha moves fast enough to produce events"
    );

    for pair in view.events.windows(2) {
        assert!(
            pair[0].at.unix_ms <= pair[1].at.unix_ms,
            "events must be in time order"
        );
    }
    for event in &view.events {
        assert_eq!(event.date.year, 2026);
        assert_eq!(event.date.month, 8);
        assert_eq!(
            event.at.day_offset, 0,
            "an event must be expressed relative to its own day"
        );
    }
}

#[test]
fn the_moon_is_not_served_by_the_transit_view() {
    let almanac = almanac();
    reset(&almanac);
    assert!(almanac.graha_month(Graha::Chandra, 2026, 8).is_err());
}

#[test]
fn changing_the_ayanamsa_changes_positions_and_clears_the_cache() {
    let almanac = almanac();
    reset(&almanac);

    let lahiri = almanac.graha_month(Graha::Shani, 2026, 8).expect("month");
    almanac
        .set_sidereal(SiderealConfig {
            ayanamsa: Ayanamsa::Raman,
            node_type: NodeType::True,
        })
        .expect("set ayanamsa");
    let raman = almanac.graha_month(Graha::Shani, 2026, 8).expect("month");

    let difference = (lahiri.days[0].longitude - raman.days[0].longitude).abs();
    assert!(
        difference > 1.0,
        "Lahiri and Raman differ by about 1.4 degrees; got {difference}"
    );

    reset(&almanac);
    let back = almanac.graha_month(Graha::Shani, 2026, 8).expect("month");
    assert!(
        (back.days[0].longitude - lahiri.days[0].longitude).abs() < 1e-9,
        "returning to Lahiri must reproduce the original figures"
    );
}

#[test]
fn changing_location_changes_rise_times() {
    let almanac = almanac();
    reset(&almanac);

    let date = DateKey::new(2026, 8, 20).unwrap();
    let DayDetail::Moon(here) = almanac.day_detail(Graha::Chandra, date).expect("detail") else {
        panic!("moon detail");
    };

    almanac
        .set_location(Location {
            observer: Observer::new(-33.8688, 151.2093, 58.0),
            zone_name: "Australia/Sydney".into(),
        })
        .expect("relocate");
    let DayDetail::Moon(sydney) = almanac.day_detail(Graha::Chandra, date).expect("detail") else {
        panic!("moon detail");
    };

    assert_ne!(
        here.moonrise.map(|m| m.unix_ms),
        sydney.moonrise.map(|m| m.unix_ms),
        "moonrise must depend on where the observer is"
    );
    reset(&almanac);
}

#[test]
fn a_cold_month_is_assembled_well_inside_the_budget() {
    let almanac = almanac();
    reset(&almanac);

    let started = Instant::now();
    let _ = almanac.moon_month(2031, 3).expect("month");
    let cold = started.elapsed();

    let started = Instant::now();
    let _ = almanac.moon_month(2031, 3).expect("month");
    let warm = started.elapsed();

    // Budget from docs/ARCHITECTURE.md section 5, with headroom so this fails on
    // a regression rather than on a busy machine.
    assert!(
        cold.as_millis() < 120,
        "cold moon month took {cold:?}, budget 30ms with headroom"
    );
    assert!(
        warm.as_micros() < 5_000,
        "warm moon month took {warm:?}, must be served from cache"
    );
    assert!(warm < cold, "the cache must actually be faster");
}

#[test]
fn a_cold_graha_month_is_assembled_well_inside_the_budget() {
    let almanac = almanac();
    reset(&almanac);

    let started = Instant::now();
    let _ = almanac.graha_month(Graha::Shani, 2032, 5).expect("month");
    let slow_body = started.elapsed();

    let started = Instant::now();
    let _ = almanac.graha_month(Graha::Budha, 2032, 5).expect("month");
    let fast_body = started.elapsed();

    assert!(
        slow_body.as_millis() < 120,
        "Shani month took {slow_body:?}"
    );
    assert!(
        fast_body.as_millis() < 250,
        "Budha month took {fast_body:?}"
    );
}

#[test]
fn dates_outside_the_bundled_data_are_marked_as_degraded() {
    let almanac = almanac();
    reset(&almanac);

    let inside = almanac.moon_month(2026, 8).expect("month");
    assert_eq!(inside.source, Source::Swieph);

    let outside = almanac.moon_month(1650, 8).expect("month");
    assert_eq!(
        outside.source,
        Source::Moshier,
        "a month before 1800 must be reported as lower precision, not passed off as exact"
    );
}

/// Not an assertion: prints the figures that back the budgets in
/// `docs/ARCHITECTURE.md`, so a regression is visible with `--nocapture`.
#[test]
fn report_timings() {
    let almanac = almanac();
    reset(&almanac);

    let line = |label: &str, elapsed: std::time::Duration| {
        println!("{label:<34} {:>8.2} ms", elapsed.as_secs_f64() * 1000.0);
    };

    let t = Instant::now();
    let _ = almanac.moon_month(2033, 7).expect("month");
    line("moon month, cold", t.elapsed());

    let t = Instant::now();
    let _ = almanac.moon_month(2033, 7).expect("month");
    line("moon month, cached", t.elapsed());

    for graha in [Graha::Budha, Graha::Mangala, Graha::Guru, Graha::Shani] {
        let t = Instant::now();
        let _ = almanac.graha_month(graha, 2033, 7).expect("month");
        line(&format!("{} month, cold", graha.name()), t.elapsed());
    }

    let t = Instant::now();
    let _ = almanac
        .day_detail(Graha::Chandra, DateKey::new(2033, 7, 15).unwrap())
        .expect("detail");
    line("moon day detail", t.elapsed());

    let t = Instant::now();
    let _ = almanac
        .day_detail(Graha::Shani, DateKey::new(2033, 7, 15).unwrap())
        .expect("detail");
    line("Shani day detail (slowest body)", t.elapsed());

    let t = Instant::now();
    let _ = almanac
        .now(1_755_000_000_000, &Graha::ALL)
        .expect("snapshot");
    line("tray snapshot, all nine grahas", t.elapsed());
}
