//! Panel window lifecycle. It is the only window the app creates; settings are
//! a view inside it.

use std::sync::mpsc;
use std::time::Duration;

use chandra_ephemeris::Graha;
use tauri::{
    AppHandle, Emitter, LogicalPosition, Manager, Rect, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use crate::error::{AppError, Result};
use crate::location::Outcome;
use crate::state::AppState;

pub const PANEL_LABEL: &str = "panel";

/// Carries the subject to the already-running panel as it opens.
pub const OPEN_EVENT: &str = "chandra://open";

/// Panel width, fixed forever (`docs/DESIGN.md` 2.2).
const PANEL_WIDTH: f64 = 320.0;

/// Panel height, and therefore window height.
///
/// Constant: 12 padding + 40 header + 4 + 264 region + 12 padding. Every view -
/// calendar, day, settings - swaps inside that 264px region rather than growing
/// the panel, so the window never resizes and the material behind it never has
/// to be resized either.
const PANEL_HEIGHT: f64 = 332.0;

/// Corner radius of the panel, matched by the window material behind it.
const PANEL_RADIUS: f64 = 12.0;

/// Resizes the window to the panel's drawn size.
///
/// The panel's layout is fixed at 320 by 332 and the page scales itself with a
/// transform, so the window has to be told the same multiplier or the drawn
/// panel and the window it lives in stop agreeing - which shows as a clipped
/// corner at one end and a band of desktop at the other.
pub fn apply_scale(app: &AppHandle, scale: f64) {
    let Some(window) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };
    let _ = window.set_size(tauri::LogicalSize::new(
        PANEL_WIDTH * scale,
        PANEL_HEIGHT * scale,
    ));
    // The material's corner is re-cut to match. The panel's own radius is a CSS
    // length and scales with the transform, so leaving the material at the
    // composed 12pt showed its corner through the panel's rounder one.
    apply_material_with_radius(&window, PANEL_RADIUS * scale);
}

/// The panel's drawn width, which is what placement has to centre.
fn scaled_width(app: &AppHandle) -> f64 {
    PANEL_WIDTH * app.state::<AppState>().settings().appearance.clamped()
}

/// How long to wait for CoreLocation before giving up and keeping the offline
/// resolution. Long enough for the authorisation prompt to be answered, short
/// enough that the location section does not appear stuck.
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

    // Dark regardless of the system appearance: the palette is a single dark
    // one (D-011), and the light variant of the material would put near-white
    // text on a near-white backdrop.
    let _ = window.set_theme(Some(tauri::Theme::Dark));

    app.state::<PanelMaterial>()
        .0
        .store(apply_material(&window), std::sync::atomic::Ordering::SeqCst);

    Ok(window)
}

/// Whether the system's popover material is actually behind the panel.
///
/// Not cosmetic. The panel window is transparent and the panel itself paints a
/// scrim, so with no material there is nothing for the scrim to darken and the
/// desktop shows through it. The front end has to be told, because it is the
/// half that can paint an opaque ground instead.
#[derive(Default)]
pub struct PanelMaterial(std::sync::atomic::AtomicBool);

/// Whether the panel is sitting on the system's popover material.
pub fn has_material(app: &AppHandle) -> bool {
    app.state::<PanelMaterial>()
        .0
        .load(std::sync::atomic::Ordering::SeqCst)
}

/// Gives the panel the system's popover material, reporting whether it took.
///
/// macOS draws its own menu bar popovers on a translucent, blurred backdrop.
/// A flat fill sits oddly among them, so the panel uses the same public
/// AppKit material. The window is already transparent and the panel paints only
/// a thin scrim over this, so the blur is what shows through.
#[cfg(target_os = "macos")]
fn apply_material(window: &WebviewWindow) -> bool {
    apply_material_with_radius(window, PANEL_RADIUS)
}

