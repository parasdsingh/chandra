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
/// The thread compares clocks rather than trusting that it woke when it meant
/// to: a sleep does not run while the machine is suspended, and one that meant
/// to end on the hour can return long after it.
fn watch_the_clock(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        // The tray was drawn for this hour as the app started.
        let mut drawn_for = current_hour();

        loop {
            std::thread::sleep(next_hour_check());

            let hour = current_hour();
            if hour == drawn_for {
                continue;
            }
            drawn_for = hour;

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

/// The local date and hour, which is what a redraw is keyed on.
///
/// A pair rather than an hour alone: comparing hours across midnight would find
/// 23 and 0 equal one day apart only by accident, and a machine suspended for
/// exactly a day would skip the redraw entirely.
fn current_hour() -> (jiff::civil::Date, i8) {
    let now = jiff::Zoned::now();
    (now.date(), now.hour())
}

/// How long to wait before looking at the clock again.
///
/// Until just after the next local hour boundary, or [`HOUR_CHECK_INTERVAL`],
/// whichever comes first.
fn next_hour_check() -> Duration {
    duration_until_next_hour()
        .map(|until| until + HOUR_SLACK)
        .unwrap_or(HOUR_CHECK_INTERVAL)
        .min(HOUR_CHECK_INTERVAL)
}

/// Time from now until the next local hour boundary.
///
/// Derived from the tz database rather than from the minutes and seconds on the
/// clock: a zone that moves its offset by thirty or forty-five minutes - India,
/// Nepal, parts of Australia - has hour boundaries that are not on the hour in
/// any other zone, and a daylight saving transition can make the step to the
/// next one shorter or longer than an hour.
fn duration_until_next_hour() -> Option<Duration> {
    let now = jiff::Zoned::now();
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

    #[test]
    fn the_next_hour_is_always_within_two_hours() {
        // Two rather than one: a zone that moves its clock back repeats an hour,
        // and the slack is added on top.
        let wait = duration_until_next_hour().expect("a next hour exists");
        assert!(wait.as_secs() > 0);
        assert!(
            wait.as_secs() <= 2 * 3600,
            "waiting {} seconds is not a next hour",
            wait.as_secs()
        );
    }

    /// The watcher must never park past the point where it could notice a
    /// machine that was asleep over the boundary.
    #[test]
    fn the_watcher_looks_at_the_clock_at_least_every_interval() {
        let wait = next_hour_check();
        assert!(wait > Duration::ZERO, "a zero wait would spin");
        assert!(
            wait <= HOUR_CHECK_INTERVAL,
            "parked for {wait:?}, past the point a resumed machine would be noticed"
        );
    }
}
