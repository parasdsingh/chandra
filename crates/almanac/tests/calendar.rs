//! Integration tests for calendar assembly.
//!
//! These run the real ephemeris. Expected values are either independently
//! verifiable astronomy (a full moon is 100% lit; a nakshatra boundary is at an
//! exact multiple of 13 degrees 20 arcminutes) or invariants that must hold for
//! every date rather than for one hand-picked one.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use chandra_almanac::lunar::MonthSystem as System;
use chandra_almanac::month::{DayDetail, GrahaCell, GrahaMonth, MoonCell, MoonMonth};
use chandra_almanac::time::DateKey;
use chandra_almanac::zodiac::{NAKSHATRA_ARC, RASHI_ARC};
use chandra_almanac::{Almanac, Location, MonthCursor};
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

/// A solar-month cursor pointing at local noon on the first of a month.
fn solar(year: i32, month: u32) -> MonthCursor {
    MonthCursor {
        anchor_unix_ms: (chandra_ephemeris::jd_to_unix_seconds(chandra_ephemeris::julian_day(
            year, month, 15, 0.25,
        )) * 1000.0) as i64,
        offset: 0,
        system: System::Solar,
        // Monday-first, so the grid layout is fixed across every test.
        first_weekday: 0,
    }
}

/// The cells of the month itself.
///
/// A month view carries the whole 42 cell grid, so the leading and trailing days
/// of the neighbouring months are in `days` too. Every assertion about "the
/// month" means these.
fn inside(month: &MoonMonth) -> Vec<&MoonCell> {
    month.days.iter().filter(|cell| cell.in_month).collect()
}

fn inside_graha(month: &GrahaMonth) -> Vec<&GrahaCell> {
    month.days.iter().filter(|cell| cell.in_month).collect()
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
        let view = almanac.moon_month(solar(year, month)).expect("month");
        let days = inside(&view);

        // The grid is always six rows, whatever the month holds.
        assert_eq!(view.days.len(), 42, "{year}-{month} grid");
        assert_eq!(days.len(), expected, "{year}-{month}");
        assert_eq!(days[0].date.day, 1);
        assert_eq!(days[expected - 1].date.day, expected as i8);
        assert_eq!(view.source, Source::Swieph);

        // The leading and trailing cells carry real data, not blanks: they are
        // drawn dimmed, not left empty.
        assert!(view.days.iter().all(|cell| cell.illumination >= 0.0));
    }
}