/// No vibrancy outside macOS, so the panel always paints its own ground.
#[cfg(not(target_os = "macos"))]
fn apply_material(_window: &WebviewWindow) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn apply_material_with_radius(window: &WebviewWindow, radius: f64) -> bool {
    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

    // Popover is the material the system uses for exactly this kind of window.
    // The radius matches the panel's own corner radius, so the material does not
    // show as square corners behind rounded content.
    match apply_vibrancy(
        window,
        NSVisualEffectMaterial::Popover,
        Some(NSVisualEffectState::Active),
        Some(radius),
    ) {
        Ok(()) => true,
        Err(error) => {
            // Not fatal, but not silent either: the panel has to stop being a
            // scrim and paint its own ground, which is what the answer is for.
            eprintln!("chandra: could not apply the panel material: {error}");
            false
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_material_with_radius(_window: &WebviewWindow, _radius: f64) -> bool {
    false
}

/// Shows the panel for a subject, or hides it if it is already showing that one.
///
/// `tray_rect` is the clicked item's rectangle on screen, taken from the click
/// event itself.
pub fn toggle(app: &AppHandle, subject: Graha, tray_rect: Rect) {
    let Some(window) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };

    let showing_same_subject =
        window.is_visible().unwrap_or(false) && current_subject(app) == Some(subject);
    if showing_same_subject {
        hide(app);
        return;
    }

    // Positioned before it is shown, so it never appears in the wrong place and
    // then jumps. A failure here is reported and then ignored: the panel opens
    // where it last was, which is wrong on a second display but is a great deal
    // better than a tray item that does nothing at all. Refusing to show it made
    // one bad monitor lookup uninstall the app's only surface.
    if let Err(error) = place(&window, tray_rect, scaled_width(app)) {
        eprintln!("chandra: {error}");
    }

    set_current_subject(app, subject);

    // The page is told, and never reloaded.
    //
    // It used to be navigated to `?subject=<key>` on every open, which is
    // reliable - a page reads its own URL on load, no reactivity required - and
    // is why it was chosen after pushing the value into the running page failed
    // (I-038). It also threw the whole document away every time: measured, the
    // window appeared empty for 100 to 200ms and the calendar landed at 265 to
    // 414ms. Without the reload the same open paints in about 100ms with no
    // empty frame, because the months are already in hand.
    //
    // Told twice, and idempotently. `chandra://open` carries the subject and is
    // what makes the switch immediate; the page also re-reads the subject when
    // the window takes focus, which is a pull and so cannot be missed. Both call
    // the same function with the same value, so arriving twice is arriving once.
    let _ = window.emit_to(PANEL_LABEL, OPEN_EVENT, subject.key());

    // On macOS the app must be shown as well as the window: hiding only the
    // window leaves the app present in Mission Control and Cmd-Tab, and showing
    // only the window leaves it unable to take key focus.
    let _ = app.show();
    let _ = window.show();
    let _ = window.set_focus();
}

/// Tells the running panel which subject it is showing.
///
/// Called as the window is shown and again when it takes focus. The handler at
/// the other end is idempotent, so being told twice is being told once.
pub fn announce_subject(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        let _ = window.emit_to(PANEL_LABEL, OPEN_EVENT, subject_or_default(app).key());
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        let _ = window.hide();
    }
    let _ = app.hide();
}

/// Centres the panel under the tray item, 6px below the menu bar, fully on
/// screen (`docs/DESIGN.md` 2.2).
///
/// The monitor is the one the clicked item is on, found from the item's own
/// rectangle. `current_monitor` answers for the panel instead, which is the
/// display it was last shown on: on a Retina laptop beside a 1x external screen
/// that both picked the wrong display and divided a physical rectangle by the
/// wrong scale, then clamped the result into the wrong bounds. It never
/// self-corrected, because the answer it gave became the input to the next one.
fn place(window: &WebviewWindow, tray_rect: Rect, width: f64) -> Result<()> {
    const EDGE_MARGIN: f64 = 12.0;
    const MENU_BAR_GAP: f64 = 6.0;

    // Which display the tray item is on, not the one the panel was left on: an
    // item clicked on a second screen must open its panel there.
    //
    // The lookup is fed two candidate points because the units do not line up.
    // `tray-icon` builds the rect by multiplying by the status item's backing
    // scale, so what arrives is physical; `tao` implements the lookup as
    // `CGRectContainsPoint(CGDisplayBounds(...))` and treats those bounds as
    // logical. On a 2x display the physical point is therefore twice as far
    // right as the bounds it is tested against, lands outside the only screen,
    // and the lookup finds nothing at all - which is what stopped the panel
    // opening on this Mac. The status item's own scale is not reachable from
    // here, so the panel's is the best estimate available, and trying the raw
    // point first keeps a 1x display exact.
    let anchor = tray_rect.position.to_physical::<f64>(1.0);
    let estimate = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|monitor| monitor.scale_factor())
        .unwrap_or(1.0);

    let monitor = window
        .monitor_from_point(anchor.x, anchor.y)
        .ok()
        .flatten()
        .or_else(|| {
            window
                .monitor_from_point(anchor.x / estimate, anchor.y / estimate)
                .ok()
                .flatten()
        })
        // The panel's own display, then the primary one. Both are wrong on a
        // second screen, and both are better than the alternative, which is a
        // tray item that does nothing.
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| {
            AppError::Engine(format!(
                "no display contains the tray item at {}, {}",
                anchor.x, anchor.y
            ))
        })?;
    let scale = monitor.scale_factor();
    let monitor_position = monitor.position().to_logical::<f64>(scale);
    let monitor_size = monitor.size().to_logical::<f64>(scale);

    let tray_position = tray_rect.position.to_logical::<f64>(scale);
    let tray_size = tray_rect.size.to_logical::<f64>(scale);
    let tray_centre = tray_position.x + tray_size.width / 2.0;

    // Clamping matters: a tray item near a screen corner would otherwise put
    // part of the panel off screen.
    let minimum_x = monitor_position.x + EDGE_MARGIN;
    let maximum_x = monitor_position.x + monitor_size.width - width - EDGE_MARGIN;
    let x = (tray_centre - width / 2.0).clamp(minimum_x, maximum_x.max(minimum_x));

    // The work area begins below the menu bar, so its top edge gives the menu
    // bar height without measuring or assuming one.
    let y = (monitor.work_area().position.y as f64) / scale + MENU_BAR_GAP;

    window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|error| AppError::Engine(format!("cannot position the panel: {error}")))
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
    let outcome =
        tauri::async_runtime::spawn_blocking(move || receiver.recv_timeout(LOCATION_TIMEOUT))
            .await
            .ok()
            .and_then(|result| result.ok());

    // Whatever the answer, the wait is over and nothing more will be read from
    // the manager. A request that timed out would otherwise hold it and its
    // delegate until the next one replaced them.
    let _ = handle.run_on_main_thread(crate::location::release);

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
            let _ = handle.emit("chandra://location", state.location());
            let for_tray = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                let _ = crate::tray::refresh_icons(&for_tray);
            });
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
    *app.state::<CurrentSubject>()
        .0
        .lock()
        .expect("subject lock")
}

