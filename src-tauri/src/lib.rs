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
use tauri_plugin_autostart::MacosLauncher;

use crate::state::AppState;

/// How long after local midnight to redraw the tray.
///
/// A few seconds of slack, so a clock adjustment across the boundary cannot make
/// the timer fire on the day it just left.
const MIDNIGHT_SLACK: Duration = Duration::from_secs(5);

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(panel::CurrentSubject::default())
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::moon_month,
            commands::graha_month,
            commands::day_detail,
            commands::snapshot,
            commands::ayanamsa_degrees,
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
                // Closing the settings window must not quit a menu bar app.
                api.prevent_exit();
            }
        });
}

/// Redraws the tray shortly after every local midnight.
///
/// The moon disc shows the illumination at local noon of the current date, so it
/// changes exactly once a day (`docs/DESIGN.md` 7.1). A thread that sleeps until
/// the boundary costs nothing while it waits, which keeps idle CPU at zero as
/// D-017 requires; polling on a short interval would not.
fn watch_for_midnight(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        let wait = duration_until_local_midnight().unwrap_or(Duration::from_secs(3600));
        std::thread::sleep(wait + MIDNIGHT_SLACK);

        // Same rule as everywhere else: the status item is an AppKit object and
        // must only be touched on the main thread.
        let handle = app.clone();
        let dispatched = app.run_on_main_thread(move || {
            if let Err(error) = tray::refresh_icons(&handle) {
                // A failed redraw leaves yesterday's disc in the menu bar, which
                // is wrong but not fatal, so the loop continues to the next day.
                eprintln!("chandra: could not redraw the menu bar: {error}");
            }
        });
        if dispatched.is_err() {
            // The app is shutting down; nothing left to redraw.
            return;
        }
    });
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
}
