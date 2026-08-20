//! Panel and settings window lifecycle.

use std::sync::mpsc;
use std::time::Duration;

use chandra_ephemeris::Graha;
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use crate::error::{AppError, Result};
use crate::location::Outcome;
use crate::state::AppState;

pub const PANEL_LABEL: &str = "panel";
pub const SETTINGS_LABEL: &str = "settings";

/// Panel width, fixed forever (`docs/DESIGN.md` 2.2).
const PANEL_WIDTH: f64 = 320.0;

/// The window is created at the maximum height the panel can ever reach and
/// never resized. The visible panel is a div inside it that animates its own
/// height, with the surrounding area transparent and click-through.
///
/// Resizing the window per frame to follow the detail expanding would mean an
/// IPC call every frame for 220ms, and the frame and its contents would visibly
/// disagree whenever one lagged the other. A fixed window has neither problem.
const PANEL_HEIGHT: f64 = 620.0;

const SETTINGS_WIDTH: f64 = 520.0;
const SETTINGS_HEIGHT: f64 = 420.0;

/// How long to wait for CoreLocation before giving up and keeping the offline
/// resolution. Long enough for the authorisation prompt to be answered, short
/// enough that the settings pane does not appear stuck.
const LOCATION_TIMEOUT: Duration = Duration::from_secs(20);

/// Creates the panel window, hidden.
///
/// Built once at startup rather than per open: creating a webview takes long
/// enough to be visible, and the panel must appear immediately on a tray click.
pub fn create(app: &AppHandle) -> Result<WebviewWindow> {
    let window = WebviewWindowBuilder::new(app, PANEL_LABEL, WebviewUrl::App("index.html".into()))
        .title("Chandra")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .shadow(false)
        .build()
        .map_err(|e| AppError::Engine(format!("cannot create the panel window: {e}")))?;

    Ok(window)
}

/// Shows the panel for a subject, or hides it if it is already showing that one.
///
/// `tray_rect` is the clicked item's rectangle on screen, taken from the click
/// event itself.
pub fn toggle(app: &AppHandle, subject: Graha, tray_rect: Rect) {
    let Some(window) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };

    let showing_same_subject = window.is_visible().unwrap_or(false) && current_subject(app) == Some(subject);
    if showing_same_subject {
        hide(app);
        return;
    }

    set_current_subject(app, subject);
    let _ = window.emit("chandra://subject", subject);

    place(&window, tray_rect);

    // On macOS the app must be shown as well as the window: hiding only the
    // window leaves the app present in Mission Control and Cmd-Tab, and showing
    // only the window leaves it unable to take key focus.
    let _ = app.show();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        let _ = window.hide();
    }
    // Settings is a normal window; hiding the whole app while it is open would
    // take it off screen too.
    let settings_open = app
        .get_webview_window(SETTINGS_LABEL)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if !settings_open {
        let _ = app.hide();
    }
}

/// Centres the panel under the tray item, 6px below the menu bar, fully on
/// screen (`docs/DESIGN.md` 2.2).
fn place(window: &WebviewWindow, tray_rect: Rect) {
    const EDGE_MARGIN: f64 = 12.0;
    const MENU_BAR_GAP: f64 = 6.0;

    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let scale = monitor.scale_factor();
    let monitor_position = monitor.position().to_logical::<f64>(scale);
    let monitor_size = monitor.size().to_logical::<f64>(scale);

    let tray_position = tray_rect.position.to_logical::<f64>(scale);
    let tray_size = tray_rect.size.to_logical::<f64>(scale);
    let tray_centre = tray_position.x + tray_size.width / 2.0;

    // Clamping matters: a tray item near a screen corner would otherwise put
    // part of the panel off screen.
    let minimum_x = monitor_position.x + EDGE_MARGIN;
    let maximum_x = monitor_position.x + monitor_size.width - PANEL_WIDTH - EDGE_MARGIN;
    let x = (tray_centre - PANEL_WIDTH / 2.0).clamp(minimum_x, maximum_x.max(minimum_x));

    // The work area begins below the menu bar, so its top edge gives the menu
    // bar height without measuring or assuming one.
    let y = (monitor.work_area().position.y as f64) / scale + MENU_BAR_GAP;

    let _ = window.set_position(LogicalPosition::new(x, y));
}

pub fn open_settings(app: &AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = app.show();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        SETTINGS_LABEL,
        WebviewUrl::App("index.html?window=settings".into()),
    )
    .title("Chandra Settings")
    .inner_size(SETTINGS_WIDTH, SETTINGS_HEIGHT)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .visible(false)
    .build()
    .map_err(|e| AppError::Engine(format!("cannot create the settings window: {e}")))?;

    let _ = window.set_size(LogicalSize::new(SETTINGS_WIDTH, SETTINGS_HEIGHT));
    let _ = app.show();
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

/// Asks macOS for the device's coordinates, waits for an answer, and records it.
///
/// CoreLocation delivers on the main thread's run loop, so the request is
/// dispatched there and the answer comes back over a channel. The wait is
/// bounded: an unanswered authorisation prompt must not leave the caller hanging.
pub async fn request_device_location(app: &AppHandle) {
    let (sender, receiver) = mpsc::channel();
    let dispatch = app.run_on_main_thread(move || {
        crate::location::request(move |outcome| {
            let _ = sender.send(outcome);
        });
    });
    if dispatch.is_err() {
        return;
    }

    let handle = app.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || receiver.recv_timeout(LOCATION_TIMEOUT))
        .await
        .ok()
        .and_then(|result| result.ok());

    if let Some(Outcome::Located {
        latitude,
        longitude,
        elevation,
    }) = outcome
    {
        let state = handle.state::<AppState>();
        if state
            .accept_device_location(latitude, longitude, elevation)
            .is_ok()
        {
            let _ = crate::tray::refresh_icons(&handle);
            let _ = handle.emit("chandra://location", state.location());
        }
    }
}

/// Which subject the panel is currently showing.
///
/// Held in Tauri's managed state rather than in [`AppState`] because it is
/// window state, not almanac state, and the two have different lifetimes.
#[derive(Default)]
pub struct CurrentSubject(std::sync::Mutex<Option<Graha>>);

fn current_subject(app: &AppHandle) -> Option<Graha> {
    *app.state::<CurrentSubject>().0.lock().expect("subject lock")
}

fn set_current_subject(app: &AppHandle, subject: Graha) {
    *app.state::<CurrentSubject>().0.lock().expect("subject lock") = Some(subject);
}