/// The subject the panel is showing, for the front end to read on mount.
///
/// The tray also emits an event when the subject changes, but an event is a
/// one-shot: a webview that has not finished registering its listener misses it
/// and shows the Moon under whichever tray item was clicked. Reading the
/// authoritative value on mount closes that window.
pub fn subject_or_default(app: &AppHandle) -> Graha {
    current_subject(app).unwrap_or(Graha::Chandra)
}

fn set_current_subject(app: &AppHandle, subject: Graha) {
    *app.state::<CurrentSubject>()
        .0
        .lock()
        .expect("subject lock") = Some(subject);
}

#[cfg(test)]
mod tests {
    /// The tray rect arrives physical; the lookup compares against logical.
    ///
    /// On a 2x display the raw point is off the right-hand edge of the only
    /// screen, so the lookup finds nothing and the panel never opens. Dividing
    /// by the scale brings it back inside. This pins the arithmetic; which
    /// display the window server actually returns needs a window server.
    #[test]
    fn a_physical_tray_point_is_outside_logical_bounds_on_a_retina_display() {
        // A 1512pt wide Retina laptop: the item sits at logical x 1300.
        let logical_x = 1300.0_f64;
        let scale = 2.0_f64;
        let physical_x = logical_x * scale;
        let screen_width = 1512.0_f64;

        assert!(
            physical_x > screen_width,
            "the raw point must fall outside the display, which is the defect"
        );
        assert!(
            physical_x / scale <= screen_width,
            "dividing by the scale must bring it back inside"
        );
        assert_eq!(physical_x / scale, logical_x);
    }
}
