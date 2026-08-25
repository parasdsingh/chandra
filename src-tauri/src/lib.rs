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

/// How long after local midnight to redraw the tray.
///
/// A few seconds of slack, so a clock adjustment across the boundary cannot make
/// the timer fire on the day it just left.
const MIDNIGHT_SLACK: Duration = Duration::from_secs(5);

/// Longest the midnight watcher parks for in one go.
///
/// `thread::sleep` does not advance while the machine is asleep, so a laptop
/// shut over midnight returns from a single long sleep hours after the boundary
/// it was waiting for and keeps yesterday's disc in the menu bar until the rest
/// of that sleep has elapsed awake - which can be another whole day. Looking at
/// the clock periodically bounds that to this interval.
///
/// Not polling in the sense D-017 rules out: what happens on each wake is two
/// date computations costing microseconds, and the redraw still happens only
/// when the displayed day has actually rolled over, which is the one trigger
/// D-017 names.
const MIDNIGHT_CHECK_INTERVAL: Duration = Duration::from_secs(600);

pub fn run() {
    tauri::Builder::default()
        .manage(panel::CurrentSubject::default())
        .manage(panel::PanelMaterial::default())
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::moon_month,
            commands::graha_month,
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
            watch_for_midnight(handle.clone());

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

/// Redraws the tray when the local date changes.
///
/// The moon disc shows the illumination at local noon of the current date, so it
/// changes exactly once a day (`docs/DESIGN.md` 7.1). The thread waits for that
/// boundary rather than recomputing on a timer, and compares dates rather than
/// trusting that it woke when it meant to: a sleep does not run while the
/// machine is suspended, and one that meant to end at midnight can return long
/// after it.
fn watch_for_midnight(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        // The tray was drawn for today as the app started.
        let mut drawn_for = jiff::Zoned::now().date();

        loop {
            std::thread::sleep(next_midnight_check());

            let today = jiff::Zoned::now().date();
            if today == drawn_for {
                continue;
            }
            drawn_for = today;

            // Same rule as everywhere else: the status item is an AppKit object
            // and must only be touched on the main thread.
            let handle = app.clone();
            let dispatched = app.run_on_main_thread(move || {
                if let Err(error) = tray::refresh_icons(&handle) {
                    // A failed redraw leaves yesterday's disc in the menu bar,
                    // which is wrong but not fatal, so the loop continues.
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
/// Until just after the next local midnight, or [`MIDNIGHT_CHECK_INTERVAL`],
/// whichever comes first.
fn next_midnight_check() -> Duration {
    duration_until_local_midnight()
        .map(|until| until + MIDNIGHT_SLACK)
        .unwrap_or(MIDNIGHT_CHECK_INTERVAL)
        .min(MIDNIGHT_CHECK_INTERVAL)
}

/// Time from now until the next local midnight.
///
/// Derived from the tz database rather than by rounding up to the next multiple
/// of 24 hours: a daylight saving transition makes a day 23 or 25 hours long,
/// and arithmetic would drift a day out of step twice a year.
fn duration_until_local_midnight() -> Option<Duration> {
    let now = jiff::Zoned::now();
    let tomorrow = now.date().tomorrow().ok()?;
    let midnight = tomorrow.to_zoned(now.time_zone().clone()).ok()?;

    let seconds = midnight.timestamp().as_second() - now.timestamp().as_second();
    (seconds > 0).then(|| Duration::from_secs(seconds as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midnight_is_always_within_the_next_twenty_six_hours() {
        // 26 rather than 24: a zone that moves its clock back makes the longest
        // possible day 25 hours, and the slack is added on top.
        let wait = duration_until_local_midnight().expect("a next midnight exists");
        assert!(wait.as_secs() > 0);
        assert!(
            wait.as_secs() <= 26 * 3600,
            "waiting {} hours is not a next midnight",
            wait.as_secs() / 3600
        );
    }

    /// The watcher must never park past the point where it could notice a
    /// machine that was asleep over the boundary.
    #[test]
    fn the_watcher_looks_at_the_clock_at_least_every_interval() {
        let wait = next_midnight_check();
        assert!(wait > Duration::ZERO, "a zero wait would spin");
        assert!(
            wait <= MIDNIGHT_CHECK_INTERVAL,
            "parked for {wait:?}, past the point a resumed machine would be noticed"
        );
    }
}
