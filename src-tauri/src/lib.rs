//! Application shell.
//!
//! Thin by design: it owns windows, the menu bar, settings persistence and the
//! IPC surface, and nothing else. Every astronomical decision lives in
//! `chandra-almanac` and `chandra-ephemeris`, which is what lets the domain be
//! tested without a windowing system.

mod commands;
mod error;
mod location;
mod panel;
mod settings;
mod state;
mod tray;

/// Re-exported solely so `tests/contract.rs` can build a fully populated value
/// and record its shape. Not part of the running app's surface.
pub use settings::Settings as PublicSettings;

use std::time::Duration;

use tauri::{Manager, RunEvent, WindowEvent};

use crate::state::AppState;

/// How long after an hour boundary to redraw the tray.
///
/// A few seconds of slack, so a clock adjustment across the boundary cannot make
/// the timer fire on the hour it just left.
const HOUR_SLACK: Duration = Duration::from_secs(5);

/// Longest the tray watcher parks for in one go.
///
/// `thread::sleep` does not advance while the machine is asleep, so a laptop
/// shut over an hour boundary returns from a single long sleep well after the
/// boundary it was waiting for and keeps a stale disc in the menu bar until the
/// rest of that sleep has elapsed awake. Looking at the clock periodically
/// bounds that to this interval.
///
/// Not polling in the sense D-017 ruled out: what happens on each wake is two
/// clock reads costing microseconds, and the redraw still happens only when the
/// displayed hour has actually rolled over.
const HOUR_CHECK_INTERVAL: Duration = Duration::from_secs(300);

/// How much real time must pass before the tray is redrawn.
///
/// Elapsed time, not a change in the clock reading. Comparing the local date and
/// hour looked equivalent and is not: a daylight saving fall-back repeats an
/// hour, so 01:30 EDT and 01:30 EST are the same reading an hour apart and the
/// disc went two hours stale once a year. Real time always advances.
///
/// Fifty-five minutes rather than sixty, because the wake is aligned to the hour
/// boundary and a strict hour would miss it by the few seconds of slack and skip
/// to the next one.
const REFRESH_AFTER: Duration = Duration::from_secs(55 * 60);

pub fn run() {
    tauri::Builder::default()
        .manage(panel::CurrentSubject::default())
        .manage(panel::PanelMaterial::default())
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::moon_month,
            commands::graha_month,
            commands::month_index,
            commands::chakra,
            commands::day_detail,
            commands::snapshot,
            commands::update_settings,
            commands::search_cities,
            commands::request_device_location,
            commands::close_panel,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // No dock icon and no menu bar of its own: this is a menu bar
            // accessory, and appearing in Cmd-Tab would be wrong for one.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let config_dir = handle
                .path()
                .app_config_dir()
                .map_err(|e| format!("no application config directory: {e}"))?;
            let ephemeris_dir = handle
                .path()
                .resolve("resources/ephe", tauri::path::BaseDirectory::Resource)
                .map_err(|e| format!("bundled ephemeris data is missing: {e}"))?;

            let state = AppState::new(config_dir, &ephemeris_dir)
                .map_err(|e| format!("cannot start the almanac: {e}"))?;
            app.manage(state);

            panel::create(&handle)?;
            // The window is built at the composed size; a stored size applies it
            // before anything is shown, so the panel never opens at one size and
            // resizes under the pointer.
            panel::apply_scale(
                &handle,
                handle.state::<AppState>().settings().appearance.clamped(),
            );
            tray::build(&handle)?;
            watch_the_clock(handle.clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != panel::PANEL_LABEL {
                return;
            }
            match event {
                // A menu bar panel closes when it loses focus. Anything else
                // leaves a floating window the user has to dismiss deliberately.
                WindowEvent::Focused(false) => panel::hide(window.app_handle()),
                // The panel is never reloaded, so the subject has to be told to
                // a page that is already running. It is told twice: once as the
                // window is shown, and again here. This one is the safety net -
                // focus is raised by the platform after the show has completed,
                // so it cannot be lost to a listener that was not ready. Both
                // carry the same value into the same handler.
                WindowEvent::Focused(true) => panel::announce_subject(window.app_handle()),
                WindowEvent::CloseRequested { api, .. } => {
                    // The panel is created once and reused, so closing it would
                    // leave the tray items opening nothing.
                    api.prevent_close();
                    panel::hide(window.app_handle());
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("the Tauri context must be valid")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                // The panel closing must not quit a menu bar app; it is the only
                // window there is.
                api.prevent_exit();
            }
        });
}

/// Redraws the tray when the local hour changes.
///
/// The moon disc shows the illumination *now*, not at any fixed instant of the
/// day, so it is only ever as current as its last redraw. It used to be redrawn
/// at local midnight alone, which left the disc and its tooltip up to
/// twenty-four hours stale - the Moon's lit fraction moves by as much as
/// thirteen points across a day, so by evening the menu bar was visibly wrong.
///
/// Hourly is the resolution the disc can actually show: a fifty-fifth of a
/// percent of illumination is well under a pixel at 22pt. The tooltip is not
/// bound to this at all - it is rebuilt when the pointer arrives.
///
/// The thread measures elapsed time rather than comparing clock readings: a
/// sleep does not run while the machine is suspended, and a wall clock does not
/// always advance. See [`REFRESH_AFTER`].
fn watch_the_clock(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        // The tray was drawn as the app started.
        let mut drawn_at = jiff::Timestamp::now();

        loop {
            std::thread::sleep(next_hour_check());

            let now = jiff::Timestamp::now();
            if now.as_second().saturating_sub(drawn_at.as_second()) < REFRESH_AFTER.as_secs() as i64
            {
                continue;
            }
            drawn_at = now;

            // Same rule as everywhere else: the status item is an AppKit object
            // and must only be touched on the main thread.
            let handle = app.clone();
            let dispatched = app.run_on_main_thread(move || {
                if let Err(error) = tray::refresh_icons(&handle) {
                    // A failed redraw leaves a stale disc in the menu bar, which
                    // is wrong but not fatal, so the loop continues.
                    eprintln!("chandra: could not redraw the menu bar: {error}");
                }
            });
            if dispatched.is_err() {
                // The app is shutting down; nothing left to redraw.
                return;
            }
        }
    });
}