#[test]
fn illumination_is_a_fraction_and_tracks_the_phase() {
    let almanac = almanac();
    reset(&almanac);

    let view = almanac.moon_month(solar(2026, 8)).expect("month");
    for cell in &view.days {
        assert!(
            (0.0..=1.0).contains(&cell.illumination),
            "day {} illumination {}",
            cell.date.day,
            cell.illumination
        );
    }

    for cell in inside(&view).into_iter().filter(|c| c.phase.is_principal()) {
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

    let view = almanac.moon_month(solar(2026, 8)).expect("month");
    let principal: Vec<_> = inside(&view)
        .into_iter()
        .filter(|c| c.phase.is_principal())
        .collect();

    assert!(
        (3..=5).contains(&principal.len()),
        "a 31 day month should contain 3 to 5 principal phases, found {}",
        principal.len()
    );
    for pair in principal.windows(2) {
        assert!(
            pair[1].date.day - pair[0].date.day >= 6,
            "principal phases {} and {} are too close to be real",
            pair[0].date.day,
            pair[1].date.day
        );
    }
}

#[test]
fn nakshatra_entry_and_exit_land_on_exact_boundaries() {
    let almanac = almanac();
    reset(&almanac);

    let detail = almanac
        .day_detail(
            Graha::Chandra,
            DateKey::new(2026, 8, 20).unwrap(),
            System::Solar,
        )
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
        .day_detail(
            Graha::Chandra,
            DateKey::new(2026, 8, 20).unwrap(),
            System::Solar,
        )
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
            .day_detail(
                Graha::Chandra,
                DateKey::new(2026, 8, day).unwrap(),
                System::Solar,
            )
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

/// A day is not always twenty-four hours, and the rise search must follow it.
///
/// Asia/Kolkata has no daylight saving, so the month above cannot see this at
/// all. The invariant that catches it is the spacing: moonrise drifts about
/// fifty minutes a day, and on the day it drifts past midnight no rise is
/// reported - but the gap between the rises either side of that day is still one
/// interval, not two. A day clamped to twenty-four hours drops a real rise in the
/// twenty-fifth hour of a long day, doubling the gap, and reports one twice
/// across a short day, collapsing it.
#[test]
fn a_daylight_saving_day_reports_every_moonrise_exactly_once() {
    let almanac = almanac();
    reset(&almanac);
    almanac
        .set_location(Location {
            observer: Observer::new(40.7128, -74.0060, 10.0),
            zone_name: "America/New_York".into(),
        })
        .expect("relocate");

    // The two 2026 transitions: 8 March springs forward, 1 November falls back.
    for (month, days) in [(3, 31), (11, 30)] {
        let mut rises: Vec<i64> = Vec::new();
        for day in 1..=days {
            let DayDetail::Moon(moon) = almanac
                .day_detail(
                    Graha::Chandra,
                    DateKey::new(2026, month, day).unwrap(),
                    System::Solar,
                )
                .expect("detail")
            else {
                panic!("moon detail");
            };

            if let Some(rise) = moon.moonrise {
                assert_eq!(
                    rise.day_offset, 0,
                    "2026-{month}-{day}: rise outside its day"
                );
                rises.push(rise.unix_ms);
            }
            if let Some(set) = moon.moonset {
                assert_eq!(set.day_offset, 0, "2026-{month}-{day}: set outside its day");
            }
        }

        const HOUR_MS: i64 = 3_600_000;
        for pair in rises.windows(2) {
            let gap = pair[1] - pair[0];
            assert!(
                (20 * HOUR_MS..27 * HOUR_MS).contains(&gap),
                "2026-{month}: {:.2} hours between consecutive moonrises, so one was dropped \
                 or counted twice",
                gap as f64 / HOUR_MS as f64
            );
        }
    }

    reset(&almanac);
}

#[test]
fn graha_months_find_stations_and_ingresses() {
    use chandra_almanac::events::EventKind;

    let almanac = almanac();
    reset(&almanac);

    let view = almanac
        .graha_month(Graha::Mangala, solar(2025, 2))
        .expect("month");
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

    let days = inside_graha(&view);
    let before = days.iter().find(|d| d.date.day == 20).unwrap();
    let after = days.iter().find(|d| d.date.day == 28).unwrap();
    assert!(before.retrograde, "Mangala retrograde before the station");
    assert!(!after.retrograde, "Mangala direct after the station");
}

#[test]
fn events_are_ordered_and_attributed_to_the_day_they_occur_on() {
    let almanac = almanac();
    reset(&almanac);

    let view = almanac
        .graha_month(Graha::Budha, solar(2026, 8))
        .expect("month");
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
    assert!(almanac.graha_month(Graha::Chandra, solar(2026, 8)).is_err());
}

#[test]
fn changing_the_ayanamsa_changes_positions_and_clears_the_cache() {
    let almanac = almanac();
    reset(&almanac);

    let lahiri = almanac
        .graha_month(Graha::Shani, solar(2026, 8))
        .expect("month");
    almanac
        .set_sidereal(SiderealConfig {
            ayanamsa: Ayanamsa::Raman,
            node_type: NodeType::True,
        })
        .expect("set ayanamsa");
    let raman = almanac
        .graha_month(Graha::Shani, solar(2026, 8))
        .expect("month");

    let difference = (inside_graha(&lahiri)[0].longitude - inside_graha(&raman)[0].longitude).abs();
    assert!(
        difference > 1.0,
        "Lahiri and Raman differ by about 1.4 degrees; got {difference}"
    );

    reset(&almanac);
    let back = almanac
        .graha_month(Graha::Shani, solar(2026, 8))
        .expect("month");
    assert!(
        (inside_graha(&back)[0].longitude - inside_graha(&lahiri)[0].longitude).abs() < 1e-9,
        "returning to Lahiri must reproduce the original figures"
    );
}

#[test]
fn changing_location_changes_rise_times() {
    let almanac = almanac();
    reset(&almanac);

    let date = DateKey::new(2026, 8, 20).unwrap();
    let DayDetail::Moon(here) = almanac
        .day_detail(Graha::Chandra, date, System::Solar)
        .expect("detail")
    else {
        panic!("moon detail");
    };

    almanac
        .set_location(Location {
            observer: Observer::new(-33.8688, 151.2093, 58.0),
            zone_name: "Australia/Sydney".into(),
        })
        .expect("relocate");
    let DayDetail::Moon(sydney) = almanac
        .day_detail(Graha::Chandra, date, System::Solar)
        .expect("detail")
    else {
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
    let _ = almanac.moon_month(solar(2031, 3)).expect("month");
    let cold = started.elapsed();

    let started = Instant::now();
    let _ = almanac.moon_month(solar(2031, 3)).expect("month");
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

/// A lunar month costs more, and must still fit.
///
/// It walks syzygies to find its own boundaries and resolves a sunrise for every
/// one of the 44 days the grid and its neighbours span. The tithis are then
/// shared: the second subject drawn on the same month must not pay for them
/// again, which is what the frame cache exists for.
#[test]
fn a_lunar_month_stays_inside_the_budget_and_is_computed_once() {
    let almanac = almanac();
    reset(&almanac);

    let cursor = MonthCursor {
        system: System::Amanta,
        ..solar(2031, 3)
    };

    let started = Instant::now();
    let month = almanac.moon_month(cursor).expect("month");
    let cold = started.elapsed();
    assert!(
        month.days.iter().all(|cell| cell.tithi.is_some()),
        "every cell of a lunar grid carries a tithi"
    );
    assert!(
        cold.as_millis() < 150,
        "cold lunar month took {cold:?}, budget 30ms with headroom"
    );

    // A different subject over the same month: the tithis are already resolved,
    // so this must not repeat the 44 sunrises.
    let started = Instant::now();
    let graha = almanac
        .graha_month(Graha::Mangala, cursor)
        .expect("graha month");
    let shared = started.elapsed();
    assert!(
        graha.days.iter().all(|cell| cell.tithi.is_some()),
        "a graha grid names the same lunar days"
    );
    assert!(
        shared < cold,
        "the second subject re-resolved the tithis: {shared:?} against {cold:?}"
    );

    // Warm, which the solar test has always measured and this one never did.
    // The resolution is the expensive half and the key was derived from its
    // answer, so a warm lunar month walked its syzygies and built its 42 civil
    // days again before the cache was consulted at all.
    let started = Instant::now();
    let _ = almanac.moon_month(cursor).expect("month");
    let warm = started.elapsed();
    assert!(
        warm.as_micros() < 5_000,
        "warm lunar month took {warm:?}, over the 5ms budget"
    );

    // And at an offset, where the syzygy walk from the anchor is longest.
    let far = MonthCursor {
        offset: 5,
        ..cursor
    };
    let _ = almanac.moon_month(far).expect("month");
    let started = Instant::now();
    let _ = almanac.moon_month(far).expect("month");
    let warm_far = started.elapsed();
    assert!(
        warm_far.as_micros() < 5_000,
        "warm lunar month five steps out took {warm_far:?}, over the 5ms budget"
    );

    // Solar mode names no tithi, and pays nothing for it.
    let solar_month = almanac.moon_month(solar(2031, 3)).expect("month");
    assert!(solar_month.days.iter().all(|cell| cell.tithi.is_none()));
}

#[test]
fn a_cold_graha_month_is_assembled_well_inside_the_budget() {
    let almanac = almanac();
    reset(&almanac);

    let started = Instant::now();
    let _ = almanac
        .graha_month(Graha::Shani, solar(2032, 5))
        .expect("month");
    let slow_body = started.elapsed();

    let started = Instant::now();
    let _ = almanac
        .graha_month(Graha::Budha, solar(2032, 5))
        .expect("month");
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

    let inside = almanac.moon_month(solar(2026, 8)).expect("month");
    assert_eq!(inside.source, Source::Swieph);

    let outside = almanac.moon_month(solar(1650, 8)).expect("month");
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
    let _ = almanac.moon_month(solar(2033, 7)).expect("month");
    line("moon month, cold", t.elapsed());

    let t = Instant::now();
    let _ = almanac.moon_month(solar(2033, 7)).expect("month");
    line("moon month, cached", t.elapsed());

    // A lunar month costs more: it walks syzygies to find its own boundaries,
    // and resolves a sunrise for all 44 days the grid and its neighbours span.
    let lunar = MonthCursor {
        system: System::Amanta,
        ..solar(2033, 7)
    };
    let t = Instant::now();
    let _ = almanac.moon_month(lunar).expect("month");
    line("lunar moon month, cold", t.elapsed());

    let t = Instant::now();
    let _ = almanac.graha_month(Graha::Mangala, lunar).expect("month");
    line("lunar graha month, frames cached", t.elapsed());

    let t = Instant::now();
    let _ = almanac
        .day_detail(
            Graha::Chandra,
            DateKey::new(2033, 7, 15).unwrap(),
            System::Amanta,
        )
        .expect("detail");
    line("lunar moon day detail", t.elapsed());

    for graha in [Graha::Budha, Graha::Mangala, Graha::Guru, Graha::Shani] {
        let t = Instant::now();
        let _ = almanac.graha_month(graha, solar(2033, 7)).expect("month");
        line(&format!("{} month, cold", graha.name()), t.elapsed());
    }

    let t = Instant::now();
    let _ = almanac
        .day_detail(
            Graha::Chandra,
            DateKey::new(2033, 7, 15).unwrap(),
            System::Solar,
        )
        .expect("detail");
    line("moon day detail", t.elapsed());

    let t = Instant::now();
    let _ = almanac
        .day_detail(
            Graha::Shani,
            DateKey::new(2033, 7, 15).unwrap(),
            System::Solar,
        )
        .expect("detail");
    line("Shani day detail (slowest body)", t.elapsed());

    let t = Instant::now();
    let _ = almanac
        .now(1_755_000_000_000, &Graha::ALL)
        .expect("snapshot");
    line("tray snapshot, all nine grahas", t.elapsed());
}

// ---------------------------------------------------------------- lunar months

use chandra_almanac::lunar::MonthSystem;

/// Julian Day of local noon on a date in the almanac's zone.
fn noon_jd(date: (i16, i8, i8)) -> f64 {
    chandra_ephemeris::julian_day(date.0 as i32, date.1 as u32, date.2 as u32, 0.25)
}

#[test]
fn the_amanta_month_of_august_2026_is_shravana() {
    let almanac = almanac();
    reset(&almanac);

    // The new moon of 12 August 2026 falls with the Sun in Karka, and the month
    // it opens is Shravana.
    let month = almanac
        .lunar_month_at(noon_jd((2026, 8, 20)), MonthSystem::Amanta)
        .expect("month");

    assert_eq!(month.name, "Shravana");
    assert!(!month.adhika);
    assert_eq!(month.first_day.month, 8, "Shravana 2026 opens in August");
}

#[test]
fn the_purnimanta_month_carries_the_same_name() {
    let almanac = almanac();
    reset(&almanac);

    // A purnimanta month begins a fortnight earlier than the amanta month of the
    // same name and contains its opening new moon.
    let month = almanac
        .lunar_month_at(noon_jd((2026, 8, 20)), MonthSystem::Purnimanta)
        .expect("month");

    assert_eq!(month.name, "Shravana");
    assert!(
        month.first_day.month == 7 || month.first_day.day < 12,
        "purnimanta Shravana starts before the new moon, got {:?}",
        month.first_day
    );
}

#[test]
fn adhika_shravana_2023_is_detected() {
    let almanac = almanac();
    reset(&almanac);

    // 2023 had an intercalary Shravana running 18 July to 16 August, followed by
    // the regular Shravana. This is the case the sankranti rule exists for.
    let intercalary = almanac
        .lunar_month_at(noon_jd((2023, 8, 1)), MonthSystem::Amanta)
        .expect("month");
    assert_eq!(intercalary.name, "Shravana");
    assert!(
        intercalary.adhika,
        "the lunar month spanning 1 August 2023 is Adhika Shravana"
    );
    assert_eq!(intercalary.display_name(), "Adhika Shravana");

    // The month that follows repeats the name without the prefix.
    let regular = almanac
        .lunar_month_at(noon_jd((2023, 9, 1)), MonthSystem::Amanta)
        .expect("month");
    assert_eq!(regular.name, "Shravana");
    assert!(!regular.adhika, "the second Shravana is the regular one");
}

#[test]
fn an_intercalary_month_contains_no_sankranti() {
    let almanac = almanac();
    reset(&almanac);

    // The rule stated directly: an adhika month is one the Sun crosses no rashi
    // boundary inside. Checked against the independent sankranti search rather
    // than against the same comparison the detection uses.
    let intercalary = almanac
        .lunar_month_at(noon_jd((2023, 8, 1)), MonthSystem::Amanta)
        .expect("month");
    assert_eq!(
        almanac
            .sankrantis_between(intercalary.start_jd, intercalary.end_jd)
            .expect("sankrantis")
            .len(),
        0
    );

    let ordinary = almanac
        .lunar_month_at(noon_jd((2026, 8, 20)), MonthSystem::Amanta)
        .expect("month");
    assert_eq!(
        almanac
            .sankrantis_between(ordinary.start_jd, ordinary.end_jd)
            .expect("sankrantis")
            .len(),
        1,
        "an ordinary lunar month contains exactly one sankranti"
    );
}

#[test]
fn lunar_months_are_contiguous_and_of_plausible_length() {
    let almanac = almanac();
    reset(&almanac);

    for system in [MonthSystem::Amanta, MonthSystem::Purnimanta] {
        let mut month = almanac
            .lunar_month_at(noon_jd((2026, 1, 15)), system)
            .expect("month");

        for step in 0..14 {
            let next = almanac.lunar_month_shift(&month, 1, system).expect("next");

            let length = next.start_jd - month.start_jd;
            assert!(
                (29.2..=29.9).contains(&length),
                "{system:?} month {step} is {length} days, outside the synodic range"
            );

            // Civil day coverage must not gap or overlap.
            let after_last = chandra_almanac::time::DateKey::new(
                month.last_day.year,
                month.last_day.month,
                month.last_day.day,
            )
            .expect("date");
            assert_eq!(
                almanac.day_after(after_last).expect("day after"),
                next.first_day,
                "{system:?} month {step} does not run up to the next"
            );

            month = next;
        }
    }
}

#[test]
fn shifting_forward_and_back_returns_the_same_month() {
    let almanac = almanac();
    reset(&almanac);

    let month = almanac
        .lunar_month_at(noon_jd((2026, 8, 20)), MonthSystem::Amanta)
        .expect("month");
    let round_trip = almanac
        .lunar_month_shift(&month, 3, MonthSystem::Amanta)
        .and_then(|m| almanac.lunar_month_shift(&m, -3, MonthSystem::Amanta))
        .expect("round trip");

    assert_eq!(round_trip.name, month.name);
    assert!((round_trip.start_jd - month.start_jd).abs() < 1e-6);
}

#[test]
fn every_lunar_month_name_occurs_across_a_year() {
    let almanac = almanac();
    reset(&almanac);

    let mut month = almanac
        .lunar_month_at(noon_jd((2026, 4, 1)), MonthSystem::Amanta)
        .expect("month");
    let mut seen = std::collections::BTreeSet::new();

    for _ in 0..13 {
        seen.insert(month.name);
        month = almanac
            .lunar_month_shift(&month, 1, MonthSystem::Amanta)
            .expect("next");
    }

    assert_eq!(
        seen.len(),
        12,
        "a year of lunar months should use all twelve names, saw {seen:?}"
    );
}

/// A mark on a cell and the reading inside it must be the same fact.
///
/// The grid marks a day combust; opening that day shows the separation from the
/// Sun that decided it. Judging them at different instants would let a cell be
/// marked while the day it opens said nothing, which is the kind of discrepancy
/// nobody reports as a bug and everybody stops trusting.
#[test]
fn the_combustion_mark_and_the_day_it_opens_agree() {
    let almanac = almanac();
    reset(&almanac);

    // August 2026 contains a new moon, so the Moon is combust on some of these
    // days and not on others: the test would pass vacuously otherwise.
    let month = almanac.moon_month(solar(2026, 8)).expect("moon month");
    let mut combust_days = 0;

    for cell in inside(&month) {
        let DayDetail::Moon(day) = almanac
            .day_detail(Graha::Chandra, cell.date, System::Solar)
            .expect("moon detail")
        else {
            panic!("moon detail");
        };
        assert_eq!(
            cell.combust, day.combustion.combust,
            "{:?}: cell and day disagree about combustion",
            cell.date
        );
        assert_eq!(
            day.combustion.orb,
            Some(12.0),
            "the Moon's orb is 12 degrees"
        );
        assert!(
            day.combustion.combust == (day.combustion.separation < 12.0),
            "{:?}: combustion must follow from the separation shown",
            cell.date
        );
        combust_days += usize::from(cell.combust);
    }
    assert!(
        combust_days > 0 && combust_days < inside(&month).len(),
        "expected both combust and non-combust days, got {combust_days}"
    );

    for graha in [Graha::Budha, Graha::Shukra, Graha::Guru] {
        let month = almanac
            .graha_month(graha, solar(2026, 8))
            .expect("graha month");
        for cell in inside_graha(&month) {
            let DayDetail::Graha(day) = almanac
                .day_detail(graha, cell.date, System::Solar)
                .expect("detail")
            else {
                panic!("graha detail");
            };
            assert_eq!(
                cell.combust, day.combustion.combust,
                "{graha:?} {:?}: cell and day disagree about combustion",
                cell.date
            );
        }
    }

    // The Sun cannot be combust by itself, and the nodes have no visibility to
    // lose. They carry no orb, so the day view omits the block entirely.
    for graha in [Graha::Surya, Graha::Rahu, Graha::Ketu] {
        let DayDetail::Graha(day) = almanac
            .day_detail(graha, DateKey::new(2026, 8, 14).unwrap(), System::Solar)
            .expect("detail")
        else {
            panic!("graha detail");
        };
        assert_eq!(day.combustion.orb, None, "{graha:?} has no combustion orb");
        assert!(!day.combustion.combust);
    }
}

/// The month system is a property of the calendar, not of the subject.
///
/// Every subject's month must span the same civil days and carry the same name,
/// so switching from the Moon to a graha changes what is plotted and nothing
/// else. The cursor carrying the system is built in exactly one place
/// (`src-tauri/src/state.rs::cursor`, from settings), which is what makes this
/// hold for the application as well as for the library.
#[test]
fn every_subject_shares_one_month_system() {
    let almanac = almanac();
    reset(&almanac);

    for system in [System::Solar, System::Amanta, System::Purnimanta] {
        let cursor = MonthCursor {
            system,
            ..solar(2026, 8)
        };
        let moon = almanac.moon_month(cursor).expect("moon month");

        for graha in [Graha::Mangala, Graha::Shukra, Graha::Shani] {
            let month = almanac.graha_month(graha, cursor).expect("graha month");
            assert_eq!(month.label, moon.label, "{graha:?} {system:?}: label");
            assert_eq!(
                inside_graha(&month)
                    .iter()
                    .map(|day| day.date)
                    .collect::<Vec<_>>(),
                inside(&moon).iter().map(|day| day.date).collect::<Vec<_>>(),
                "{graha:?} {system:?}: days"
            );
        }
    }
}

/// The grid's tithi numbers must be the ones a panchang prints.
///
/// Checked against the shape of a lunar month rather than against a table of
/// dates: an amanta month opens on Shukla Pratipada and closes on Amavasya, a
/// purnimanta month opens on Krishna Pratipada and closes on Purnima, and in
/// both the numbers advance by one a day except where a tithi is skipped or
/// repeated - which is exactly what the marks are for.
#[test]
fn a_lunar_month_runs_from_one_end_of_the_tithis_to_the_other() {
    use chandra_almanac::tithi::Paksha;

    let almanac = almanac();
    reset(&almanac);

    let month = |system| {
        almanac
            .moon_month(MonthCursor {
                system,
                ..solar(2026, 8)
            })
            .expect("month")
    };

    let amanta = month(System::Amanta);
    let days = inside(&amanta);
    let first = days[0].tithi.as_ref().expect("lunar cells carry a tithi");
    let last = days[days.len() - 1].tithi.as_ref().expect("tithi");

    assert_eq!(
        amanta.label, "Shravana VS 2083",
        "the era is named: a bare 2083 reads as a Gregorian year"
    );
    assert_eq!(first.paksha, Paksha::Shukla);
    assert_eq!(first.number, 1, "an amanta month opens on Shukla Pratipada");
    assert_eq!(last.paksha, Paksha::Krishna);
    assert_eq!(last.number, 15, "and closes on Amavasya");
    assert_eq!(last.name, "Amavasya");

    let purnimanta = month(System::Purnimanta);
    let days = inside(&purnimanta);
    let first = days[0].tithi.as_ref().expect("tithi");
    let last = days[days.len() - 1].tithi.as_ref().expect("tithi");

    assert_eq!(first.paksha, Paksha::Krishna);
    assert_eq!(
        first.number, 1,
        "a purnimanta month opens on Krishna Pratipada"
    );
    assert_eq!(last.paksha, Paksha::Shukla);
    assert_eq!(last.number, 15, "and closes on Purnima");
    assert_eq!(last.name, "Purnima");

    // Purnima and Amavasya each fall exactly once inside an amanta month.
    let syzygies = inside(&amanta)
        .iter()
        .filter_map(|cell| cell.tithi.as_ref())
        .filter(|tithi| tithi.number == 15)
        .count();
    assert_eq!(syzygies, 2, "one Purnima and one Amavasya");
}

/// Every cell of the grid advances by one tithi, and where it does not, it says
/// so.
///
/// This is the invariant the kshaya dot and the vriddhi rule exist to protect. A
/// jump with no kshaya recorded, or a repeat with no vriddhi, would be a bug the
/// user would read as a wrong number.
#[test]
fn a_jump_or_a_repeat_in_the_tithi_numbers_is_always_marked() {
    use chandra_almanac::tithi::{Tithi, Vriddhi};

    let almanac = almanac();
    reset(&almanac);

    let mut jumps = 0;
    let mut repeats = 0;

    // A year of lunar months, so both cases are certain to occur: a synodic
    // month holds 30 tithis in 29.53 days, so roughly one a month is one or the
    // other.
    for offset in 0..13 {
        let month = almanac
            .moon_month(MonthCursor {
                system: System::Amanta,
                offset,
                ..solar(2026, 8)
            })
            .expect("month");

        for pair in month.days.windows(2) {
            let here = pair[0].tithi.as_ref().expect("tithi");
            let next = pair[1].tithi.as_ref().expect("tithi");
            let steps = Tithi::from_index(here.index)
                .unwrap()
                .steps_to(Tithi::from_index(next.index).unwrap());

            match steps {
                0 => {
                    repeats += 1;
                    assert_eq!(
                        here.vriddhi,
                        Some(Vriddhi::First),
                        "{:?} repeats its tithi without being marked",
                        pair[0].date
                    );
                    assert_eq!(next.vriddhi, Some(Vriddhi::Second), "{:?}", pair[1].date);
                    assert!(here.kshaya.is_empty(), "a repeat cannot also skip");
                }
                1 => assert!(
                    here.kshaya.is_empty(),
                    "{:?} advances by one but claims a skip",
                    pair[0].date
                ),
                more => {
                    jumps += 1;
                    assert_eq!(
                        here.kshaya.len(),
                        more as usize - 1,
                        "{:?} jumps {more} tithis and must name every one it passed",
                        pair[0].date
                    );
                    // The named tithis are the ones actually in between.
                    let mut expected = Tithi::from_index(here.index).unwrap().next();
                    for skipped in &here.kshaya {
                        assert_eq!(skipped.index, expected.index());
                        assert_eq!(skipped.name, expected.full_name());
                        expected = expected.next();
                    }
                }
            }
        }
    }

    assert!(jumps > 0, "a year of lunar months contains a kshaya tithi");
    assert!(
        repeats > 0,
        "a year of lunar months contains a vriddhi tithi"
    );
}

/// The grid and the day it opens must name the same tithi.
///
/// The cell reads the tithi at sunrise; the day view resolves the spans that
/// touch the day and marks one prevailing. Two routines, one answer - or the
/// number in the grid means nothing.
///
/// A year of months, not one. The disagreement is a boundary falling between
/// the reference instant and the nearest sample of the hourly scan grid, which
/// is rare enough that August 2026 happens to contain none of them: the single
/// month this used to walk passed while the thing it guards was broken.
#[test]
fn the_cell_and_the_day_it_opens_name_the_same_tithi() {
    let almanac = almanac();
    reset(&almanac);

    for offset in 0..13 {
        let month = almanac
            .moon_month(MonthCursor {
                system: System::Amanta,
                offset,
                ..solar(2026, 8)
            })
            .expect("month");

        for cell in inside(&month) {
            let tithi = cell.tithi.as_ref().expect("tithi");
            let DayDetail::Moon(day) = almanac
                .day_detail(Graha::Chandra, cell.date, System::Amanta)
                .expect("detail")
            else {
                panic!("moon detail");
            };
            let panchanga = day.panchanga.expect("a lunar day carries a panchanga");
            let prevailing = panchanga
                .tithis
                .iter()
                .find(|span| span.prevailing)
                .expect("exactly one span prevails");

            assert_eq!(
                prevailing.index, tithi.index,
                "{:?}: cell and day disagree about the tithi",
                cell.date
            );
            assert_eq!(prevailing.number, tithi.number);
            assert_eq!(prevailing.paksha, tithi.paksha);
            assert_eq!(prevailing.name, tithi.name);

            // Every span's boundaries are ordered, and the day's spans are
            // contiguous: one ends where the next begins.
            for span in &panchanga.tithis {
                if let (Some(entry), Some(exit)) = (span.entry, span.exit) {
                    assert!(entry.unix_ms < exit.unix_ms, "{:?} {span:?}", cell.date);
                }
            }
            for pair in panchanga.tithis.windows(2) {
                if let (Some(exit), Some(entry)) = (pair[0].exit, pair[1].entry) {
                    assert!(
                        (exit.unix_ms - entry.unix_ms).abs() < 1000,
                        "{:?}: a gap between consecutive tithis",
                        cell.date
                    );
                }
            }
        }
    }

    // Solar mode carries no panchanga at all: it is absent, not empty.
    let DayDetail::Moon(day) = almanac
        .day_detail(
            Graha::Chandra,
            DateKey::new(2026, 8, 20).unwrap(),
            System::Solar,
        )
        .expect("detail")
    else {
        panic!("moon detail");
    };
    assert!(day.panchanga.is_none());
}

/// A day's own longitude and its own prevailing span must name one division.
///
/// The day view prints the longitude at the reference instant and, beside it,
/// the nakshatra and rashi it marks prevailing. Those come from two routines:
/// one reads the position directly, the other picks a span. When they disagree
/// the panel states a longitude in Krittika and labels it Bharani.
///
/// The slow bodies are the ones that matter. Guru, Shani and the nodes are
/// scanned at a whole day, so a boundary crossed between midnight and sunrise
/// sits inside a single sample interval and cannot be resolved by comparing the
/// reference against the samples.
#[test]
fn a_graha_day_and_its_prevailing_span_name_the_same_division() {
    use chandra_almanac::zodiac::{pada, Nakshatra, Rashi};

    let almanac = almanac();
    reset(&almanac);

    for graha in [Graha::Guru, Graha::Shani, Graha::Rahu, Graha::Ketu] {
        // A year, because a slow body crosses only a handful of boundaries in
        // one and the defect needs a crossing to show at all.
        for month in 1..=12i8 {
            for day in 1..=chandra_almanac::time::days_in_month(2024, month).expect("month length")
            {
                let date = DateKey::new(2024, month, day as i8).expect("date");
                let DayDetail::Graha(detail) = almanac
                    .day_detail(graha, date, System::Solar)
                    .expect("detail")
                else {
                    panic!("graha detail");
                };

                let nakshatra = detail
                    .nakshatras
                    .iter()
                    .find(|span| span.prevailing)
                    .expect("exactly one nakshatra prevails");
                assert_eq!(
                    nakshatra.nakshatra,
                    Nakshatra::from_longitude(detail.longitude),
                    "{graha:?} {date:?}: longitude {} is not in {}",
                    detail.longitude,
                    nakshatra.name
                );
                assert_eq!(
                    nakshatra.pada,
                    pada(detail.longitude),
                    "{graha:?} {date:?}: pada belongs to another nakshatra"
                );

                let rashi = detail
                    .rashis
                    .iter()
                    .find(|span| span.prevailing)
                    .expect("exactly one rashi prevails");
                assert_eq!(
                    rashi.rashi,
                    Rashi::from_longitude(detail.longitude),
                    "{graha:?} {date:?}: longitude {} is not in {}",
                    detail.longitude,
                    rashi.name
                );
            }
        }
    }
}

/// A month computed under one ayanamsa must never be served under another.
///
/// Every command runs on a blocking pool, so a settings change genuinely
/// overlaps a month being assembled. The cache is keyed on the month, not on the
/// configuration, and correctness rests entirely on the generation counter: a
/// reader that captured no generation before it started stamps its result with
/// whatever generation is current when it finishes, and a month computed partly
/// under Raman then sits in the cache as a Lahiri one and never expires.
#[test]
fn a_month_computed_under_one_ayanamsa_is_never_served_under_another() {
    const MONTHS: i32 = 8;

    let almanac = almanac();
    reset(&almanac);

    let shared = &*almanac;
    let cursor = |offset: i32| MonthCursor {
        offset,
        ..solar(2026, 8)
    };
    let lahiri = SiderealConfig {
        ayanamsa: Ayanamsa::Lahiri,
        node_type: NodeType::True,
    };
    let raman = SiderealConfig {
        ayanamsa: Ayanamsa::Raman,
        node_type: NodeType::True,
    };

    std::thread::scope(|scope| {
        scope.spawn(|| {
            for offset in 0..MONTHS {
                let _ = shared.graha_month(Graha::Budha, cursor(offset));
            }
        });

        // The reader is assembling months for tens of milliseconds. Changing
        // the ayanamsa inside that window is exactly what the settings pane
        // does, and the change lands between two of its ephemeris calls.
        std::thread::sleep(Duration::from_millis(10));
        shared.set_sidereal(raman).expect("sidereal");
        std::thread::sleep(Duration::from_millis(5));
        shared.set_sidereal(lahiri).expect("sidereal");
    });

    let served: Vec<_> = (0..MONTHS)
        .map(|offset| {
            almanac
                .graha_month(Graha::Budha, cursor(offset))
                .expect("graha month")
        })
        .collect();

    // What those months are, from a cache that is certainly cold.
    almanac.set_sidereal(raman).expect("sidereal");
    reset(&almanac);
    let expected: Vec<_> = (0..MONTHS)
        .map(|offset| {
            almanac
                .graha_month(Graha::Budha, cursor(offset))
                .expect("graha month")
        })
        .collect();

    for (month, reference) in served.iter().zip(expected.iter()) {
        for (cell, truth) in month.days.iter().zip(reference.days.iter()) {
            assert!(
                (cell.longitude - truth.longitude).abs() < 1e-9,
                "{:?}: served {} where Lahiri gives {}",
                cell.date,
                cell.longitude,
                truth.longitude
            );
        }
    }
}

/// The month a day opens is the month whose grid draws it.
///
/// Two routes decide "which month is this day in": the syzygy the anchor falls
/// after, and the first-sunrise-after-syzygy rule that fixes the month's civil
/// extent. They differ by up to a day, and where they do the panel opened on a
/// month that did not contain today - drawn as a dimmed leading cell, or on
/// 31 May 2026 amanta absent from all 42.
#[test]
fn every_day_opens_the_lunar_month_that_draws_it() {
    let almanac = almanac();
    reset(&almanac);

    for system in [System::Amanta, System::Purnimanta] {
        for month in 1..=12i8 {
            for day in 1..=chandra_almanac::time::days_in_month(2026, month).expect("month length")
            {
                let date = DateKey::new(2026, month, day as i8).expect("date");
                let view = almanac
                    .moon_month(MonthCursor {
                        anchor_unix_ms: (chandra_ephemeris::jd_to_unix_seconds(noon_jd((
                            2026, month, day as i8,
                        ))) * 1000.0) as i64,
                        offset: 0,
                        system,
                        first_weekday: 0,
                    })
                    .expect("month");

                let cell = view
                    .days
                    .iter()
                    .find(|cell| cell.date == date)
                    .unwrap_or_else(|| {
                        panic!(
                            "{system:?} {date:?}: today is not among the 42 cells of {}",
                            view.label
                        )
                    });
                assert!(
                    cell.in_month,
                    "{system:?} {date:?}: today is drawn as a neighbouring month's cell in {}",
                    view.label
                );
            }
        }
    }
}

/// A sunrise count of zero and no count at all are different answers.
///
/// Zero says no civil day is named after this tithi, which is a kshaya and is
/// drawn as one. Inside a polar night there are no sunrises to count against at
/// all, which is a fact about the latitude: reporting zero there captioned every
/// tithi of every day a kshaya, while the grid cell beside it fell back to local
/// noon and was right.
#[test]
fn a_polar_night_reports_no_sunrise_count_rather_than_none_at_all() {
    let almanac = almanac();
    reset(&almanac);
    almanac
        .set_location(Location {
            // Longyearbyen: the Sun stays below the horizon from November to
            // late January.
            observer: Observer::new(78.2232, 15.6469, 20.0),
            zone_name: "Arctic/Longyearbyen".into(),
        })
        .expect("relocate");

    let DayDetail::Moon(day) = almanac
        .day_detail(
            Graha::Chandra,
            DateKey::new(2026, 1, 5).unwrap(),
            System::Amanta,
        )
        .expect("detail")
    else {
        panic!("moon detail");
    };

    let panchanga = day.panchanga.expect("a lunar day carries a panchanga");
    assert!(panchanga.sunrise.is_none(), "the Sun does not rise here");
    for span in &panchanga.tithis {
        assert_eq!(
            span.sunrises, None,
            "{span:?} claims a sunrise count where the Sun never rose"
        );
    }

    // The same day at a latitude where the Sun does rise counts them, and a
    // year of them contains a real kshaya reported as an honest zero.
    reset(&almanac);
    let mut counted = 0;
    let mut kshaya = 0;
    for offset in 0..13 {
        let month = almanac
            .moon_month(MonthCursor {
                system: System::Amanta,
                offset,
                ..solar(2026, 8)
            })
            .expect("month");

        for cell in inside(&month) {
            let DayDetail::Moon(day) = almanac
                .day_detail(Graha::Chandra, cell.date, System::Amanta)
                .expect("detail")
            else {
                panic!("moon detail");
            };
            for span in &day.panchanga.expect("panchanga").tithis {
                match span.sunrises {
                    Some(0) => kshaya += 1,
                    Some(_) => counted += 1,
                    None => panic!("{:?}: no count where the Sun rises", cell.date),
                }
            }
        }
    }
    assert!(counted > 0);
    assert!(
        kshaya > 0,
        "a year of lunar months contains a tithi no day is named after"
    );
}

/// The jump overlay's index, which is the pointer route to a year.
///
/// A Vikram Samvat year holds twelve months or thirteen, and which it is
/// depends on whether a lunation fitted inside one solar rashi. The index walks
/// out from the month in hand until the year changes rather than counting to
/// twelve, and these are the two facts that walk has to get right.
#[test]
fn a_lunar_year_holds_the_months_it_actually_holds() {
    let almanac = almanac();

    // VS 2083 contains Adhika Jyeshtha: thirteen months, and the intercalary
    // one sits before the month whose name it repeats.
    let cursor = MonthCursor {
        system: System::Amanta,
        ..solar(2026, 8)
    };
    let index = almanac.month_index(cursor).expect("index");

    assert_eq!(index.year, "VS 2083");
    assert_eq!(
        index.months.len(),
        13,
        "an intercalary year has thirteen months, and all thirteen must be reachable"
    );

    let adhika = index
        .months
        .iter()
        .position(|month| month.adhika)
        .expect("VS 2083 has an adhika masa");
    assert_eq!(
        index.months[adhika].name, "Adhika Jyeshtha",
        "the display name carries the qualifier"
    );
    assert_eq!(
        index.months[adhika + 1].name,
        "Jyeshtha",
        "an adhika masa precedes the month whose name it repeats"
    );

    // Every offset resolves to the month it claims, against the same anchor.
    for month in &index.months {
        let at = MonthCursor {
            offset: month.offset,
            ..cursor
        };
        let resolved = almanac.moon_month(at).expect("month at the listed offset");
        assert_eq!(
            resolved.label,
            format!("{} {}", month.name, index.year),
            "offset {} did not land on {}",
            month.offset,
            month.name
        );
    }

    // Paging lands in the neighbouring years, whatever their length.
    let before = almanac
        .month_index(MonthCursor {
            offset: index.previous_year,
            ..cursor
        })
        .expect("previous year");
    assert_eq!(before.year, "VS 2082");

    let after = almanac
        .month_index(MonthCursor {
            offset: index.next_year,
            ..cursor
        })
        .expect("next year");
    assert_eq!(after.year, "VS 2084");
    assert_eq!(after.months.len(), 12, "an ordinary year has twelve");
}

/// A Gregorian year is twelve months and needs no search, but the offsets still
/// have to be right: the index is applied by setting the cursor to them.
#[test]
fn a_solar_year_indexes_january_to_december() {
    let almanac = almanac();

    // Cursor on August, so the January offset is negative and December's
    // positive - the case that a naive `offset + n` would get wrong.
    let index = almanac.month_index(solar(2026, 8)).expect("index");

    assert_eq!(index.year, "2026");
    assert_eq!(index.months.len(), 12);
    assert_eq!(index.months[0].name, "January");
    assert_eq!(index.months[11].name, "December");
    assert!(
        index.months.iter().all(|month| !month.adhika),
        "a Gregorian year has no intercalary month"
    );

    for month in &index.months {
        let at = MonthCursor {
            offset: month.offset,
            ..solar(2026, 8)
        };
        let resolved = almanac.moon_month(at).expect("month at the listed offset");
        assert_eq!(resolved.label, format!("{} 2026", month.name));
    }

    assert_eq!(index.previous_year, index.months[0].offset - 1);
    assert_eq!(index.next_year, index.months[11].offset + 1);
}