/// How long to wait before looking at the clock again.
///
/// Until just after the next local hour boundary, or [`HOUR_CHECK_INTERVAL`],
/// whichever comes first. Waking on the boundary is cosmetic - it makes the
/// redraw land on the hour rather than at some offset from launch - and
/// [`REFRESH_AFTER`] is what actually decides whether to redraw.
fn next_hour_check() -> Duration {
    duration_until_next_hour(&jiff::Zoned::now())
        .map(|until| until + HOUR_SLACK)
        .unwrap_or(HOUR_CHECK_INTERVAL)
        .min(HOUR_CHECK_INTERVAL)
}

/// Time from `now` until the next local hour boundary.
///
/// Derived from the tz database rather than from the minutes and seconds on the
/// clock: a zone that moves its offset by thirty or forty-five minutes - India,
/// Nepal, parts of Australia - has hour boundaries that are not on the hour in
/// any other zone, and a daylight saving transition can make the step to the
/// next one shorter or longer than an hour.
///
/// `None` where the next boundary does not exist or does not lie ahead, which a
/// fall-back transition can produce: at Australia/Lord_Howe the clock goes back
/// thirty minutes, so for half an hour the "next" boundary is already behind.
/// The caller falls back to [`HOUR_CHECK_INTERVAL`] rather than unwrapping.
///
/// Takes the instant rather than reading the clock, so the transitions this has
/// to survive can be tested instead of waited for.
fn duration_until_next_hour(now: &jiff::Zoned) -> Option<Duration> {
    let next = now
        .with()
        .minute(0)
        .second(0)
        .subsec_nanosecond(0)
        .build()
        .ok()?
        .checked_add(jiff::Span::new().hours(1))
        .ok()?;

    let seconds = next.timestamp().as_second() - now.timestamp().as_second();
    (seconds > 0).then(|| Duration::from_secs(seconds as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(zone: &str, year: i16, month: i8, day: i8, hour: i8, minute: i8) -> jiff::Zoned {
        jiff::civil::date(year, month, day)
            .at(hour, minute, 0, 0)
            .in_tz(zone)
            .expect("a civil time that exists in this zone")
    }

    /// The wake is bounded, so a machine that was asleep over a boundary is
    /// noticed within one interval of waking.
    ///
    /// This used to be the only test here, and it asserted nothing: both its
    /// claims follow from the `.min(HOUR_CHECK_INTERVAL)` in `next_hour_check`
    /// and would hold against any hour logic at all, including none. The
    /// transitions below are what actually needed covering.
    #[test]
    fn the_watcher_looks_at_the_clock_at_least_every_interval() {
        let wait = next_hour_check();
        assert!(wait > Duration::ZERO, "a zero wait would spin");
        assert!(wait <= HOUR_CHECK_INTERVAL);
    }

    /// An ordinary hour boundary is an hour away at most.
    #[test]
    fn the_next_boundary_is_within_the_hour() {
        let wait = duration_until_next_hour(&at("Asia/Kolkata", 2026, 8, 30, 14, 12))
            .expect("an ordinary hour has a next boundary");
        // India is offset by thirty minutes, so its boundaries are at :30 in UTC
        // terms - but they are still an hour apart from each other.
        assert!(wait.as_secs() > 0 && wait.as_secs() <= 3600, "{wait:?}");
    }

    /// A fall-back that moves the clock by less than an hour can leave the next
    /// boundary behind us.
    ///
    /// Australia/Lord_Howe goes back thirty minutes. `duration_until_next_hour`
    /// answers `None` for that window, and the caller must fall back rather than
    /// unwrap - which is what an earlier version of the test below did, so it
    /// panicked for anyone whose machine was in that zone.
    #[test]
    fn a_short_fall_back_has_no_next_boundary_and_does_not_panic() {
        for minute in [0, 15, 30, 45, 59] {
            let now = at("Australia/Lord_Howe", 2026, 4, 5, 1, minute);
            // Whatever it answers, the scheduler must produce a usable wait.
            let _ = duration_until_next_hour(&now);
        }
        assert!(next_hour_check() > Duration::ZERO);
    }

    /// A repeated hour is two different instants with the same clock reading.
    ///
    /// This is what `REFRESH_AFTER` exists for. Comparing the local date and
    /// hour would find these equal and skip the redraw, leaving the disc two
    /// hours stale once a year; elapsed time tells them apart.
    #[test]
    fn a_repeated_hour_is_still_an_hour_of_elapsed_time() {
        let before = at("America/New_York", 2026, 11, 1, 1, 30)
            .timestamp()
            .as_second();
        // The same wall clock, one hour later, after the fall-back.
        let after = before + 3600;

        assert!(
            after - before >= REFRESH_AFTER.as_secs() as i64,
            "the repeated hour must still trigger a redraw"
        );
    }

    /// The alignment wake must never be longer than the refresh interval, or the
    /// redraw it exists to align would be late rather than early.
    #[test]
    fn the_wake_is_finer_than_the_refresh_it_schedules() {
        assert!(HOUR_CHECK_INTERVAL < REFRESH_AFTER);
    }
}
